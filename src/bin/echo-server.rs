use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await?;

    loop {
        let (mut stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            // Put buf in heap so that each async task avoid copying
            // this 1KB buffer into stack.
            // The net memory size is the same,
            // but the Future's stack has to be allocated continuously.
            // If Future's stack is large, the memory would be fragmented.
            let mut buf = vec![0; 1024];
            loop {
                match stream.read(&mut buf).await {
                    Ok(0) => {
                        // The remote side has closed.
                        // If forget this, the CPU usage will be 100%
                        // because this `read` returns immediately and
                        // the loop repeats forever.
                        return;
                    }
                    Ok(n) => {
                        if stream.write_all(&buf[..n]).await.is_err() {
                            return; // Unexpected socket error
                        }
                    }
                    Err(_) => {
                        return; // Unexpected socket error
                    }
                }
            }
        });
    }
}
