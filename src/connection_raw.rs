use mini_redis::{Frame, Result};
use tokio::{io::AsyncReadExt, net::TcpStream};

pub struct Connection {
    stream: TcpStream,
    buffer: Vec<u8>,
    cursor: usize,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: stream,
            buffer: vec![0; 4096],
            cursor: 0,
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            // Does buffer have enough capacity?
            if self.buffer.len() == self.cursor {
                // Grow the buffer
                self.buffer.resize(self.cursor * 2, 0);
            }

            // read from stream and write into buffer from the position of cursor.
            // Don't forget to pass the empty portion of the buffer
            // because `read` overwrites the buffer.
            let n = self.stream.read(&mut self.buffer[self.cursor..]).await?;

            // Is the connection closed?
            if n == 0 {
                // Received no data from stream
                // and closed connection gracefully
                if self.cursor == 0 {
                    return Ok(None);
                } else {
                    // Received some incomplete data but the connection has been closed,
                    // this means the connection was closed abruptly.
                    return Err("connection reset by peer".into());
                }
            } else {
                self.cursor += n;
            }
        }
    }
}
