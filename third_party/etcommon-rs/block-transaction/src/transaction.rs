use secp256k1::{self, Message, Error, Signature, RecoveryId, SecretKey, PublicKey};

use rlp::{self, Encodable, Decodable, RlpStream, DecoderError, UntrustedRlp};
use bigint::{Address, Gas, H256, U256, B256, M256};
use sha3::{Digest, Keccak256};
use address::FromKey;
use std::marker::PhantomData;
use std::str::FromStr;
use std::vec::Vec;
use super::{TransactionAction, RlpHash};

/// Refer to EIP155 related to chain ID.
pub trait SignaturePatch {
    fn chain_id() -> Option<u64>;
}

/// Frontier signature patch without EIP155.
pub struct GlobalSignaturePatch;
impl SignaturePatch for GlobalSignaturePatch {
    fn chain_id() -> Option<u64> { None }
}

/// EIP155 Ethereum Classic chain.
pub struct ClassicSignaturePatch;
impl SignaturePatch for ClassicSignaturePatch {
    fn chain_id() -> Option<u64> { Some(61) }
}

/// Refer to Homestead transaction validation.
pub trait ValidationPatch {
    fn require_low_s() -> bool;
}

/// Frontier validation patch.
pub struct FrontierValidationPatch;
impl ValidationPatch for FrontierValidationPatch {
    fn require_low_s() -> bool { false }
}

/// Homestead validation patch.
pub struct HomesteadValidationPatch;
impl ValidationPatch for HomesteadValidationPatch {
    fn require_low_s() -> bool { true }
}

const ECDSA_SIGNATURE_BYTES: usize = 65;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionSignature {
    pub v: u64,
    pub r: H256,
    pub s: H256,
}

impl TransactionSignature {
    pub fn standard_v(&self) -> u8 {
        let v = self.v;
        if v == 27 || v == 28 || v > 36 {
            ((v - 1) % 2) as u8
        } else {
            4
        }
    }

    pub fn is_low_s(&self) -> bool {
        self.s <= H256::from_str("0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A0").unwrap()
    }

    pub fn is_valid(&self) -> bool {
        self.standard_v() <= 1 &&
            self.r < H256::from_str("0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141").unwrap() &&
            self.r >= H256::from(1) &&
            self.s < H256::from_str("0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141").unwrap() &&
            self.s >= H256::from(1)
    }

    pub fn chain_id(&self) -> Option<u64> {
        if self.v > 36 {
            Some((self.v - 35) / 2)
        } else {
            None
        }
    }

    pub fn to_recoverable_signature(&self) -> Result<(Signature, RecoveryId), Error> {
        let mut sig = [0u8; 64];
        sig[0..32].copy_from_slice(&self.r);
        sig[32..64].copy_from_slice(&self.s);

        Ok((Signature::parse(&sig), RecoveryId::parse(self.standard_v() as u8)?))
    }
}

pub struct UnsignedTransaction {
    pub nonce: U256,
    pub gas_price: Gas,
    pub gas_limit: Gas,
    pub action: TransactionAction,
    pub value: U256,
    pub input: Vec<u8>,
}

impl UnsignedTransaction {
    fn signing_rlp_append(&self, s: &mut RlpStream, chain_id: Option<u64>) {
        s.begin_list(if chain_id.is_some() { 9 } else { 6 });
        s.append(&self.nonce);
        s.append(&self.gas_price);
        s.append(&self.gas_limit);
        s.append(&self.action);
        s.append(&self.value);
        s.append(&self.input);

        if let Some(chain_id) = chain_id {
            s.append(&chain_id);
            s.append(&0u8);
            s.append(&0u8);
        }
    }

    fn signing_hash(&self, chain_id: Option<u64>) -> H256 {
        let mut stream = RlpStream::new();
        self.signing_rlp_append(&mut stream, chain_id);
        H256::from(Keccak256::digest(&stream.drain()).as_slice())
    }

    pub fn sign<P: SignaturePatch>(self, key: &SecretKey) -> Transaction {
        let hash = self.signing_hash(P::chain_id());
        // hash is always MESSAGE_SIZE bytes.
        let msg = {
            { let mut a = [0u8; 32];
              for i in 0..32 {
                  a[i] = hash[i];
              }
              Message::parse(&a)
            }
        };

        // SecretKey and Message are always valid.
        let s = {
            { secp256k1::sign(&msg, key).unwrap() }
        };
        let (rid, sig) = {
            { (s.1, s.0.serialize()) }
        };

        let sig = TransactionSignature {
            v: ({
                { let v: i32 = rid.into(); v }
            } + if let Some(n) = P::chain_id() { (35 + n * 2) as i32 } else { 27 }) as u64,
            r: H256::from(&sig[0..32]),
            s: H256::from(&sig[32..64]),
        };

        Transaction {
            nonce: self.nonce,
            gas_price: self.gas_price,
            gas_limit: self.gas_limit,
            action: self.action,
            value: self.value,
            input: self.input,
            signature: sig,
        }
    }

