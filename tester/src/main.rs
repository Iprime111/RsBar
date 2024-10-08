use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let mut stream = UnixStream::connect("/tmp/rsbar_call.sock").await?;
    let (read_stream, mut write_stream) = stream.split();
    let mut reader = tokio::io::BufReader::new(read_stream);

    write_stream.write_all(b"volume/toggleMuted/\n").await?;
    let _ = write_stream.flush();
    let mut response = String::new();

    println!("Sent request 1");
    
    reader.read_line(&mut response).await?;
    println!("Response: {response}");
    response.clear();

    let mut stream_event = UnixStream::connect("/tmp/rsbar_event.sock").await?;
    let (read_stream_event, mut write_stream_event) = stream_event.split();
    let reader_event = tokio::io::BufReader::new(read_stream_event);


    write_stream_event.write_all(b"hyprland/workspace\n").await?;
    let _ = write_stream_event.flush();

    println!("Sent request 2");

    let mut lines = reader_event.lines();

    while let Some(response) = lines.next_line().await? {
        println!("Response: {response}");
    }

    Ok(())
}
