// Uncomment this block to pass the first stage
use anyhow::Result;
use std::{io::Write, net::TcpListener};

use bytes::Bytes;

// TODO: ignore at the moment as we don't know the protocol yet
enum Command {
    Ping(Option<Bytes>),
}

fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                stream.write_all("+PING\r\n".as_bytes())?;
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}
