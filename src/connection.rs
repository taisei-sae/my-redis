use std::io::Cursor;

use bytes::{Buf, BytesMut};
use mini_redis::{Frame, Result};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

pub struct Connection {
    // Not using TcpStream directly,
    // in order to avoid calling write syscalls for each part of frame.
    // The cost of syscall is not negligible.
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
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

            // If there is not enough data in buffer,
            // reads more data from stream and write it into buffer.
            //
            // The `BufMut` has an internal cursor.
            // We don't need to worry about if the buffer would be overwrited.
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                // The connection has been closed
                if self.buffer.is_empty() {
                    // Received no data from stream
                    return Ok(None);
                } else {
                    // A partial frame has been received but the connection has been closed,
                    // this means the connection was closed abruptly.
                    return Err("connection reset by peer ".into());
                }
            }
        }
    }

    fn parse_frame(&mut self) -> Result<Option<Frame>> {
        // Cursor position is 0 even if the buffer contains data
        let mut cursor = Cursor::new(&self.buffer[..]);

        // This `check` advances the cursor position
        match Frame::check(&mut cursor) {
            Ok(_) => {
                // Get the length of the data
                let len = cursor.position() as usize;

                // Reset the cursor position
                // to call `parse`
                cursor.set_position(0);

                // This `parse` uses cursor
                // to read the data from start to end
                let frame = Frame::parse(&mut cursor)?;

                // Advance the internal cursor to discard this frame
                self.buffer.advance(len);

                Ok(Some(frame))
            }
            Err(mini_redis::frame::Error::Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn write_frame(self, frame: &Frame) -> io::Result<()> {
        match frame {
            Frame::Simple(val) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Error(val) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Integer(val) => {
                self.stream.write_u8(b':').await?;
                self.write_decimal(*val).await?;
            }
            Frame::Null => {
                self.stream.write_all(b"$-1\r\n").await?;
            }
            Frame::Bulk(val) => {
                let len = val.len();

                self.stream.write_u8(b'$').await?;
                self.write_decimal(len as u64).await?;
                self.stream.write_all(val).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Array(_) => {
                unimplemented!()
            }
        }

        // The actual `write` syscall
        //
        // Another alternative would be providing `flush` on `Connection`,
        // so that `Connection` can flush multiple frames at the same time.
        // But this time we prioritize simplicity.
        self.stream.flush().await;

        Ok(())
    }
}
