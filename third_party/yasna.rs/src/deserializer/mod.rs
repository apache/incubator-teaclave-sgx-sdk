// Copyright 2016 Masaki Hara
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::hash::Hash;
use std::prelude::v1::*;

#[cfg(feature = "bigint")]
use num_bigint::{BigInt,BigUint};
#[cfg(feature = "bitvec")]
use bit_vec::BitVec;

use super::tags::{TAG_PRINTABLESTRING,TAG_UTCTIME};

use super::{ASN1Error,ASN1Result,ASN1ErrorKind,BERMode,BERReader,parse_ber_general};
use super::models::{PrintableString,UtcTime,ObjectIdentifier,SetOf};

pub trait FromBER: Sized + Eq + Hash {
    fn from_ber<'a, 'b>(reader: BERReader<'a, 'b>) -> ASN1Result<Self>;

    fn deserialize_ber_general(src: &[u8], mode: BERMode) -> ASN1Result<Self> {
        return parse_ber_general(src, mode, |reader| {
            return Self::from_ber(reader);
        });
    }
}

impl<T> FromBER for Vec<T> where T: Sized + Eq + Hash + FromBER {
    fn from_ber(reader: BERReader) -> ASN1Result<Self> {
        reader.read_sequence(|reader| {
            let mut ret = Vec::new();
            loop {
                let result = try!(reader.read_optional(|reader| {
                    T::from_ber(reader)
                }));
                match result {
                    Some(result) => {
                        ret.push(result);
                    },
                    None => {
                        break;
                    }
                };
            }
            return Ok(ret);
        })
    }
}

impl<T> FromBER for SetOf<T> where T: Sized + Eq + Hash + FromBER {
    fn from_ber<'a, 'b>(reader: BERReader<'a, 'b>) -> ASN1Result<Self> {
        let mut ret = SetOf::new();
        try!(reader.read_set_of(|reader| {
            ret.vec.push(try!(T::from_ber(reader)));
            return Ok(());
        }));
        return Ok(ret);
    }
}

impl FromBER for i64 {
    fn from_ber(reader: BERReader) -> ASN1Result<Self> {
        reader.read_i64()
    }
}

#[cfg(feature = "bigint")]
impl FromBER for BigInt {
    fn from_ber(reader: BERReader) -> ASN1Result<Self> {
        reader.read_bigint()
    }
}

#[cfg(feature = "bigint")]
impl FromBER for BigUint {
    fn from_ber(reader: BERReader) -> ASN1Result<Self> {
        match try!(BigInt::from_ber(reader)).to_biguint() {
            Some(result) => Ok(result),
            None => Err(ASN1Error::new(ASN1ErrorKind::Invalid)),
        }
    }
}

impl FromBER for () {
    fn from_ber(reader: BERReader) -> ASN1Result<Self> {
        reader.read_null()
    }
}

impl FromBER for bool {
    fn from_ber(reader: BERReader) -> ASN1Result<Self> {
        reader.read_bool()
    }
}

#[cfg(feature = "bitvec")]
impl FromBER for BitVec {
    fn from_ber(reader: BERReader) -> ASN1Result<Self> {
        reader.read_bitvec()
    }
}

impl FromBER for Vec<u8> {
    fn from_ber(reader: BERReader) -> ASN1Result<Self> {
        reader.read_bytes()
    }
}

impl FromBER for ObjectIdentifier {
    fn from_ber(reader: BERReader) -> ASN1Result<Self> {
        reader.read_oid()
    }
}

impl FromBER for PrintableString {
    fn from_ber(reader: BERReader) -> ASN1Result<Self> {
        reader.read_tagged_implicit(TAG_PRINTABLESTRING, |reader| {
            let octets = try!(reader.read_bytes());
            return PrintableString::from_bytes(octets)
                .ok_or(ASN1Error::new(ASN1ErrorKind::Invalid));
        })
    }
}

impl FromBER for UtcTime {
    fn from_ber(reader: BERReader) -> ASN1Result<Self> {
        reader.read_tagged_implicit(TAG_UTCTIME, |reader| {
            let octets = try!(reader.read_bytes());
            // TODO: format check
            return Ok(UtcTime::new(octets));
        })
    }
}
