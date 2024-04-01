// Uncomment this block to pass the first stage
use anyhow::Result;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};

use bytes::Bytes;


// TODO: ignore at the moment as we don't know the protocol yet
enum Command {
    Ping(Option<Bytes>),
}

async fn process_socket(mut stream: TcpStream) {
    let mut buf = [0u8; 64];

    loop {
    match stream.read(&mut buf).await {
        Ok(0) => {
            break;
        },
        Ok(_) => {
            stream.write_all("+PONG\r\n".as_bytes()).await.expect("Connection died");
        },
        Err(_) => {
            break;
        },
    }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let (stream, sock_addr) = listener.accept().await?;
        println!("New client at {}", sock_addr);

        tokio::spawn(async move {
            process_socket(stream).await;
        });
    }
}
