use mini_redis::{Result, client};

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut client = client::connect("127.0.0.1:6379").await?;
    client.set("key", "value".into()).await?;
    let result = client.get("key").await?;

    println!("got value from the server; result={:?}", result);
    Ok(())
}
