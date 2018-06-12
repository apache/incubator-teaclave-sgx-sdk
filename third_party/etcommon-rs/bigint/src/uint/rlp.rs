// Copyright 2017 ETC Dev Team
// Copyright 2015-2017 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::{U256, U128};
use rlp::{RlpStream, Encodable, Decodable, DecoderError, UntrustedRlp};

macro_rules! impl_encodable_for_uint {
	($name: ident, $size: expr) => {
		impl Encodable for $name {
			fn rlp_append(&self, s: &mut RlpStream) {
				let leading_empty_bytes = $size - (self.bits() + 7) / 8;
				let mut buffer = [0u8; $size];
				self.to_big_endian(&mut buffer);
				s.encoder().encode_value(&buffer[leading_empty_bytes..]);
			}
		}
	}
}

macro_rules! impl_decodable_for_uint {
	($name: ident, $size: expr) => {
		impl Decodable for $name {
			fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
				rlp.decoder().decode_value(|bytes| {
					if !bytes.is_empty() && bytes[0] == 0 {
						Err(DecoderError::RlpInvalidIndirection)
					} else if bytes.len() <= $size {
						Ok($name::from(bytes))
					} else {
						Err(DecoderError::RlpIsTooBig)
					}
				})
			}
		}
	}
}

impl_encodable_for_uint!(U256, 32);
impl_encodable_for_uint!(U128, 16);

impl_decodable_for_uint!(U256, 32);
impl_decodable_for_uint!(U128, 16);
