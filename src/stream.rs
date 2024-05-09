use crate::error::InputError;
use std::io;

#[derive(Debug)]
pub enum Fetch {
    Byte(u8),
    Error(io::Error),
    Ended,
}

pub trait RioStream {
    fn getch(&mut self) -> Fetch;
}

impl<T> RioStream for T
where
    T: Iterator<Item = io::Result<u8>>,
{
    fn getch(&mut self) -> Fetch {
        match self.next() {
            None => Fetch::Ended,
            Some(Err(err)) => Fetch::Error(err),
            Some(Ok(byte)) => Fetch::Byte(byte),
        }
    }
}
