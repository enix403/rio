use std::io;

use num_traits::*;

use crate::error::InputError;
use crate::stream::{Fetch, RioStream};
use crate::utils;

pub struct Rio<R> {
    stream: R,
    ended: bool,
    ungetted: Option<u8>,
    error: Option<InputError>,
    ignore_count: u32,
    ignore_stop: Option<u8>,
}

pub trait Extract: Sized {
    fn extract(rio: &mut Rio<impl RioStream>) -> Option<Self>;
}

impl<R: RioStream> Rio<R> {
    pub fn new(stream: R) -> Self {
        Self {
            stream,
            ended: false,
            ungetted: None,
            error: None,
            ignore_count: 0,
            ignore_stop: None,
        }
    }

    fn ungetch(&mut self, byte: u8) {
        self.ungetted = Some(byte);
    }

    fn getch_impl(&mut self) -> Option<u8> {
        match self.ungetted {
            Some(_) => self.ungetted.take(),
            _ => match self.stream.getch() {
                Fetch::Byte(byte) => Some(byte),
                Fetch::Error(error) => {
                    self.error = Some(InputError::IO(error));
                    None
                }
                Fetch::Ended => {
                    self.ended = true;
                    None
                }
            },
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

    pub fn read_n<T, N>(&mut self, n: N) -> Option<Vec<T>>
    where
        T: Extract,
        N: Unsigned + ToPrimitive,
    {
        std::iter::repeat_with(|| self.read())
            .take(unsafe { n.to_usize().unwrap_unchecked() })
            .collect()
    }

    pub fn read_or_default<T>(&mut self) -> T
    where
        T: Extract + Default,
    {
        self.read().unwrap_or_default()
    }

    pub fn read_n_or_default<T, N>(&mut self, n: N) -> Vec<T>
    where
        T: Extract + Default,
        N: Unsigned + ToPrimitive,
    {
        std::iter::repeat_with(|| self.read_or_default())
            .take(unsafe { n.to_usize().unwrap_unchecked() })
            .collect()
    }
}

fn extract_int<T, R>(rio: &mut Rio<R>) -> Option<T>
where
    T: Zero + FromPrimitive + CheckedAdd<Output = T> + CheckedMul<Output = T>,
    R: RioStream,
{
    let mut val: T = T::zero();
    let ten = utils::small_value_to_num(10);

    let mut started = false;

    while let Some(current) = rio.getch() {
        let as_digit = utils::to_digit(current);

        let updated = as_digit.and_then(|x| {
            let x = utils::small_value_to_num(x);
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
            fn extract(rio: &mut Rio<impl RioStream>) -> Option<Self> {
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
    fn extract(rio: &mut Rio<impl RioStream>) -> Option<Self> {
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