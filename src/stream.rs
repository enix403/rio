use std::io::{self, Bytes, Read};

#[derive(Debug)]
pub enum Fetch {
    Byte(u8),
    Error(io::Error),
    Ended,
}

pub struct RioStream<R> {
    bytes: Bytes<R>,
}

impl<R: Read> RioStream<R> {
    pub fn new(reader: R) -> Self {
        Self {
            bytes: reader.bytes()
        }
    }
    
    pub fn getch(&mut self) -> Fetch {
        match self.bytes.next() {
            None => Fetch::Ended,
            Some(Err(err)) => Fetch::Error(err),
            Some(Ok(byte)) => Fetch::Byte(byte),
        }
    }
}
