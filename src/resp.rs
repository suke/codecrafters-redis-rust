use std::convert::TryFrom;

pub const SIMPLE_STRING: char = '+';
pub const ERROR: char = '-';
pub const INTEGER: char = ':';
pub const BULK_STRING: char = '$';
pub const ARRAY: char = '*';

pub const CRLF: &'static str = "\r\n";
pub const NULL_STRING: &'static str = "$-1\r\n";

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Type {
    SimpleString,
    Error,
    Integer,
    BulkString,
    Array,
}

impl TryFrom<char> for Type {
    type Error = String;

    fn try_from(data_type_char: char) -> Result<Self, String> {
        match data_type_char {
            SIMPLE_STRING => Ok(Type::SimpleString),
            ERROR => Ok(Type::Error),
            INTEGER => Ok(Type::Integer),
            BULK_STRING => Ok(Type::BulkString),
            ARRAY => Ok(Type::Array),
            _ => Err(format!("invalid data type byte: {}", data_type_char)),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str = match self {
            Type::SimpleString => SIMPLE_STRING,
            Type::Error => ERROR,
            Type::Integer => INTEGER,
            Type::BulkString => BULK_STRING,
            Type::Array => ARRAY,
        };

        write!(f, "{}", str)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RESPError {
    pub error_type: String,
    pub message: String,
}

impl RESPError {
    pub fn new(error_type: String, message: String) -> Self {
        RESPError {
            error_type,
            message,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RESP {
    pub value_type: Type,
    data: Vec<u8>,
    children: Vec<RESP>,
}

impl RESP {
    pub fn new(value_type: Type, data: Vec<u8>, children: Vec<RESP>) -> Self {
        RESP {
            value_type,
            data,
            children,
        }
    }

    pub fn string(&self) -> String {
        String::from_utf8_lossy(&self.data).into_owned()
    }

    pub fn integer(&self) -> i64 {
        String::from_utf8_lossy(&self.data)
            .to_owned()
            .parse::<i64>()
            .unwrap()
    }

    pub fn array(&self) -> &Vec<RESP> {
        &self.children
    }
}
