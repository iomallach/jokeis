use anyhow::Result;
use bytes::{Buf, BytesMut};
use std::io::Write;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufWriter};

use crate::message;

pub struct Connection<W: AsyncWrite + AsyncRead> {
    buf_writer: BufWriter<W>,
    buffer: BytesMut,
}

impl<W: AsyncWrite + AsyncRead + Unpin> Connection<W> {
    pub fn new(stream: W) -> Self {
        Self {
            buf_writer: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(1024 * 512), // 512KB
        }
    }

    pub async fn read_message(&mut self) -> Result<message::Value> {
        self.buf_writer.read_buf(&mut self.buffer).await?;
        let mut cursor = std::io::Cursor::new(&self.buffer[..]);

        let message = message::parse_message(&mut cursor);
        self.buffer.advance(cursor.position() as usize);
        message
    }

    // TODO: dumbest write_message is down below. The missing piece is casting
    // a command to Value
    pub async fn write_message(&mut self, msg: message::Value) -> Result<()> {
        match msg {
            message::Value::BulkString(s) => {
                self.buf_writer.write_all(b"$").await?;
                let mut buf = [0u8; 64];
                let mut buf = std::io::Cursor::new(&mut buf[..]);
                write!(&mut buf, "{}", s.len());

                self.buf_writer
                    .write_all(&buf.get_ref()[..buf.position() as usize])
                    .await?;
                self.buf_writer.write_all(b"\r\n").await?;
                self.buf_writer.write_all(&s[..]).await?;
                self.buf_writer.write_all(b"\r\n").await?;
                self.buf_writer.flush().await?;
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Expected array message")),
        }
    }
}
