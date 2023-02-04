use std::io::prelude::*;
use std::io::ErrorKind;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    for stream in listener.incoming() {
        // TODO: using thread pool
        thread::spawn(|| match stream {
            Ok(stream) => {
                handle_request(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        });
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
        match stream.write(response.as_bytes()) {
            Ok(_) => {
                stream.flush().expect("error while writing to connection");
            }
            Err(error) => match error.kind() {
                ErrorKind::BrokenPipe => {
                    break;
                }
                other_error => {
                    panic!("error: {:?}", other_error);
                }
            },
        }
    }
}
