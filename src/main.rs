// Uncomment this block to pass the first stage
use anyhow::Result;
use message::{Command, Value};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

mod connection;
mod message;

async fn process_socket(mut conn: connection::Connection<TcpStream>) -> Result<()> {
    loop {
        let command = Command::from_value(conn.read_message().await?)?;
    }
    let mut buf = [0u8; 64];
    loop {
        match stream.read(&mut buf).await {
            Ok(0) => {
                break;
            }
            Ok(_) => {
                stream
                    .write_all("+PONG\r\n".as_bytes())
                    .await
                    .expect("Connection died");
            }
            Err(_) => {
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        match listener.accept().await {
            Ok((stream, sock_addr)) => {
                println!("New client at {}", sock_addr);
                let connection = connection::Connection::new(stream);

                tokio::spawn(async move {
                    process_socket(stream).await;
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection {e}");
            }
        }
    }
}
