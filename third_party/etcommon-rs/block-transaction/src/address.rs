use secp256k1::{self, PublicKey, SecretKey, Error};
use bigint::{H256, Address};
use sha3::{Digest, Keccak256};

pub trait FromKey: Sized {
    fn from_public_key(key: &PublicKey) -> Self;
    fn from_secret_key(key: &SecretKey) -> Result<Self, Error>;
}

impl FromKey for Address {
    fn from_public_key(key: &PublicKey) -> Self {
        let hash = H256::from(
            Keccak256::digest(&key.serialize()[1..]).as_slice());
        Address::from(hash)
    }

    fn from_secret_key(key: &SecretKey) -> Result<Self, Error> {
        let public_key = PublicKey::from_secret_key(key);
        Ok(Self::from_public_key(&public_key))
    }
}
