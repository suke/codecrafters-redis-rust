use crate::resp::{BULK_STRING, CRLF, INTEGER, RESP, SIMPLE_STRING};

#[derive(Debug, Clone, PartialEq)]
pub struct CommandExecuter<'a> {
    args: &'a Vec<RESP>,
}

impl<'a> CommandExecuter<'a> {
    pub fn new(args: &'a Vec<RESP>) -> Self {
        CommandExecuter { args }
    }

    pub fn execute(&self) -> Vec<u8> {
        let mut response_bytes: Vec<u8> = vec![];
        match &*self.command() {
            "ping" => append_pong(&mut response_bytes),
            "echo" => append_bulk_string(&mut response_bytes, self.value()),
            // TODO: implement the remaining commands
            _ => append_integer(&mut response_bytes, 0),
        }

        response_bytes
    }

    fn command(&self) -> String {
        if self.args.len() == 0 {
            return "".to_owned();
        }

        self.args[0].string().to_lowercase()
    }

    fn value(&self) -> String {
        if self.args.len() <= 1 {
            return "".to_owned();
        }

        self.args[1].string()
    }
}

fn append_pong(bytes: &mut Vec<u8>) {
    let mut data = [
        SIMPLE_STRING.to_string(),
        "PONG".to_string(),
        CRLF.to_string(),
    ]
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

fn append_integer(bytes: &mut Vec<u8>, int: i64) {
    let mut data = [INTEGER.to_string(), int.to_string(), CRLF.to_string()]
        .join("")
        .as_bytes()
        .to_vec();
    bytes.append(&mut data);
}

#[cfg(test)]
mod tests {
    use super::CommandExecuter;
    use crate::resp_decoder::RESPDecoder;

    #[test]
    fn it_should_execute_ping() {
        let command = "*1\r\n$4\r\nping\r\n";
        let mut decorder = RESPDecoder::new(command.as_bytes().to_vec());
        let resp = decorder.next_resp().unwrap();
        let args = resp.array();

        let response = CommandExecuter::new(args).execute();
        assert_eq!(String::from_utf8_lossy(&response[..]), "+PONG\r\n");
    }

    #[test]
    fn it_should_execute_echo() {
        let command = "*2\r\n$4\r\necho\r\n$5\r\nhello\r\n";
        let mut decorder = RESPDecoder::new(command.as_bytes().to_vec());
        let resp = decorder.next_resp().unwrap();
        let args = resp.array();

        let response = CommandExecuter::new(args).execute();
        assert_eq!(String::from_utf8_lossy(&response[..]), "$5\r\nhello\r\n");
    }
}
