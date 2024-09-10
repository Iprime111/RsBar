mod server_context;
mod volume_context;
mod brightness_context;
mod hyprland_context;
mod time_context;

use brightness_context::BrightnessContext;
use hyprland_context::HyprlandContext;
use server_context::ServerContext;

use tokio::net::{UnixStream, UnixListener};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::{sync::Mutex, task, time};
use volume_context::VolumeContext;
use std::io::ErrorKind;
use tokio::sync::mpsc;
use std::{sync::Arc, time::Duration};

const POLLING_INTERVAL: u64 = 1000; 

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let mut main_context = ServerContext::new();

    main_context.add_context(VolumeContext::new());
    main_context.add_context(BrightnessContext::new());
    main_context.add_context(HyprlandContext::new());

    main_context.init().await?;

    let main_context_shared = Arc::new(Mutex::new(main_context));
    let context_clone_1 = main_context_shared.clone();
    let context_clone_2 = main_context_shared.clone();
    
    // TODO seaparate function 
    let call_listener = bind_socket("/tmp/rsbar_call.sock")?;

    task::spawn(async move {
        loop {
            match call_listener.accept().await {
                Ok((stream, _addr)) => { let _ = task::spawn(handle_call_client(stream, context_clone_1.clone())).await; },
                Err(error) => { println!("Client connection failed (call request attempt): {:?}", error); },
            }
        }
    });

    let event_listener = bind_socket("/tmp/rsbar_event.sock")?;

    task::spawn(async move {
        loop {
            match event_listener.accept().await {
                Ok((stream, _addr)) => { let _ = task::spawn(handle_event_client(stream, context_clone_2.clone())).await; },
                Err(error) => { println!("Client connection failed (event request attempt): {:?}", error); },
            }
        }
    });

    let mut interval = time::interval(Duration::from_millis(POLLING_INTERVAL));

    // Update cycle
    loop {
        main_context_shared.lock().await.update().await?;
        interval.tick().await;
    }
}

async fn handle_call_client(stream: UnixStream, context: Arc<Mutex<ServerContext>>) -> tokio::io::Result<()> {
    let (read_stream, mut write_stream) = stream.into_split();
    let reader = tokio::io::BufReader::new(read_stream);
    let mut lines = reader.lines();

    while let Some(request) = lines.next_line().await? {
        println!("Got new call request: {}", request);
        
        let response = context.lock().await.new_call(&request).await;
        
        if response.is_none() {
            println!("Invalid request!");
            return Err(std::io::Error::new(ErrorKind::Other, ""));
        }

        let response_str = response.unwrap();
        
        println!("Response: {}", response_str);
        write_response(&response_str, &mut write_stream).await?;
    }

    Ok(())
}

async fn handle_event_client(stream: UnixStream, context: Arc<Mutex<ServerContext>>) -> tokio::io::Result<()> {
    let (read_stream, mut write_stream) = stream.into_split();
    let reader = tokio::io::BufReader::new(read_stream);
    let mut lines = reader.lines();

    let (tx, mut rx) = mpsc::channel::<String>(32);

    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            println!("New update: {}", message);

            let _ = write_response(message.as_ref(), &mut write_stream).await;
        }
    });

    while let Some(request) = lines.next_line().await? {
        println!("Got new event request: {}", request);

        let _ = context.lock().await.new_event_client(&request, tx.clone()).await;
    }

    Ok(())
}

fn bind_socket(path: impl AsRef<std::path::Path>) -> std::io::Result<UnixListener> {
    let path = path.as_ref();
    std::fs::remove_file(path)?;

    UnixListener::bind(path)
}

async fn write_response(response: &str, stream: &mut tokio::net::unix::OwnedWriteHalf) -> tokio::io::Result<()> {
    stream.write(response.as_bytes()).await?;
    stream.write(b"\n").await?;
    stream.flush().await?;
    
    Ok(())
}
