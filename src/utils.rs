use num_traits::*;

pub(crate) fn to_digit(val: u8) -> Option<u8> {
    match val {
        b'0'..=b'9' => Some(val - b'0'),
        _ => None,
    }
}

pub(crate) fn small_value_to_num<T: FromPrimitive>(val: u8) -> T {
    unsafe { T::from_u8(val).unwrap_unchecked() }
}
