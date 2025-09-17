use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

pub mod connection;
pub mod connection_raw;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let mut f = File::open("foo.txt").await?;
    let mut buf = [0; 10]; // compiler infers this type as u8

    // read() reads the file source and copy into buf
    let _ = f.read(&mut buf).await?;

    println!("The bytes: {:?}", &buf);

    let mut f = File::create("foo.txt").await.unwrap();

    // write the bytes into file

    let n = f.write(b"WATAWATA").await.unwrap();

    println!("Wrote the first {} bytes of 'WATAWATA'.", n);
    Ok(())
}
