#![allow(unused)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use num_traits::*;
use std::io::{self, Read};

enum InputError {
    IO(io::Error),
    Extraction,
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

struct Rio<R> {
    reader: R,
    ungetted: Option<u8>,
    ended: bool,
    error: Option<InputError>,
    ignore_count: u32,
    ignore_stop: Option<u8>,
}

trait Extract: Sized {
    fn extract<R>(rio: &mut Rio<R>) -> Option<Self>
    where
        R: Iterator<Item = io::Result<u8>>;
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
            ignore_count: 0,
            ignore_stop: None,
        }
    }

    fn getch_impl(&mut self) -> Option<u8> {
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

    fn getch(&mut self) -> Option<u8> {
        if self.error.is_some() || self.ended {
            return None;
        }

        while self.ignore_count > 0 {
            self.ignore_count -= 1;
            let c = self.getch_impl();
            if self.ignore_stop == c {
                self.ignore_count = 0;
            }
        }

        self.getch_impl()
    }

    fn ungetch(&mut self, byte: u8) {
        self.ungetted = Some(byte);
    }

    pub fn ignore(&mut self, count: u32, stop: Option<u8>) {
        self.ignore_count = count;
        self.ignore_stop = stop;
    }

    pub fn clear(&mut self) -> Option<InputError> {
        self.error.take()
    }

    pub fn readline(&mut self) -> Option<String> {
        let mut result = String::new();

        let mut started = false;

        while let Some(current) = self.getch() {
            started = true;

            if current == b'\n' {
                break;
            }

            result.push(current.into());
        }

        if started {
            Some(result)
        } else {
            None
        }
    }

    pub fn read<T>(&mut self) -> Option<T>
    where
        T: Extract,
    {
        // Skip whitespace
        while let Some(current) = self.getch() {
            if !current.is_ascii_whitespace() {
                self.ungetch(current);
                break;
            }
        }

        let result = <T as Extract>::extract(self);

        if result.is_none() {
            self.error = Some(InputError::Extraction);
        }

        result
    }
}

fn extract_int<T, R>(rio: &mut Rio<R>) -> Option<T>
where
    T: Zero + FromPrimitive + CheckedAdd<Output = T> + CheckedMul<Output = T>,
    R: Iterator<Item = io::Result<u8>>,
{
    let mut val: T = T::zero();
    let ten = small_value_to_num(10);

    let mut started = false;

    while let Some(current) = rio.getch() {
        let as_digit = to_digit(current);

        let updated = as_digit.and_then(|x| {
            let x = small_value_to_num(x);
            val.checked_mul(&ten).and_then(|val| val.checked_add(&x))
        });

        match updated {
            Some(updated) => {
                val = updated;
                started = true;
            }
            None => {
                rio.ungetch(current);
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

macro_rules! extract_int_impl {
    ($type:ty) => {
        impl Extract for $type {
            fn extract<R>(rio: &mut Rio<R>) -> Option<Self>
            where
                R: Iterator<Item = io::Result<u8>>,
            {
                extract_int(rio)
            }
        }
    };
}

extract_int_impl!(u8);
extract_int_impl!(i8);
extract_int_impl!(u16);
extract_int_impl!(i16);
extract_int_impl!(u32);
extract_int_impl!(i32);
extract_int_impl!(u64);
extract_int_impl!(i64);
extract_int_impl!(u128);
extract_int_impl!(i128);
extract_int_impl!(usize);
extract_int_impl!(isize);

impl Extract for String {
    fn extract<R>(rio: &mut Rio<R>) -> Option<Self>
    where
        R: Iterator<Item = io::Result<u8>>,
    {
        let mut result = String::new();

        let mut started = false;

        while let Some(current) = rio.getch() {
            if current.is_ascii_whitespace() {
                rio.ungetch(current);
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

    let a = rio.read::<u32>().unwrap_or_default();
    let b = rio.read::<u32>().unwrap_or_default();
    println!("{} x {} = {}", a, b, a * b);

    let s = rio.read::<String>().unwrap_or_default();
    println!("\"{}\"", s);

    let s = rio.readline().unwrap_or_default();
    println!("\"{}\"", s);

    let s = rio.readline().unwrap_or_default();
    println!("\"{}\"", s);
    
    let s = rio.readline().unwrap_or_default();
    println!("\"{}\"", s);
}
