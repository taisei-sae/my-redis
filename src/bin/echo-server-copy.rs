use tokio::{
    // io::{AsyncReadExt, AsyncWriteExt},
    io,
    net::{TcpListener, TcpStream},
};

#[tokio::main]
// Set the return type to io::Result<()> so that
// Rust runtime can set an appropriate termination code.
// Also I can use `?` instead of unwrap() and can avoid panic.
async fn main() -> io::Result<()> {
    // `TcpListener` is a server, waiting for connection from clients.
    // This creates `TcpStream` instances for each accepted connection.
    // This `127.0.0.1:6142` is the address of this listener.
    let _listener = TcpListener::bind("127.0.0.1:6142").await?;

    // loop {
    //     let (stream, address) = _listener.accept().await?;
    //     tokio::spawn(async move { todo!() });
    // }

    // `TcpStream` is a connection between server and client,
    // can be used by both client and server.
    // This `127.0.0.1:6142` is the address of remote host,
    // so this is trying creating a connection with remote host.
    let mut stream = TcpStream::connect("127.0.0.1:6142").await?;

    // `io::split` uses Arc and Mutex so it is a bit expensive.
    // let (mut rd, mut wr) = io::split(stream);

    tokio::spawn(async move {
        // `split()` doesn't use Mutex or arc so it is zero-cost.
        // Also it just takes reference of stream, it can be called within
        // the task.
        let (mut rd, mut wr) = stream.split();

        if io::copy(&mut rd, &mut wr).await.is_err() {
            eprintln!("Failed to copy");
        }
    });

    Ok(())
}
