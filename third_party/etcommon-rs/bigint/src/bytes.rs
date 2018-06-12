#[cfg(feature = "rlp")]
use rlp::{Encodable, Decodable, RlpStream, DecoderError, UntrustedRlp};

/// Maximum 256-bit byte-array that does not require heap allocation.
#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct B256(usize, [u8; 32]);

impl B256 {
    pub fn new(value: &[u8]) -> Self {
        assert!(value.len() <= 32);
        let mut ret = [0u8; 32];
        for i in 0..value.len() {
            ret[i] = value[i];
        }
        B256(value.len(), ret)
    }
}

impl Default for B256 {
    fn default() -> B256 {
        B256(0, [0u8; 32])
    }
}

#[cfg(feature = "rlp")]
impl Encodable for B256 {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.encoder().encode_value(&self.1[0..self.0])
    }
}

#[cfg(feature = "rlp")]
impl Decodable for B256 {
    fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
        rlp.decoder().decode_value(|bytes| {
            if bytes.len() > 32 {
                Err(DecoderError::Custom("More than 32 bytes"))
            } else {
                let mut ret = B256(0, [0u8; 32]);
                ret.0 = bytes.len();
                for i in 0..bytes.len() {
                    ret.1[i] = bytes[i];
                }
                Ok(ret)
            }
        })
    }
}
