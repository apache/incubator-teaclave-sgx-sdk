extern crate json;

use json::number::Number;

#[test]
fn issue_107() {
    let n = unsafe { Number::from_parts_unchecked(true, 1, -32768) };
    assert_eq!(format!("{}", n), "1e-32768");
}

#[test]
fn issue_108_exponent_positive() {
    let n = unsafe { Number::from_parts_unchecked(true, 10_000_000_000_000_000_001, -18) };
    assert_eq!(format!("{}", n), "1.0000000000000000001e+1");
}

#[test]
fn issue_108_exponent_0() {
    let n = unsafe { Number::from_parts_unchecked(true, 10_000_000_000_000_000_001, -19) };
    assert_eq!(format!("{}", n), "1.0000000000000000001");
}

#[test]
fn trailing_zeroes_int() {
    let n = Number::from_parts(true, 100, -1);
    assert_eq!(format!("{}", n), "10");
}

#[test]
fn trailing_zeroes_fp() {
    let n = Number::from_parts(true, 100, -3);
    assert_eq!(format!("{}", n), "0.1");
}

#[test]
fn trailing_zeroes_small_fp() {
    let n = Number::from_parts(true, 100, -302);
    assert_eq!(format!("{}", n), "1e-300");
}
