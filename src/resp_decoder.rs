use crate::resp::{Type, RESP};
use anyhow::{anyhow, Result};
use std::io::{self, BufRead, Read};

pub struct RESPDecoder {
    cursor: io::Cursor<Vec<u8>>,
}

impl RESPDecoder {
    pub fn new(buffer: Vec<u8>) -> Self {
        RESPDecoder {
            cursor: io::Cursor::new(buffer),
        }
    }

    pub fn next_resp(&mut self) -> Result<RESP> {
        let data_type = self.read_data_type()?;
        let result = match data_type {
            Type::SimpleString => self.decode_simple_string()?,
            Type::BulkString => self.decode_bulk_string()?,
            Type::Integer => self.decode_integer()?,
            Type::Array => self.decode_array()?,
            Type::Error => self.decode_error()?,
        };
        Ok(result)
    }

    fn decode_simple_string(&mut self) -> Result<RESP> {
        let bytes = self.read_until_crlf()?;
        Ok(RESP::new(Type::SimpleString, bytes, vec![]))
    }

    fn decode_bulk_string(&mut self) -> Result<RESP> {
        let bytes = self.read_until_crlf()?;
        let string_count = String::from_utf8_lossy(&bytes[..])
            .to_owned()
            .parse::<usize>()?;

        let mut string_bytes: Vec<u8> = vec![0; string_count + 2];
        self.cursor.read(&mut string_bytes[..])?;
        Ok(RESP::new(
            Type::BulkString,
            string_bytes[..string_bytes.len() - 2].to_vec(),
            vec![],
        ))
    }

    fn decode_integer(&mut self) -> Result<RESP> {
        let bytes = self.read_until_crlf()?;
        Ok(RESP::new(Type::Integer, bytes, vec![]))
    }

    fn decode_error(&mut self) -> Result<RESP> {
        let bytes = self.read_until_crlf()?;
        Ok(RESP::new(Type::Error, bytes, vec![]))
    }

    fn decode_array(&mut self) -> Result<RESP> {
        let bytes = self.read_until_crlf()?;
        let array_size = String::from_utf8_lossy(&bytes[..])
            .to_owned()
            .parse::<usize>()?;

        let mut children: Vec<RESP> = vec![];
        for _ in 0..array_size {
            let child_resp = self.next_resp()?;
            children.push(child_resp);
        }

        Ok(RESP::new(Type::Array, bytes, children))
    }

    fn read_data_type(&mut self) -> Result<Type> {
        let mut buffer = [0; 1];
        self.cursor.read(&mut buffer)?;
        let data_type_char = char::from(buffer[0]);
        match Type::try_from(data_type_char) {
            Ok(data_type) => Ok(data_type),
            Err(e) => Err(anyhow!(e)),
        }
    }

    pub fn read_until_crlf(&mut self) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = vec![];
        loop {
            let num_bytes = self.cursor.read_until(b'\n', &mut bytes)?;
            if num_bytes > 2 && bytes[num_bytes - 2] == b'\r' {
                break;
            }
        }

        Ok(bytes[..bytes.len() - 2].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::{RESPDecoder, Type};

    #[test]
    fn it_should_decode_simple_string() {
        let simple_string = b"+OK\r\n";
        let resp = RESPDecoder::new(simple_string.to_vec())
            .next_resp()
            .unwrap();
        assert_eq!(resp.string(), "OK");
    }

    #[test]
    fn it_should_decode_bulk_string() {
        let bulk_string = b"$5\r\nhello\r\n";
        let resp = RESPDecoder::new(bulk_string.to_vec()).next_resp().unwrap();
        assert_eq!(resp.string(), "hello");
    }

    #[test]
    fn it_should_decode_integer_string() {
        let integer = b":1000\r\n";
        let resp = RESPDecoder::new(integer.to_vec()).next_resp().unwrap();
        assert_eq!(resp.integer(), 1000);
    }

    #[test]
    fn it_should_decode_error_string() {
        let error = b"-ERR unknown command 'helloworld'\r\n";
        let resp = RESPDecoder::new(error.to_vec()).next_resp().unwrap();
        assert_eq!(resp.string(), "ERR unknown command 'helloworld'");
    }

    #[test]
    fn it_should_decode_array_string() {
        let error = b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n";
        let resp = RESPDecoder::new(error.to_vec()).next_resp().unwrap();
        let children = resp.array();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].string(), "hello");
        assert_eq!(children[1].string(), "world");
    }

    #[test]
    fn it_should_decode_nested_array_string() {
        let error = b"*2\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n+Hello\r\n-World\r\n";

        let resp = RESPDecoder::new(error.to_vec()).next_resp().unwrap();
        assert_eq!(resp.value_type, Type::Array);

        let children = resp.array();
        assert_eq!(children.len(), 2);

        let first_child = &children[0];
        let second_child = &children[1];
        assert_eq!(first_child.value_type, Type::Array);
        assert_eq!(second_child.value_type, Type::Array);
        let first_child_array = first_child.array();
        let second_child_array = second_child.array();
        assert_eq!(first_child_array[0].integer(), 1);
        assert_eq!(first_child_array[1].integer(), 2);
        assert_eq!(first_child_array[2].integer(), 3);
        assert_eq!(second_child_array[0].string(), "Hello");
        assert_eq!(second_child_array[1].string(), "World");
    }
}
