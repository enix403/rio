use num_traits::*;

pub(crate) fn to_digit(val: u8) -> Option<i8> {
    match val {
        b'0'..=b'9' => Some((val - b'0') as i8),
        _ => None,
    }
}

pub(crate) fn small_value_to_num<T: FromPrimitive>(val: i8) -> T {
    unsafe { T::from_i8(val).unwrap_unchecked() }
}
