use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::{unix::{OwnedReadHalf, OwnedWriteHalf}, UnixStream}};

pub struct ChannelsData {
    pub event_subscription_tx: tokio::sync::mpsc::Sender<String>,
    pub event_rx: tokio::sync::broadcast::Receiver<RsbarEvent>,
    pub call_tx:  tokio::sync::broadcast::Sender<String>,
}

impl Clone for ChannelsData {
    fn clone(&self) -> Self {
        ChannelsData {
            event_subscription_tx: self.event_subscription_tx.clone(),
            event_rx:              self.event_rx.resubscribe(),
            call_tx:               self.call_tx.clone()
        }
    }
}

struct UnixSocketConnection {
    write_stream: OwnedWriteHalf,
    reader:       BufReader<OwnedReadHalf>,
}

#[derive(Clone)]
pub struct RsbarEvent {
    pub name:    String,
    pub value:   String,
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

async fn send_message(stream: &mut OwnedWriteHalf, message: &str) -> tokio::io::Result<()> {
    stream.write(message.as_bytes()).await?;
    stream.write(b"\0").await?;
    stream.flush().await?;

    Ok(())
}

pub async fn setup_unix_sockets() -> tokio::io::Result<ChannelsData> {
    let mut event_socket_data = connect_to_unix_socket("/tmp/rsbar_event.sock").await?;
    let mut call_socket_data  = connect_to_unix_socket("/tmp/rsbar_call.sock").await?;

    let mut event_reader = event_socket_data.reader;
    
    let (event_subscription_tx, mut event_subscription_rx) = tokio::sync::mpsc::channel::<String>(32);
    let (event_tx, event_rx)   = tokio::sync::broadcast::channel::<RsbarEvent>(32);
    let (call_tx, mut call_rx) = tokio::sync::broadcast::channel::<String>(32);

    // TODO subscribe to events
    
    tokio::spawn(async move {
        while let Some(new_event) = event_subscription_rx.recv().await {
            // TODO process error
            println!("Subscribing to event: {}", new_event);
            let _ = send_message(&mut event_socket_data.write_stream, new_event.as_str()).await;
        }
    });

    tokio::spawn(async move {
        let mut event_vec = Vec::new();

        while event_reader.read_until(b'\0', &mut event_vec).await.unwrap() > 0 {
            event_vec.pop();
            let event = String::from_utf8(event_vec.clone()).unwrap();

            println!("Got event: {}", event);

            let split_result = split_at_nth_char_ex(&event, '/', 1);
        
            if split_result.is_none() {
                println!("Bad event format");
                continue;
            }

            let (name, value) = split_result.unwrap();

            let event_struct = RsbarEvent {
                name:  name.to_string(),
                value: value.to_string(),                
            };

            let _ = event_tx.send(event_struct);

            event_vec.clear();
        }
    });

    tokio::spawn(async move {
        while let Some(call) = call_rx.recv().await.ok() {
            println!("Calling: {}", call);

            // TODO process errors
            let _ = send_message(&mut call_socket_data.write_stream, call.as_str()).await;
        }
    });

    Ok(ChannelsData {
        event_subscription_tx,
        event_rx,
        call_tx,
    })
}

fn split_at_nth_char(s: &str, p: char, n: usize) -> Option<(&str, &str)> {
    s.match_indices(p).nth(n).map(|(index, _)| s.split_at(index))
}

fn split_at_nth_char_ex(s: &str, p: char, n: usize) -> Option<(&str, &str)> {
    split_at_nth_char(s, p, n).map(|(left, right)| {
        (
            left,
            // Trim 1 character.
            &right[right.char_indices().nth(1).unwrap().0..],
        )
    })
}
