//! Gas capped at the maximum number of U256. It is practically
//! impossible to obtain this number during a block formation.

use super::{M256, U256};
#[cfg(feature = "string")]
use hexutil::ParseHexError;
#[cfg(feature = "rlp")]
use rlp::{Encodable, Decodable, RlpStream, DecoderError, UntrustedRlp};

#[cfg(feature = "std")] use std::ops::{Add, Sub, Mul, Div, Rem};
#[cfg(feature = "std")] use std::cmp::Ordering;
#[cfg(feature = "std")] use std::str::FromStr;
#[cfg(feature = "std")] use std::fmt;

#[cfg(not(feature = "std"))] use core::ops::{Add, Sub, Mul, Div, Rem};
#[cfg(not(feature = "std"))] use core::cmp::Ordering;
#[cfg(not(feature = "std"))] use core::str::FromStr;
#[cfg(not(feature = "std"))] use core::fmt;

#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct Gas(U256);

impl Gas {
    /// Zero value of Gas,
    pub fn zero() -> Gas { Gas(U256::zero()) }
    /// One value of Gas,
    pub fn one() -> Gas { Gas(U256::one()) }
    /// Maximum value of Gas,
    pub fn max_value() -> Gas { Gas(U256::max_value()) }
    /// Minimum value of Gas,
    pub fn min_value() -> Gas { Gas(U256::min_value()) }
    /// Bits required to represent this value.
    pub fn bits(self) -> usize { self.0.bits() }
    /// Equals `floor(log2(*))`. This is always an integer.
    pub fn log2floor(self) -> usize { self.0.log2floor() }
    /// Conversion to u32 with overflow checking
    ///
    /// # Panics
    ///
    /// Panics if the number is larger than 2^32.
    pub fn as_u32(&self) -> u32 {
        self.0.as_u32()
    }
    /// Conversion to u64 with overflow checking
    ///
    /// # Panics
    ///
    /// Panics if the number is larger than 2^64.
    pub fn as_u64(&self) -> u64 {
        self.0.as_u64()
    }
    /// Conversion to usize with overflow checking
    ///
    /// # Panics
    ///
    /// Panics if the number is larger than usize::max_value().
    pub fn as_usize(&self) -> usize {
        self.0.as_usize()
    }
}

impl Default for Gas { fn default() -> Gas { Gas::zero() } }

#[cfg(feature = "string")]
impl FromStr for Gas {
    type Err = ParseHexError;

    fn from_str(s: &str) -> Result<Gas, ParseHexError> {
        U256::from_str(s).map(|s| Gas(s))
    }
}

#[cfg(feature = "rlp")]
impl Encodable for Gas {
    fn rlp_append(&self, s: &mut RlpStream) {
        self.0.rlp_append(s);
    }
}

#[cfg(feature = "rlp")]
impl Decodable for Gas {
    fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
        Ok(Gas(U256::decode(rlp)?))
    }
}

impl From<u64> for Gas { fn from(val: u64) -> Gas { Gas(U256::from(val)) } }
impl Into<u64> for Gas { fn into(self) -> u64 { self.0.into() } }
impl From<usize> for Gas { fn from(val: usize) -> Gas { Gas(U256::from(val)) } }
impl<'a> From<&'a [u8]> for Gas { fn from(val: &'a [u8]) -> Gas { Gas(U256::from(val)) } }
impl From<bool> for Gas {
    fn from(val: bool) -> Gas {
        if val {
            Gas::one()
        } else {
            Gas::zero()
        }
    }
}
impl From<U256> for Gas { fn from(val: U256) -> Gas { Gas(val) } }
impl Into<U256> for Gas { fn into(self) -> U256 { self.0 } }
impl From<M256> for Gas {
    fn from(val: M256) -> Gas {
        Gas(val.into())
    }
}
impl Into<M256> for Gas {
    fn into(self) -> M256 {
        self.0.into()
    }
}

impl Ord for Gas { fn cmp(&self, other: &Gas) -> Ordering { self.0.cmp(&other.0) } }
impl PartialOrd for Gas {
    fn partial_cmp(&self, other: &Gas) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Add<Gas> for Gas {
    type Output = Gas;

    fn add(self, other: Gas) -> Gas {
        Gas(self.0.saturating_add(other.0))
    }
}

impl Sub<Gas> for Gas {
    type Output = Gas;

    fn sub(self, other: Gas) -> Gas {
        Gas(self.0 - other.0)
    }
}

impl Mul<Gas> for Gas {
    type Output = Gas;

    fn mul(self, other: Gas) -> Gas {
        Gas(self.0.saturating_mul(other.0))
    }
}

impl Div for Gas {
    type Output = Gas;

    fn div(self, other: Gas) -> Gas {
        Gas(self.0 / other.0)
    }
}

impl Rem for Gas {
    type Output = Gas;

    fn rem(self, other: Gas) -> Gas {
        Gas(self.0 % other.0)
    }
}

impl fmt::LowerHex for Gas {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl fmt::UpperHex for Gas {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X}", self.0)
    }
}
