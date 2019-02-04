#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate num;
extern crate num_bigint;

#[cfg(test)] extern crate rand;

use num::{Zero, One};
use num_bigint::{BigUint, ToBigUint};
use num::traits::ToPrimitive;
use std::prelude::v1::*;
use std::fmt;

pub use self::FromBase58Error::*;

const BTC_ALPHA: &'static[u8] = b"123456789\
                                  ABCDEFGHJKLMNPQRSTUVWXYZ\
                                  abcdefghijkmnopqrstuvwxyz";

const FLICKR_ALPHA: &'static[u8] = b"123456789\
                                     abcdefghijkmnopqrstuvwxyz\
                                     ABCDEFGHJKLMNPQRSTUVWXYZ";

/// A trait for converting base58-encoded values
pub trait FromBase58 {
    /// Converts the value of `self`, interpreted as base58 encoded data,
    /// into an owned vector of bytes, returning the vector.
    fn from_base58(&self) -> Result<Vec<u8>, FromBase58Error>;
}


/// Errors that can occur when decoding a base58-encoded string
#[derive(Clone, Copy)]
pub enum FromBase58Error {
    /// The input contained a character not part of the base58 alphabet
    InvalidBase58Byte(u8, usize),
}

impl fmt::Debug for FromBase58Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InvalidBase58Byte(ch, idx) =>
                write!(f, "Invalid character '{}' at position {}", ch, idx),
        }
    }
}

impl fmt::Display for FromBase58Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}



impl FromBase58 for str {
    fn from_base58(&self) -> Result<Vec<u8>, FromBase58Error> {
        self.as_bytes().from_base58()
    }
}

impl FromBase58 for [u8] {
    // TODO: fix some of the below when the binary assignment operators +=, *=
    // are overloadable
    fn from_base58(&self) -> Result<Vec<u8>, FromBase58Error> {
        let radix = 58.to_biguint().unwrap();
        let mut x: BigUint = Zero::zero();
        let mut rad_mult: BigUint = One::one();

        // Convert the base58 string to a BigUint `x`
        for (idx, &byte) in self.iter().enumerate().rev() {
            let first_idx = BTC_ALPHA.iter()
                                     .enumerate()
                                     .find(|x| *x.1 == byte)
                                     .map(|x| x.0);
            match first_idx {
                Some(i) => { x = x + i.to_biguint().unwrap() * &rad_mult; },
                None => return Err(InvalidBase58Byte(self[idx], idx))
            }

            rad_mult = &rad_mult * &radix;
        }

        let mut r = Vec::with_capacity(self.len());
        for _ in self.iter().take_while(|&x| *x == BTC_ALPHA[0]) {
            r.push(0);
        }
        if x > Zero::zero() {
            // TODO: use append when it becomes stable
            r.extend(x.to_bytes_be());
        }
        Ok(r)
    }
}


/// A trait for converting a value to base58 encoding.
pub trait ToBase58 {
    /// Converts the value of `self` to a base-58 value, returning the owned
    /// string.
    fn to_base58(&self) -> String;
}

impl ToBase58 for [u8] {
    // This function has to read in the entire byte slice and convert it to a
    // (big) int before creating the string. There's no way to incrementally read
    // the slice and create parts of the base58 string. Example:
    //   [1, 33] should be "5z"
    //   [1, 34] should be "61"
    // so by reading "1", no way to know if first character should be 5 or 6
    // without reading the rest
    fn to_base58(&self) -> String {
        let radix = 58.to_biguint().unwrap();
        let mut x = BigUint::from_bytes_be(&self);
        let mut ans = vec![];
        while x > Zero::zero() {
            let rem = (&x % &radix).to_usize().unwrap();
            ans.push(BTC_ALPHA[rem]);
            x = &x / &radix;
        }

        // take care of leading zeros
        for _ in self.iter().take_while(|&x| *x == 0) {
            ans.push(BTC_ALPHA[0]);
        }
        ans.reverse();
        String::from_utf8(ans).unwrap()
    }
}


