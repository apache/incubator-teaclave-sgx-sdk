use rlp::{Encodable, Decodable, RlpStream, DecoderError, UntrustedRlp};
use bigint::{Address, Gas, H256, U256, B256, H64};

#[cfg(not(feature = "std"))]
use alloc::Vec;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Log {
    pub address: Address,
    pub topics: Vec<H256>,
    pub data: Vec<u8>,
}

impl Encodable for Log {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(3);
        s.append(&self.address);
        s.append_list(&self.topics);
        s.append(&self.data);
    }
}

impl Decodable for Log {
    fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
        Ok(Self {
            address: rlp.val_at(0)?,
            topics: rlp.list_at(1)?,
            data: rlp.val_at(2)?,
        })
    }
}
