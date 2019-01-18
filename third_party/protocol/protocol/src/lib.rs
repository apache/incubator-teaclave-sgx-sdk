//! Simple packet-based protocol definitions in Rust.
//!
//! * The `Parcel` trait defines any type that can be serialized
//!   to a connection.
//! * The `wire` module deals with transmission of `Parcel`s.

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

pub use self::parcel::Parcel;
pub use self::errors::{Error, ErrorKind, ResultExt, CharTryFromError, TryFromIntError};

// Must go first because it defines common macros.
#[macro_use]
mod packet;

#[macro_use]
pub mod types;
#[macro_use]
pub mod wire;

mod errors;
mod parcel;

extern crate byteorder;
//extern crate flate2;
extern crate libflate;
#[macro_use]
extern crate error_chain;

#[cfg(feature = "uuid")]
extern crate uuid;
extern crate num_traits;

/// The default byte ordering.
pub type ByteOrder = ::byteorder::BigEndian;

