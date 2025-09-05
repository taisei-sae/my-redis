use std::collections::HashMap;

use mini_redis::{Command, Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

async fn process(socket: TcpStream) {
    let mut db = HashMap::new();
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Command::Set(cmd) => {
                db.insert(cmd.key().to_string(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            Command::Get(cmd) => {
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented: {:?}", cmd),
        };

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
