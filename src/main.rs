#![allow(unused)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use num_traits::*;
use std::io::{self, Read};

enum InputError {
    IO(io::Error),
    Extraction,
}

struct Rio<R> {
    reader: R,
    ungetted: Option<u8>,
    ended: bool,
    error: Option<InputError>,
}

trait Extract<T> {
    fn extract(&mut self) -> Option<T>;
}

struct Extractor<'a, R> {
    rio: &'a mut Rio<R>,
}

impl<R> Rio<R>
where
    R: Iterator<Item = io::Result<u8>>,
{
    fn new(source: R) -> Self {
        Self {
            reader: source,
            ungetted: None,
            ended: false,
            error: None,
        }
    }

    fn getch(&mut self) -> Option<u8> {
        if self.error.is_some() || self.ended {
            return None;
        }

        match self.ungetted {
            Some(_) => self.ungetted.take(),
            _ => {
                let item = self.reader.next();

                if let Some(inner) = item {
                    match inner {
                        Ok(byte) => Some(byte),
                        Err(err) => {
                            self.error = Some(InputError::IO(err));
                            None
                        }
                    }
                } else {
                    // EOF has been reached
                    self.ended = true;
                    None
                }
            }
        }
    }

    fn ungetch(&mut self, byte: u8) {
        self.ungetted = Some(byte);
    }

    fn clear(&mut self) -> Option<InputError> {
        self.error.take()
    }

    fn read<T>(&mut self) -> Option<T>
    where
        for<'a> Extractor<'a, R>: Extract<T>,
    {
        // Skip whitespace
        while let Some(current) = self.getch() {
            if !current.is_ascii_whitespace() {
                self.ungetch(current);
                break;
            }
        }

        let result = Extractor::new(self).extract();

        if result.is_none() {
            self.error = Some(InputError::Extraction);
        }

        result
    }
}

impl<'a, R> Extractor<'a, R> {
    pub fn new(rio: &'a mut Rio<R>) -> Self {
        Self { rio }
    }

    fn to_digit(val: u8) -> Option<u8> {
        match val {
            b'0'..=b'9' => Some((val - b'0')),
            _ => None,
        }
    }

    fn small_value_to_num<T: FromPrimitive>(val: u8) -> T {
        unsafe { T::from_u8(val).unwrap_unchecked() }
    }
}

trait ExtractableNum
where
    Self: Zero + FromPrimitive + CheckedAdd<Output = Self> + CheckedMul<Output = Self>,
{
}

impl ExtractableNum for i32 {}
impl ExtractableNum for u32 {}

impl<'a, T, R> Extract<T> for Extractor<'a, R>
where
    T: ExtractableNum,
    R: Iterator<Item = io::Result<u8>>,
{
    fn extract(&mut self) -> Option<T> {
        let mut val: T = T::zero();
        let ten = Self::small_value_to_num(10);

        let mut started = false;

        while let Some(current) = self.rio.getch() {
            let as_digit = Self::to_digit(current);

            let updated = as_digit.and_then(|x| {
                let x = Self::small_value_to_num(x);
                val.checked_mul(&ten).and_then(|val| val.checked_add(&x))
            });

            match updated {
                Some(updated) => {
                    val = updated;
                    started = true;
                }
                None => {
                    self.rio.ungetch(current);
                    break;
                }
            }
        }

        if started {
            Some(val)
        } else {
            None
        }
    }
}

impl<'a, R> Extract<String> for Extractor<'a, R>
where
    R: Iterator<Item = io::Result<u8>>,
{
    fn extract(&mut self) -> Option<String> {
        let mut result = String::new();

        let mut started = false;

        while let Some(current) = self.rio.getch() {
            if current.is_ascii_whitespace() {
                self.rio.ungetch(current);
                break;
            }

            result.push(current.into());
            started = true;
        }

        if started {
            Some(result)
        } else {
            None
        }
    }
}

fn main() {
    let mut rio = Rio::new(io::stdin().bytes());

    let a = rio.read::<i32>().unwrap_or_default();
    let b = rio.read::<i32>().unwrap_or_default();

    rio.clear();

    let s = rio.read::<String>().unwrap_or_default();

    println!("{} x {} = {}", a, b, a * b);
    println!("\"{}\"", s);
}
