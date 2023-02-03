use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

fn reply_pong(mut stream: TcpStream) {
    let response = "+PONG\r\n";
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                reply_pong(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
