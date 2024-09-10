use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::{unix::{OwnedReadHalf, OwnedWriteHalf}, UnixStream}};

struct ChannelsData {
    event_channel_rx: tokio::sync::broadcast::Receiver<String>,
    call_channel_rx:  tokio::sync::broadcast::Receiver<String>,
    call_channel_tx:  tokio::sync::broadcast::Sender<String>,
}

struct UnixSocketConnection {
    write_stream: OwnedWriteHalf,
    reader:       BufReader<OwnedReadHalf>,
}

async fn connect_to_unix_socket(socket_path: &str) -> tokio::io::Result<UnixSocketConnection> {
    let server_stream               = UnixStream::connect(socket_path).await?;
    let (read_stream, write_stream) = server_stream.into_split();
    let reader                      = BufReader::new(read_stream);

    Ok(UnixSocketConnection {
        write_stream,
        reader
    })
}

pub async fn setup_unix_sockets() -> tokio::io::Result<ChannelsData> {
    let event_socket_data = connect_to_unix_socket("/tmp/rsbar_event.sock").await?;
    let mut call_socket_data  = connect_to_unix_socket("/tmp/rsbar_call.sock").await?;

    let mut event_lines = event_socket_data.reader.lines();
    
    let (event_tx, event_rx)   = tokio::sync::broadcast::channel::<String>(32);
    let (call_tx, mut call_rx) = tokio::sync::broadcast::channel::<String>(32);

    // TODO subscribe to events
    
    tokio::spawn(async move {
        while let Some(event) = event_lines.next_line().await.unwrap() {
            let _ = event_tx.send(event);
        }
    });

    tokio::spawn(async move {
        while let Some(call) = call_rx.recv().await.ok() {
            // TODO additional checks for '\n'
            let _ = call_socket_data.write_stream.write(call.as_bytes());
            let _ = call_socket_data.write_stream.write(b"\n");
            let _ = call_socket_data.write_stream.flush();

            let mut response = String::new();
            let _ = call_socket_data.reader.read_line(&mut response);

            // TODO transmit
        }
    });

    let call_rx_2 = call_tx.subscribe();

    Ok(ChannelsData {
        event_channel_rx: event_rx,
        call_channel_rx:  call_rx_2,
        call_channel_tx:  call_tx,
    })
}

