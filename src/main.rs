// Uncomment this block to pass the first stage
use anyhow::Result;
use std::{
    io::{Read, Write},
    net::TcpListener,
};

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
                let mut buf = [0u8; 64];
                loop {
                    match stream.read(&mut buf) {
                        Ok(0) => {}
                        Ok(_) => {
                            stream.write_all("+PONG\r\n".as_bytes())?;
                        }
                        Err(_) => {}
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}
