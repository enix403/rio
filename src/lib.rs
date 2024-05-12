use std::io::Read;

mod error;
mod stream;
mod utils;

use num_traits::*;

use crate::error::InputError;
use crate::stream::{Fetch, RioStream};

pub struct Rio<R> {
    stream: RioStream<R>,
    ended: bool,
    ungetted: Option<u8>,
    error: Option<InputError>,
    ignore_count: u32,
    ignore_stop: Option<u8>,
}

pub trait Extract: Sized {
    fn extract(rio: &mut Rio<impl Read>) -> Option<Self>;
}

impl<R: Read> Rio<R> {
    pub fn new(reader: R) -> Self {
        Self {
            stream: RioStream::new(reader),
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

fn extract_int<T>(rio: &mut Rio<impl Read>, is_signed: bool) -> Option<T>
where
    T: Zero + FromPrimitive + CheckedAdd<Output = T> + CheckedMul<Output = T>,
{
    let mut val: T = T::zero();
    let ten = utils::small_value_to_num(10);

    let mut started = false;

    let mut sign: i8 = 1;
    let mut sign_consumed = false;

    while let Some(current) = rio.getch() {
        let as_digit = utils::to_digit(current);

        if is_signed {
            if !started && !sign_consumed && current == b'-' {
                sign = -1;
                sign_consumed = true;
                continue;
            }
        }

        let updated = as_digit.and_then(|x| {
            let x = utils::small_value_to_num(x * sign);
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
        if sign_consumed {
            rio.ungetch(b'-');
        }
        None
    }
}

macro_rules! extract_int_impl {
    ($type:ty, $is_signed:expr) => {
        impl Extract for $type {
            fn extract(rio: &mut Rio<impl Read>) -> Option<Self> {
                extract_int(rio, $is_signed)
            }
        }
    };
}

extract_int_impl!(u8, false);
extract_int_impl!(u16, false);
extract_int_impl!(u32, false);
extract_int_impl!(u64, false);
extract_int_impl!(u128, false);
extract_int_impl!(usize, false);

extract_int_impl!(i8, true);
extract_int_impl!(i16, true);
extract_int_impl!(i32, true);
extract_int_impl!(i64, true);
extract_int_impl!(i128, true);
extract_int_impl!(isize, true);

impl Extract for f32 {
    fn extract(rio: &mut Rio<impl Read>) -> Option<Self> {
        let mut val: Self = 0.0;

        let mut started = false;

        let mut sign: Self = 1.0;
        let mut sign_consumed = false;

        let mut left_side = true;
        let mut fract_mult: Self = 0.1;

        while let Some(current) = rio.getch() {
            if left_side && !started && !sign_consumed && current == b'-' {
                sign = -1.0;
                sign_consumed = true;
                continue;
            }

            if left_side && current == b'.' {
                left_side = false;
                continue;
            }

            let Some(digit) = utils::to_digit(current) else {
                rio.ungetch(current);
                break;
            };

            let digit = digit as Self;

            if left_side {
                val.mul_add_assign(10.0, digit * sign);
            }
            else {
                val += fract_mult * digit * sign;
                fract_mult *= 0.1;
            }
            started = true;
        }

        if started {
            Some(val)
        } else {
            if !left_side {
                rio.ungetch(b'.');
            }
            // any sign consumed will not be ungetch'ed
            None
        }
    }
}

impl Extract for String {
    fn extract(rio: &mut Rio<impl Read>) -> Option<Self> {
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
