use std::prelude::v1::*;
use byteorder::{ReadBytesExt, NativeEndian};

pub fn u64_to_u8_8(v: u64) -> [u8; 8] {
    unsafe {
        ::std::mem::transmute::<u64, [u8; 8]>(v)
    }
}

pub fn u8_8_to_u64(v: [u8; 8]) -> u64 {
    unsafe {
        ::std::mem::transmute::<[u8; 8], u64>(v)
    }
}

pub fn u64_to_f64(v: u64) -> Option<f64> {
    match (&u64_to_u8_8(v) as &[u8]).read_f64::<NativeEndian>() {
        Ok(v) => Some(v),
        Err(_) => None
    }
}

pub fn f64_to_u64(v: f64) -> u64 {
    unsafe {
        ::std::mem::transmute::<f64, u64>(v)
    }
}
