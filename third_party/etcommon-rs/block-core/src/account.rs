use rlp::{Encodable, Decodable, RlpStream, DecoderError, UntrustedRlp};
use bigint::{Address, Gas, H256, U256, B256};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Account {
    pub nonce: U256,
    pub balance: U256,
    pub storage_root: H256,
    pub code_hash: H256,
}

impl Encodable for Account {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(4);
        s.append(&self.nonce);
        s.append(&self.balance);
        s.append(&self.storage_root);
        s.append(&self.code_hash);
    }
}

impl Decodable for Account {
    fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
        Ok(Self {
            nonce: rlp.val_at(0)?,
            balance: rlp.val_at(1)?,
            storage_root: rlp.val_at(2)?,
            code_hash: rlp.val_at(3)?,
        })
    }
}
