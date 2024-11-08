mod server_context;
mod volume_context;
mod brightness_context;
mod hyprland_context;
mod time_context;
mod rsbar_context;
mod battery_context;

use battery_context::BatteryContext;
use brightness_context::BrightnessContext;
use hyprland_context::HyprlandContext;
use server_context::ServerContext;

use time_context::TimeContext;
use tokio::net::{UnixStream, UnixListener};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::{sync::Mutex, task, time};
use volume_context::VolumeContext;
use tokio::sync::mpsc;
use std::{sync::Arc, time::Duration};
use log::{error, warn, info};

const POLLING_INTERVAL: u64 = 1000; 

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    colog::init();

    let mut main_context = ServerContext::new();

    main_context.add_context(VolumeContext::new());
    main_context.add_context(BrightnessContext::new());
    main_context.add_context(HyprlandContext::new());
    main_context.add_context(TimeContext::new());
    main_context.add_context(BatteryContext::new());

    main_context.init().await?;

    let main_context_shared = Arc::new(Mutex::new(main_context));
    
    spawn_listener_loops(main_context_shared.clone()).await?;

    let mut interval = time::interval(Duration::from_millis(POLLING_INTERVAL));

    // Update cycle
    loop {
        main_context_shared.lock().await.update().await?;
        interval.tick().await;
    }
}

async fn spawn_listener_loops(context: Arc<Mutex<ServerContext>>) -> tokio::io::Result<()> {
    let context_clone = context.clone();

    let call_listener = bind_socket("/tmp/rsbar_call.sock")?;

    task::spawn(async move {
        loop {
            match call_listener.accept().await {
                Ok((stream, _addr)) => { let _ = task::spawn(handle_call_client(stream, context.clone())).await; },
                Err(error) => { error!("Client connection failed (call request attempt): {:?}", error); },
            }
        }
    });

    let event_listener = bind_socket("/tmp/rsbar_event.sock")?;

    task::spawn(async move {
        loop {
            match event_listener.accept().await {
                Ok((stream, _addr)) => { let _ = task::spawn(handle_event_client(stream, context_clone.clone())).await; },
                Err(error) => { error!("Client connection failed (event request attempt): {:?}", error); },
            }
        }
    });

    Ok(())
}

async fn handle_call_client(stream: UnixStream, context: Arc<Mutex<ServerContext>>) -> tokio::io::Result<()> {
    let (read_stream, _) = stream.into_split();
    let mut reader = tokio::io::BufReader::new(read_stream);

    let mut request_vec = Vec::new();

    while reader.read_until(b'\0', &mut request_vec).await? > 0 {
        request_vec.pop();
        let request = String::from_utf8(request_vec.clone()).unwrap();

        info!("Got new call request: {}", request);
        
        if let Err(response) = context.lock().await.new_call(&request).await {
            warn!("Invalid request: {request}\n{response}");

            request_vec.clear();
            continue;
        }

        request_vec.clear();
    }

    Ok(())
}

async fn handle_event_client(stream: UnixStream, context: Arc<Mutex<ServerContext>>) -> tokio::io::Result<()> {
    let (read_stream, mut write_stream) = stream.into_split();
    let mut reader = tokio::io::BufReader::new(read_stream);

    let (tx, mut rx) = mpsc::channel::<String>(32);

    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            info!("New update: {}", message);

            // TODO somehow remove channel from client when it's disconnecting?
            if let Err(write_result) = write_response(message.as_ref(), &mut write_stream).await {
                warn!("Error occuried while sending event: {write_result}");            
    
                break;
            }
        }
    });

    let mut request_vec =  Vec::new();

    while reader.read_until(b'\0', &mut request_vec).await? > 0 {
        request_vec.pop();
        let request = String::from_utf8(request_vec.clone()).unwrap();

        info!("Got new event subscription request: {}", request);

        if let Err(result) = context.lock().await.new_event_client(&request, tx.clone()).await {
            warn!("Error occuried while subscribing to event {request}: {result}");
        }

        request_vec.clear();
    }

    Ok(())
}

fn bind_socket(path: impl AsRef<std::path::Path>) -> std::io::Result<UnixListener> {
    let path = path.as_ref();

    let _ = std::fs::remove_file(path);

    UnixListener::bind(path)
}

async fn write_response(response: &str, stream: &mut tokio::net::unix::OwnedWriteHalf) -> tokio::io::Result<()> {
    stream.write(response.as_bytes()).await?;
    stream.write(b"\0").await?;
    stream.flush().await?;
    
    Ok(())
}
