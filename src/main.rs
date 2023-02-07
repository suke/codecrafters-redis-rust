use anyhow::anyhow;
use anyhow::Result;
use redis_starter_rust::command_executor::CommandExecutor;
use redis_starter_rust::resp_decoder::RESPDecoder;
use redis_starter_rust::store::Store;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let store = Arc::new(RwLock::new(Store::new()));
    for stream in listener.incoming() {
        // TODO: using thread pool
        let stream = stream?;
        let store = Arc::clone(&store);
        thread::spawn(move || {
            handle_request(stream, store).unwrap_or_else(|error| eprintln!("{:?}", error));
        });
    }

    Ok(())
}

fn handle_request(mut stream: TcpStream, store: Arc<RwLock<Store>>) -> Result<()> {
    loop {
        let mut buffer = [0; 1024];
        let byte_count = stream
            .read(&mut buffer)
            .expect("error while reading from connection");

        if byte_count == 0 {
            break;
        }

        let mut decoder = RESPDecoder::new(buffer[..byte_count].to_vec());
        let resp = decoder.next_resp()?;
        let args = resp.array();

        if args.len() == 0 {
            continue;
        }

        let command_executer = CommandExecutor::new(args, &store);
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
