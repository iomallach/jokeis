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
        match command {
            Command::Ping(s) => {
                conn.write_message(Value::BulkString(s.msg)).await?;
            }
            Command::Echo(s) => {
                conn.write_message(Value::BulkString(s.msg)).await?;
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
                    let _ = process_socket(connection).await;
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection {e}");
            }
        }
    }
}