    pub fn sign_global(self, key: &SecretKey) -> Transaction {
        self.sign::<GlobalSignaturePatch>(key)
    }
}

impl From<Transaction> for UnsignedTransaction {
    fn from(val: Transaction) -> UnsignedTransaction {
        UnsignedTransaction {
            nonce: val.nonce,
            gas_price: val.gas_price,
            gas_limit: val.gas_limit,
            action: val.action,
            value: val.value,
            input: val.input,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transaction {
    pub nonce: U256,
    pub gas_price: Gas,
    pub gas_limit: Gas,
    pub action: TransactionAction,
    pub value: U256,
    pub signature: TransactionSignature,
    pub input: Vec<u8>, // The input data, either data or init, depending on TransactionAction.
}

impl Transaction {
    pub fn caller(&self) -> Result<Address, Error> {
        let unsigned = UnsignedTransaction::from((*self).clone());
        let hash = unsigned.signing_hash(self.signature.chain_id());
        let sig = self.signature.to_recoverable_signature()?;
        let public_key = {
            { let mut a = [0u8; 32];
              for i in 0..32 {
                  a[i] = hash[i];
              }
              secp256k1::recover(&Message::parse(&a), &sig.0, &sig.1)?
            }
        };

        Ok(Address::from_public_key(&public_key))
    }

    pub fn address(&self) -> Result<Address, Error> {
        Ok(self.action.address(self.caller()?, self.nonce))
    }

    pub fn is_basic_valid<P: SignaturePatch, Q: ValidationPatch>(&self) -> bool {
        if !self.signature.is_valid() {
            return false;
        }

        if self.signature.chain_id().is_some() && self.signature.chain_id() != P::chain_id() {
            return false;
        }

        if self.caller().is_err() {
            return false;
        }

        if Q::require_low_s() && !self.signature.is_low_s() {
            return false;
        }

        return true;
    }
}

impl Encodable for Transaction {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(9);
        s.append(&self.nonce);
        s.append(&self.gas_price);
        s.append(&self.gas_limit);
        s.append(&self.action);
        s.append(&self.value);
        s.append(&self.input);
        s.append(&self.signature.v);
        s.append(&self.signature.r);
        s.append(&self.signature.s);
    }
}

impl Decodable for Transaction {
    fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
        Ok(Self {
            nonce: rlp.val_at(0)?,
            gas_price: rlp.val_at(1)?,
            gas_limit: rlp.val_at(2)?,
            action: rlp.val_at(3)?,
            value: rlp.val_at(4)?,
            input: rlp.val_at(5)?,
            signature: TransactionSignature {
                v: rlp.val_at(6)?,
                r: rlp.val_at(7)?,
                s: rlp.val_at(8)?,
            },
        })
    }
}

impl RlpHash for Transaction {
    fn rlp_hash(&self) -> H256 {
        H256::from(Keccak256::digest(&rlp::encode(self)).as_slice())
    }
}

#[cfg(test)]
mod tests {
    use secp256k1::{Message, Error, RecoverableSignature, RecoveryId, SECP256K1};
    use secp256k1::key::{PublicKey, SecretKey};
    use rlp::{self, Encodable, Decodable, RlpStream, DecoderError, UntrustedRlp};
    use bigint::{Address, Gas, H256, U256, B256};
    use sha3::{Digest, Keccak256};
    use address::FromKey;
    use rand::os::OsRng;
    use super::{Transaction, UnsignedTransaction, TransactionAction, ClassicSignaturePatch,
                HomesteadValidationPatch};

    #[test]
    pub fn should_recover_address() {
        let mut rng = OsRng::new().unwrap();
        let secret_key = SecretKey::new(&SECP256K1, &mut rng);
        let address = Address::from_secret_key(&secret_key);

        let unsigned = UnsignedTransaction {
            nonce: U256::zero(),
            gas_price: Gas::zero(),
            gas_limit: Gas::zero(),
            action: TransactionAction::Create,
            value: U256::zero(),
            input: Vec::new()
        };
        let signed = unsigned.sign::<ClassicSignaturePatch>(&secret_key);

        assert_eq!(signed.signature.chain_id(), Some(61));
        assert!(signed.is_basic_valid::<ClassicSignaturePatch, HomesteadValidationPatch>());
        assert_eq!(signed.caller(), address);
    }
}
