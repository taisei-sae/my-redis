use bytes::BytesMut;
use mini_redis::{Frame, Result};
use tokio::{io::AsyncReadExt, net::TcpStream};

pub struct Connection {
    stream: TcpStream,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: stream,
            // 4KB buffer
            buffer: BytesMut::with_capacity(4096),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            // Attempt to parse a frame from buffer
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            // If there is not enough data in buffer
            // Read more data from stream and write it into buffer
            //
            // The `BufMut` has an internal cursor.
            // We don't need to worry about if the buffer would be rewrited.
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                // End of connection
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    // A partial frame has been received,
                    // and the connection is being terminated abruptly
                    return Err("connection reset by peer ".into());
                }
            }
        }
    }
}
