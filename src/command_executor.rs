use crate::resp::{BULK_STRING, CRLF, ERROR, INTEGER, RESP, SIMPLE_STRING};
use crate::store::Store;
use std::sync::RwLock;

#[derive(Clone)]
pub struct CommandExecutor<'a> {
    args: &'a Vec<RESP>,
    store: &'a RwLock<Store>,
}

impl<'a> CommandExecutor<'a> {
    pub fn new(args: &'a Vec<RESP>, store: &'a RwLock<Store>) -> Self {
        CommandExecutor { args, store }
    }

    pub fn execute(&self) -> Vec<u8> {
        let mut response_bytes: Vec<u8> = vec![];
        if let Some(command) = self.command() {
            match &*command {
                "ping" => append_simple_string(&mut response_bytes, "PONG".to_string()),
                "echo" => self.execute_echo_command(&mut response_bytes),
                "set" => self.execute_set_command(&mut response_bytes),
                "get" => self.execute_get_command(&mut response_bytes),
                // TODO: implement the remaining commands
                _ => append_error(
                    &mut response_bytes,
                    "ERR".to_string(),
                    "unsupported command".to_string(),
                ),
            }
        } else {
            append_error(
                &mut response_bytes,
                "ERR".to_string(),
                "command is not provided".to_string(),
            );
        }

        response_bytes
    }

    fn command(&self) -> Option<String> {
        if self.args.len() == 0 {
            return None;
        }

        Some(self.args[0].string().to_lowercase())
    }

    fn execute_echo_command(&self, mut bytes: &mut Vec<u8>) {
        if self.args.len() <= 1 {
            append_error(
                &mut bytes,
                "ERR".to_string(),
                "wrong number of arguments for 'echo' command".to_string(),
            );
            return;
        }

        let value = self.args[1].string();
        append_bulk_string(&mut bytes, value)
    }

    fn execute_set_command(&self, mut bytes: &mut Vec<u8>) {
        if self.args.len() <= 2 {
            append_error(
                &mut bytes,
                "ERR".to_string(),
                "wrong number of arguments for 'set' command".to_string(),
            );
            return;
        }

        let key = self.args[1].string();
        let value = self.args[2].string();

        match self.store.write() {
            Ok(mut store) => {
                store.set(key, value);
                append_simple_string(&mut bytes, "OK".to_string());
            }
            Err(_) => {
                append_error(
                    &mut bytes,
                    "ERR".to_string(),
                    "internal server error occurred".to_string(),
                );
            }
        }
    }

    fn execute_get_command(&self, mut bytes: &mut Vec<u8>) {
        if self.args.len() <= 1 {
            append_error(
                &mut bytes,
                "ERR".to_string(),
                "wrong number of arguments for 'get' command".to_string(),
            );
            return;
        }

        let key = self.args[1].string();
        match self.store.read() {
            Ok(store) => match store.get(key) {
                Some(value) => append_bulk_string(&mut bytes, value),
                None => append_null_string(&mut bytes),
            },
            Err(_) => {
                append_error(
                    &mut bytes,
                    "ERR".to_string(),
                    "internal server error occurred".to_string(),
                );
            }
        }
    }
}

fn append_simple_string(bytes: &mut Vec<u8>, value: String) {
    let mut data = [SIMPLE_STRING.to_string(), value, CRLF.to_string()]
        .join("")
        .as_bytes()
        .to_vec();
    bytes.append(&mut data);
}

fn append_bulk_string(bytes: &mut Vec<u8>, value: String) {
    let string_bytes = value.as_bytes();
    let mut data = [
        BULK_STRING.to_string(),
        string_bytes.len().to_string(),
        CRLF.to_string(),
        value,
        CRLF.to_string(),
    ]
    .join("")
    .as_bytes()
    .to_vec();
    bytes.append(&mut data);
}

fn append_null_string(bytes: &mut Vec<u8>) {
    let mut data = [BULK_STRING.to_string(), "-1".to_string(), CRLF.to_string()]
        .join("")
        .as_bytes()
        .to_vec();
    bytes.append(&mut data);
}

#[allow(dead_code)]
fn append_integer(bytes: &mut Vec<u8>, int: i64) {
    let mut data = [INTEGER.to_string(), int.to_string(), CRLF.to_string()]
        .join("")
        .as_bytes()
        .to_vec();
    bytes.append(&mut data);
}

fn append_error(bytes: &mut Vec<u8>, error_type: String, message: String) {
    let mut data = [
        ERROR.to_string(),
        error_type,
        " ".to_string(),
        message,
        CRLF.to_string(),
    ]
    .join("")
    .as_bytes()
    .to_vec();
    bytes.append(&mut data);
}

#[cfg(test)]
mod tests {
    use super::CommandExecutor;
    use crate::resp_decoder::RESPDecoder;
    use crate::store::Store;
    use std::sync::RwLock;

    #[test]
    fn it_should_execute_ping() {
        let store = RwLock::new(Store::new());
        let command = b"*1\r\n$4\r\nping\r\n";
        let response = execute_command(command.to_vec(), &store);
        assert_eq!(String::from_utf8_lossy(&response[..]), "+PONG\r\n");
    }

    #[test]
    fn it_should_execute_echo() {
        let store = RwLock::new(Store::new());
        let command = b"*2\r\n$4\r\necho\r\n$5\r\nhello\r\n";
        let response = execute_command(command.to_vec(), &store);
        assert_eq!(String::from_utf8_lossy(&response[..]), "$5\r\nhello\r\n");
    }

    #[test]
    fn it_should_execute_set() {
        let store = RwLock::new(Store::new());
        let command = b"*3\r\n$3\r\nset\r\n$3\r\nkey\r\n$5\r\nvalue\r\n";
        let response = execute_command(command.to_vec(), &store);
        assert_eq!(String::from_utf8_lossy(&response[..]), "+OK\r\n");
    }

    #[test]
    fn it_should_execute_get() {
        let store = RwLock::new(Store::new());
        let set_command = b"*3\r\n$3\r\nset\r\n$3\r\nkey\r\n$5\r\nvalue\r\n";
        execute_command(set_command.to_vec(), &store);

        let get_command = b"*2\r\n$3\r\nget\r\n$3\r\nkey\r\n";
        let response = execute_command(get_command.to_vec(), &store);
        assert_eq!(String::from_utf8_lossy(&response[..]), "$5\r\nvalue\r\n");
    }

    #[test]
    fn it_should_return_an_argument_error() {
        let tests = [
            ("echo", "*1\r\n$4\r\necho\r\n"),
            ("set", "*2\r\n$3\r\nset\r\n$3\r\nkey\r\n"),
            ("get", "*1\r\n$3\r\nget\r\n"),
        ];

        for test in tests {
            let store = RwLock::new(Store::new());
            let response = execute_command(test.1.as_bytes().to_vec(), &store);
            assert_eq!(
                String::from_utf8_lossy(&response[..]),
                format!(
                    "-ERR wrong number of arguments for '{}' command\r\n",
                    test.0
                )
            );
        }
    }

    fn execute_command(command: Vec<u8>, store: &RwLock<Store>) -> Vec<u8> {
        let mut decorder = RESPDecoder::new(command);
        let resp = decorder.next_resp().unwrap();
        let args = resp.array();
        CommandExecutor::new(args, store).execute()
    }
}
