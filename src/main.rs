use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

async fn process(socket: TcpStream) {
    let mut connection = Connection::new(socket);
    if let Some(frame) = connection.read_frame().await.unwrap() {
        println!("Got: {:?}", frame);

        let response = Frame::Error("unimplemented".to_string());
        connection.write_frame(&response).await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    let lisner = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    loop {
        let (socket, _addr) = lisner.accept().await.unwrap();
        tokio::spawn(async {
            process(socket).await;
        });
    }
}
