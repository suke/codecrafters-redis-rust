use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_request(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_request(mut stream: TcpStream) {
    loop {
        let mut buffer = [0; 1028];
        stream
            .read(&mut buffer)
            .expect("error while reading from connection");

        if buffer.len() == 0 {
            break;
        }

        let response = "+PONG\r\n";
        stream.write(response.as_bytes()).unwrap();
        stream.flush().expect("error while writing to connection");
    }
}
