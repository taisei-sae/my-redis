use bytes::Bytes;
use mini_redis::client;

type Responder<T> = tokio::sync::oneshot::Sender<mini_redis::Result<T>>;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        value: Bytes,
        resp: Responder<()>,
    },
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    let tx2 = tx.clone();

    let task1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let cmd = Command::Get {
            key: "foo".to_string(),
            resp: resp_tx,
        };

        // Send the GET command to the server via client manager task
        if tx.send(cmd).await.is_err() {
            eprintln!("the other side of mpsc has gone away");
            return;
        };

        // Await the response from the server via client manager task
        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });

    let task2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let cmd = Command::Set {
            key: "foo".to_string(),
            value: "bar".into(),
            resp: resp_tx,
        };

        // Send the SET command to the server via client manager task
        if tx2.send(cmd).await.is_err() {
            eprintln!("the other side of mpsc has gone away");
            return;
        };

        // Await the response from the server via client manager task
        let res = resp_rx.await.unwrap();
        println!("GOT = {:?}", res);
    });

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Get { key, resp } => {
                    let val = client.get(&key).await;
                    let _ = resp.send(val);
                    // Don't need to await here because `send` on `oneshot` will always
                    // succeed or fail immediately without waiting.
                    // Also don't use unwrap() here,
                    // because if receives Err here, a panic will occur.
                    // Ignore errors
                }
                Command::Set { key, value, resp } => {
                    let res = client.set(&key, value).await;
                    let _ = resp.send(res);
                    // same as above
                }
            }
        }
    });

    task1.await.unwrap();
    task2.await.unwrap();
    manager.await.unwrap();
}
