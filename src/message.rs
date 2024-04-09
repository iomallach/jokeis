use anyhow::Result;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::{fmt::Write, io::Cursor, vec::IntoIter};
use thiserror::Error;

#[derive(Error, Debug)]
enum ParserError {
    #[error("End of message")]
    EndOfMessage,

    #[error("Unexpected value: {0}")]
    UnexpectedValue(anyhow::Error),
}

struct CommandParser {
    inner: IntoIter<Value>,
}

impl CommandParser {
    fn new(v: Value) -> Result<Self> {
        match v {
            Value::Array(a) => Ok(Self {
                inner: a.into_iter(),
            }),
            _ => Err(anyhow::anyhow!("Expected array")),
        }
    }

    fn next_string(&mut self) -> Result<Bytes> {
        match self.inner.next().ok_or(ParserError::EndOfMessage)? {
            Value::BulkString(s) => Ok(s),
            msg => Err(anyhow::anyhow!("Expected string, got {:?}", msg)),
        }
    }
}

pub enum Command {
    Ping(Ping),
    Echo(Echo),
}

struct Ping {
    msg: Bytes,
}

impl Ping {
    fn parse(mut parser: CommandParser) -> Result<Self> {
        match parser.next_string() {
            Ok(s) => Ok(Ping { msg: s }),
            Err(e) => match e.downcast_ref::<ParserError>() {
                Some(ParserError::EndOfMessage) => Ok(Ping { msg: "PONG".into() }),
                _ => Err(e),
            },
        }
    }
}
struct Echo {
    msg: Bytes,
}

impl Echo {
    fn parse(mut parser: CommandParser) -> Result<Self> {
        match parser.next_string() {
            Ok(s) => Ok(Echo { msg: s }),
            Err(e) => match e.downcast_ref::<ParserError>() {
                Some(ParserError::EndOfMessage) => {
                    Err(anyhow::anyhow!("Expected a message, found end of message"))
                }
                _ => Err(e),
            },
        }
    }
}

impl Command {
    pub fn from_value(v: Value) -> Result<Self> {
        let mut parser = CommandParser::new(v)?;
        let cmd_name = std::str::from_utf8(&parser.next_string()?)?.to_lowercase();

        match cmd_name.as_str() {
            "ping" => Ok(Command::Ping(Ping::parse(parser)?)),
            "echo" => Ok(Command::Echo(Echo::parse(parser)?)),
        }
    }
}

#[derive(Debug)]
pub enum Value {
    BulkString(Bytes),
    Array(Vec<Value>),
}

impl Value {
    fn array() -> Self {
        Self::Array(Vec::new())
    }

    fn push_bulk_str(&mut self, bstr: Value) {
        match self {
            Self::Array(a) => a.push(bstr),
            _ => panic!("Not an array"),
        }
    }

    fn as_bytes(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(4 * 1024);
        match self {
            Self::BulkString(s) => {
                write!(buf, "${}\r\n", s.len());
                buf.extend_from_slice(s);
                buf.put(&b"\r\n"[..]);
            }
            _ => unimplemented!("Not implemented"),
        }
        buf.freeze()
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::BulkString(s1), Self::BulkString(s2)) => s1 == s2,
            (Self::Array(v1), Self::Array(v2)) => v1.iter().zip(v2.iter()).all(|(e1, e2)| e1 == e2),
            _ => false,
        }
    }
}

pub fn parse_message<'a>(cur: &mut Cursor<&'a [u8]>) -> Result<Value> {
    // TODO: see docs, it may panic if there are no remaining bytes in the buffer
    match cur.get_u8() {
        b'*' => {
            let length = parse_number(cur)?;
            let mut msg = Value::array();
            for _ in 0..length {
                msg.push_bulk_str(parse_message(cur)?);
            }
            Ok(msg)
        }
        b'$' => {
            let length = parse_number(cur)?;
            if length == 0 {
                cur.advance(2);
                Ok(Value::BulkString("".into()))
            } else {
                parse_bulk_str(cur, length)
            }
        }
        _ => unimplemented!("Not implemented"),
    }
}

fn parse_number<'a>(cur: &mut Cursor<&'a [u8]>) -> Result<usize> {
    Ok(String::from_utf8(chop_until(cur, |x| x == '\r')?.into())?.parse::<usize>()?)
}

fn chop_until<'a, F>(cur: &mut Cursor<&'a [u8]>, p: F) -> Result<&'a [u8]>
where
    F: Fn(char) -> bool,
{
    let start = cur.position() as usize;
    let end = cur.get_ref().len() - 1;

    for i in start..end {
        if p(cur.get_ref()[i] as char) {
            cur.set_position(i as u64 + 2);
            return Ok(&cur.get_ref()[start..i]);
        }
    }
    Err(anyhow::anyhow!("Caret return not found"))
}

fn parse_bulk_str<'a>(cur: &mut Cursor<&'a [u8]>, length: usize) -> Result<Value> {
    let start = cur.position() as usize;
    let bstr = &cur.get_ref()[start..start + length as usize];
    cur.set_position(start as u64 + length as u64 + 2);

    Ok(Value::BulkString(bstr.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let mut cursor = std::io::Cursor::new("123\r\n".as_bytes());
        match parse_number(&mut cursor) {
            Ok(num) => {
                if num != 123 {
                    panic!("Incorrect number parsed")
                }
            }
            Err(e) => panic!("Parsing failed {e}"),
        }
    }

    #[test]
    fn test_parse_bulk_string() {
        let mut cursor = std::io::Cursor::new("test_string\r\n".as_bytes());
        let length = cursor.get_ref().len() - 2;
        match parse_bulk_str(&mut cursor, length) {
            Ok(s) => match s {
                Value::BulkString(s) => {
                    if s != "test_string".to_owned() {
                        panic!("Strings are not equal")
                    }
                }
                _ => panic!("Unexpected value"),
            },
            Err(e) => panic!("Parsing failed {e}"),
        }
    }

    #[test]
    fn test_parse_message() {
        let mut cursor = std::io::Cursor::new("*2\r\n$4\r\nllen\r\n$6\r\nmylist\r\n".as_bytes());
        let parsed = parse_message(&mut cursor);
        match parsed {
            Ok(m) => {
                if m != Value::Array(vec![
                    Value::BulkString("llen".into()),
                    Value::BulkString("mylist".into()),
                ]) {
                    panic!("Unexpected message");
                }
            }
            Err(e) => panic!("Parsing failed {e}"),
        }
    }
}
