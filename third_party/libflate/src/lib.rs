//! A Rust implementation of DEFLATE algorithm and related formats (ZLIB, GZIP).
#![warn(missing_docs)]
#![no_std]


#[macro_use]
extern crate sgx_tstd as std;
extern crate adler32;
extern crate byteorder;

pub use finish::Finish;

macro_rules! invalid_data_error {
    ($fmt:expr) => { invalid_data_error!("{}", $fmt) };
    ($fmt:expr, $($arg:tt)*) => {
        ::std::io::Error::new(::std::io::ErrorKind::InvalidData, format!($fmt, $($arg)*))
    }
}

macro_rules! finish_try {
    ($e:expr) => {
        match $e.unwrap() {
            (inner, None) => inner,
            (inner, error) => return ::finish::Finish::new(inner, error)
        }
    }
}

#[allow(dead_code)]
pub mod lz77;
#[allow(dead_code)]
pub mod zlib;
//pub mod gzip;
#[allow(dead_code)]
pub mod deflate;
//pub mod non_blocking;

#[allow(dead_code)]
mod bit;
#[allow(dead_code)]
mod finish;
#[allow(dead_code)]
mod huffman;
#[allow(dead_code)]
mod checksum;
#[allow(dead_code)]
mod util;
