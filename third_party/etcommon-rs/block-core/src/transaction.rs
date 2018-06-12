use rlp::{UntrustedRlp, DecoderError, RlpStream, Encodable, Decodable};
use bigint::{Address, U256, M256};
use sha3::{Digest, Keccak256};

// Use transaction action so we can keep most of the common fields
// without creating a large enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransactionAction {
    Call(Address),
    Create,
}

impl TransactionAction {
    pub fn address(&self, caller: Address, nonce: U256) -> Address {
        match self {
            &TransactionAction::Call(address) => address,
            &TransactionAction::Create => {
                let mut rlp = RlpStream::new_list(2);
                rlp.append(&caller);
                rlp.append(&nonce);

                Address::from(M256::from(Keccak256::digest(rlp.out().as_slice()).as_slice()))
            },
        }
    }
}

impl Encodable for TransactionAction {
    fn rlp_append(&self, s: &mut RlpStream) {
        match self {
            &TransactionAction::Call(address) => {
                s.encoder().encode_value(&address);
            },
            &TransactionAction::Create => {
                s.encoder().encode_value(&[])
            },
        }
    }
}

impl Decodable for TransactionAction {
    fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
        Ok(if rlp.is_empty() {
            TransactionAction::Create
        } else {
            TransactionAction::Call(rlp.as_val()?)
        })
    }
}