#[cfg(test)]
mod tests {
    use super::{FromBase58, ToBase58};

    #[test]
    fn test_from_base58_basic() {
        assert_eq!("".from_base58().unwrap(), b"");
        assert_eq!("Z".from_base58().unwrap(), &[32]);
        assert_eq!("n".from_base58().unwrap(), &[45]);
        assert_eq!("q".from_base58().unwrap(), &[48]);
        assert_eq!("r".from_base58().unwrap(), &[49]);
        assert_eq!("z".from_base58().unwrap(), &[57]);
        assert_eq!("4SU".from_base58().unwrap(), &[45, 49]);
        assert_eq!("4k8".from_base58().unwrap(), &[49, 49]);
        assert_eq!("ZiCa".from_base58().unwrap(), &[97, 98, 99]);
        assert_eq!("3mJr7AoUXx2Wqd".from_base58().unwrap(), b"1234598760");
        assert_eq!("3yxU3u1igY8WkgtjK92fbJQCd4BZiiT1v25f".from_base58().unwrap(), b"abcdefghijklmnopqrstuvwxyz");
    }

    #[test]
    fn test_from_base58_bytes() {
        assert_eq!(b"ZiCa".from_base58().unwrap(), b"abc");
    }

    #[test]
    fn test_from_base58_invalid_char() {
        assert!("0".from_base58().is_err());
        assert!("O".from_base58().is_err());
        assert!("I".from_base58().is_err());
        assert!("l".from_base58().is_err());
        assert!("3mJr0".from_base58().is_err());
        assert!("O3yxU".from_base58().is_err());
        assert!("3sNI".from_base58().is_err());
        assert!("4kl8".from_base58().is_err());
        assert!("s!5<".from_base58().is_err());
        assert!("t$@mX<*".from_base58().is_err());
    }

    #[test]
    fn test_from_base58_initial_zeros() {
        assert_eq!("1ZiCa".from_base58().unwrap(), b"\0abc");
        assert_eq!("11ZiCa".from_base58().unwrap(), b"\0\0abc");
        assert_eq!("111ZiCa".from_base58().unwrap(), b"\0\0\0abc");
        assert_eq!("1111ZiCa".from_base58().unwrap(), b"\0\0\0\0abc");
    }

    #[test]
    fn test_to_base58_basic() {
        assert_eq!(b"".to_base58(), "");
        assert_eq!(&[32].to_base58(), "Z");
        assert_eq!(&[45].to_base58(), "n");
        assert_eq!(&[48].to_base58(), "q");
        assert_eq!(&[49].to_base58(), "r");
        assert_eq!(&[57].to_base58(), "z");
        assert_eq!(&[45, 49].to_base58(), "4SU");
        assert_eq!(&[49, 49].to_base58(), "4k8");
        assert_eq!(b"abc".to_base58(), "ZiCa");
        assert_eq!(b"1234598760".to_base58(), "3mJr7AoUXx2Wqd");
        assert_eq!(b"abcdefghijklmnopqrstuvwxyz".to_base58(), "3yxU3u1igY8WkgtjK92fbJQCd4BZiiT1v25f");
    }

    #[test]
    fn test_to_base58_initial_zeros() {
        assert_eq!(b"\0abc".to_base58(), "1ZiCa");
        assert_eq!(b"\0\0abc".to_base58(), "11ZiCa");
        assert_eq!(b"\0\0\0abc".to_base58(), "111ZiCa");
        assert_eq!(b"\0\0\0\0abc".to_base58(), "1111ZiCa");
    }

    #[test]
    fn test_base58_random() {
        use rand::{thread_rng, Rng};

        for _ in 0..200 {
            let times = thread_rng().gen_range(1, 100);
            let v = thread_rng().gen_iter::<u8>().take(times)
                                .collect::<Vec<_>>();
            assert_eq!(v.to_base58()
                        .from_base58()
                        .unwrap(),
                       v);
        }
    }
}
