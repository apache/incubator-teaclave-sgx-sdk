use sgx_serialize::opaque::{decode, encode};
pub use sgx_serialize::{Decodable, Deserialize, Encodable, Serialize};

#[derive(Debug)]
pub struct EncodeError;

#[derive(Debug)]
pub struct DecodeError;

pub fn serialize<T: sgx_serialize::Encodable>(object: &T) -> Result<Vec<u8>, EncodeError> {
    encode(object).ok_or(EncodeError)
}

pub fn deserialize<T: sgx_serialize::Decodable>(bytes: &[u8]) -> Result<T, DecodeError> {
    decode(bytes).ok_or(DecodeError)
}
