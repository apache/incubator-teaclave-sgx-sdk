use super::{H64, H128, H160, H256, H512, H520, H2048};
#[cfg(feature = "std")] use std::cmp;
#[cfg(not(feature = "std"))] use core::cmp;
use rlp::{RlpStream, Encodable, Decodable, DecoderError, UntrustedRlp};

macro_rules! impl_encodable_for_hash {
	($name: ident) => {
		impl Encodable for $name {
			fn rlp_append(&self, s: &mut RlpStream) {
				s.encoder().encode_value(self);
			}
		}
	}
}

macro_rules! impl_decodable_for_hash {
	($name: ident, $size: expr) => {
		impl Decodable for $name {
			fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
				rlp.decoder().decode_value(|bytes| match bytes.len().cmp(&$size) {
					cmp::Ordering::Less => Err(DecoderError::RlpIsTooShort),
					cmp::Ordering::Greater => Err(DecoderError::RlpIsTooBig),
					cmp::Ordering::Equal => {
						let mut t = [0u8; $size];
						t.copy_from_slice(bytes);
						Ok($name(t))
					}
				})
			}
		}
	}
}

impl_encodable_for_hash!(H64);
impl_encodable_for_hash!(H128);
impl_encodable_for_hash!(H160);
impl_encodable_for_hash!(H256);
impl_encodable_for_hash!(H512);
impl_encodable_for_hash!(H520);
impl_encodable_for_hash!(H2048);

impl_decodable_for_hash!(H64, 8);
impl_decodable_for_hash!(H128, 16);
impl_decodable_for_hash!(H160, 20);
impl_decodable_for_hash!(H256, 32);
impl_decodable_for_hash!(H512, 64);
impl_decodable_for_hash!(H520, 65);
impl_decodable_for_hash!(H2048, 256);
