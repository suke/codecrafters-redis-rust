use anyhow::anyhow;
use anyhow::Result;
use redis_starter_rust::command_executer::CommandExecuter;
use redis_starter_rust::resp_decoder::RESPDecoder;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    for stream in listener.incoming() {
        // TODO: using thread pool
        let stream = stream?;
        thread::spawn(|| {
            handle_request(stream).unwrap_or_else(|error| eprintln!("{:?}", error));
        });
    }

    Ok(())
}

fn handle_request(mut stream: TcpStream) -> Result<()> {
    loop {
        let mut buffer = [0; 1024];
        let byte_count = stream
            .read(&mut buffer)
            .expect("error while reading from connection");

        if byte_count == 0 {
            break;
        }

        let mut decoder = RESPDecoder::new(buffer.to_vec());
        let resp = decoder.next_resp()?;
        let args = resp.array();

        if args.len() == 0 {
            continue;
        }

        let command_executer = CommandExecuter::new(args);
        let response = command_executer.execute();
        match stream.write(&response[..]) {
            Ok(_) => {
                stream.flush().expect("error while writing to connection");
            }
            Err(error) => match error.kind() {
                ErrorKind::BrokenPipe => {
                    break;
                }
                other_error => return Err(anyhow!("{}", other_error)),
            },
        }
    }

    Ok(())
}
