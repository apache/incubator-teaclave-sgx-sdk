use crypto::rsgx_create_rsa_key_pair;
use crypto::{SgxRsaPrivKey, SgxRsaPubKey};
use itertools::Itertools;
use sgx_types::{sgx_status_t, size_t};
pub const SGX_RSA2048_KEY_SIZE: size_t     = 256;
pub const SGX_RSA2048_PRI_EXP_SIZE: size_t = 256;
pub const SGX_RSA2048_PUB_EXP_SIZE: size_t = 4;
pub const SGX_RSA2048_DEFAULT_E: [u8;SGX_RSA2048_PUB_EXP_SIZE]    = [0x01, 0x00, 0x00, 0x01]; // 65537
use std::fmt;

use serde_derive::*;

big_array! { BigArray; }

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Rsa2048KeyPair {
    #[serde(with = "BigArray")]
    n: [u8; SGX_RSA2048_KEY_SIZE],
    #[serde(with = "BigArray")]
    d: [u8; SGX_RSA2048_PRI_EXP_SIZE],
    e: [u8; SGX_RSA2048_PUB_EXP_SIZE],
    #[serde(with = "BigArray")]
    p: [u8; SGX_RSA2048_KEY_SIZE / 2],
    #[serde(with = "BigArray")]
    q: [u8; SGX_RSA2048_KEY_SIZE / 2],
    #[serde(with = "BigArray")]
    dmp1: [u8; SGX_RSA2048_KEY_SIZE / 2],
    #[serde(with = "BigArray")]
    dmq1: [u8; SGX_RSA2048_KEY_SIZE / 2],
    #[serde(with = "BigArray")]
    iqmp: [u8; SGX_RSA2048_KEY_SIZE / 2],
}

impl Default for Rsa2048KeyPair {
    fn default() -> Self {
        Rsa2048KeyPair {
            n: [0; SGX_RSA2048_KEY_SIZE],
            d: [0; SGX_RSA2048_PRI_EXP_SIZE],
            e: SGX_RSA2048_DEFAULT_E,
            p: [0; SGX_RSA2048_KEY_SIZE / 2],
            q: [0; SGX_RSA2048_KEY_SIZE / 2],
            dmp1: [0; SGX_RSA2048_KEY_SIZE / 2],
            dmq1: [0; SGX_RSA2048_KEY_SIZE / 2],
            iqmp: [0; SGX_RSA2048_KEY_SIZE / 2],
        }
    }
}

impl fmt::Debug for Rsa2048KeyPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r#"Rsa2048KeyPair: {{ n:{:02X}, d:{:02X}, e:{:02X}, p:{:02X}, q:{:02X}, dmp1:{:02X}, dmq:{:02X}, iqmp:{:02X} }}"#,
            self.n.iter().format(""),
            self.d.iter().format(""),
            self.e.iter().format(""),
            self.p.iter().format(""),
            self.q.iter().format(""),
            self.dmp1.iter().format(""),
            self.dmq1.iter().format(""),
            self.iqmp.iter().format(""))
    }
}

impl Rsa2048KeyPair {
    pub fn new() -> Result<Self, sgx_status_t> {
        let mut newkey = Self::default();
        match rsgx_create_rsa_key_pair(
            SGX_RSA2048_KEY_SIZE as i32,
            SGX_RSA2048_PUB_EXP_SIZE as i32,
            &mut newkey.n,
            &mut newkey.d,
            &mut newkey.e,
            &mut newkey.p,
            &mut newkey.q,
            &mut newkey.dmp1,
            &mut newkey.dmq1,
            &mut newkey.iqmp,
        ) {
            Ok(()) => Ok(newkey),
            Err(x) => Err(x),
        }
    }

    pub fn new_with_e(e: u32) -> Result<Self, sgx_status_t> {
        let mut newkey = Self::default();
        newkey.e = e.to_le_bytes();
        match rsgx_create_rsa_key_pair(
            SGX_RSA2048_KEY_SIZE as i32,
            SGX_RSA2048_PUB_EXP_SIZE as i32,
            &mut newkey.n,
            &mut newkey.d,
            &mut newkey.e,
            &mut newkey.p,
            &mut newkey.q,
            &mut newkey.dmp1,
            &mut newkey.dmq1,
            &mut newkey.iqmp,
        ) {
            Ok(()) => Ok(newkey),
            Err(x) => Err(x),
        }
    }

    pub fn to_privkey(self) -> Result<SgxRsaPrivKey, sgx_status_t> {
        let result = SgxRsaPrivKey::new();
        match result.create(
            SGX_RSA2048_KEY_SIZE as i32,
            SGX_RSA2048_PRI_EXP_SIZE as i32,
            &self.e,
            &self.p,
            &self.q,
            &self.dmp1,
            &self.dmq1,
            &self.iqmp,
        ) {
            Ok(()) => Ok(result),
            Err(x) => Err(x),
        }
    }

    pub fn to_pubkey(self) -> Result<SgxRsaPubKey, sgx_status_t> {
        let result = SgxRsaPubKey::new();
        match result.create(
            SGX_RSA2048_KEY_SIZE as i32,
            SGX_RSA2048_PUB_EXP_SIZE as i32,
            &self.n,
            &self.e,
        ) {
            Ok(()) => Ok(result),
            Err(x) => Err(x),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::rsa2048::Rsa2048KeyPair;
    use crate::rsa2048::SgxRsaPrivKey;
    use crate::rsa2048::SgxRsaPubKey;
    use crypto::rsgx_create_rsa_key_pair;
    use sgx_types::sgx_status_t;

    #[test]
    fn rsa2048_new() {
        let keypair = Rsa2048KeyPair::new();
        assert!(keypair.is_ok());
        let keypair = Rsa2048KeyPair::new_with_e(3);
        assert!(keypair.is_ok());
        let keypair = Rsa2048KeyPair::new_with_e(65537);
        assert!(keypair.is_ok());
    }

    #[test]
    fn rsa2048_new_fail() {
        let keypair = Rsa2048KeyPair::new_with_e(4);
        assert_eq!(keypair.unwrap_err(), sgx_status_t::SGX_ERROR_UNEXPECTED);
        let keypair = Rsa2048KeyPair::new_with_e(65536);
        assert_eq!(keypair.unwrap_err(), sgx_status_t::SGX_ERROR_UNEXPECTED);
    }

    #[test]
    fn rsa2048_to_sgx_rsa_pub_key() {
        let keypair = Rsa2048KeyPair::default();
        assert!(keypair.to_pubkey().is_err());
        let keypair = Rsa2048KeyPair::new().unwrap();
        assert!(keypair.to_pubkey().is_ok());
        let keypair = Rsa2048KeyPair::new_with_e(3).unwrap();
        assert!(keypair.to_pubkey().is_ok());
        let keypair = Rsa2048KeyPair::new_with_e(65537).unwrap();
        assert!(keypair.to_pubkey().is_ok());
    }

    #[test]
    fn rsa2048_to_sgx_rsa_priv_key() {
        let keypair = Rsa2048KeyPair::default();
        assert!(keypair.to_privkey().is_err());
        let keypair = Rsa2048KeyPair::new().unwrap();
        assert!(keypair.to_privkey().is_ok());
        let keypair = Rsa2048KeyPair::new_with_e(3).unwrap();
        assert!(keypair.to_privkey().is_ok());
        let keypair = Rsa2048KeyPair::new_with_e(65537).unwrap();
        assert!(keypair.to_privkey().is_ok());
    }

    #[test]
    fn rsa_encrypt_decrypt() {
        let text = String::from("abc");
        let text_slice = &text.into_bytes();

        let mod_size: i32 = 256;
        let exp_size: i32 = 4;
        let mut n: Vec<u8> = vec![0_u8; mod_size as usize];
        let mut d: Vec<u8> = vec![0_u8; mod_size as usize];
        let mut e: Vec<u8> = vec![1, 0, 1, 0];
        let mut p: Vec<u8> = vec![0_u8; mod_size as usize / 2];
        let mut q: Vec<u8> = vec![0_u8; mod_size as usize / 2];
        let mut dmp1: Vec<u8> = vec![0_u8; mod_size as usize / 2];
        let mut dmq1: Vec<u8> = vec![0_u8; mod_size as usize / 2];
        let mut iqmp: Vec<u8> = vec![0_u8; mod_size as usize / 2];

        assert!(rsgx_create_rsa_key_pair(
            mod_size,
            exp_size,
            n.as_mut_slice(),
            d.as_mut_slice(),
            e.as_mut_slice(),
            p.as_mut_slice(),
            q.as_mut_slice(),
            dmp1.as_mut_slice(),
            dmq1.as_mut_slice(),
            iqmp.as_mut_slice()
        )
        .is_ok());

        let privkey = SgxRsaPrivKey::new();
        let pubkey = SgxRsaPubKey::new();

        assert!(pubkey
            .create(mod_size, exp_size, n.as_slice(), e.as_slice())
            .is_ok());

        assert!(privkey
            .create(
                mod_size,
                exp_size,
                e.as_slice(),
                p.as_slice(),
                q.as_slice(),
                dmp1.as_slice(),
                dmq1.as_slice(),
                iqmp.as_slice()
            )
            .is_ok());

        let mut ciphertext: Vec<u8> = vec![0_u8; 256];
        let mut chipertext_len: usize = ciphertext.len();
        assert!(pubkey
            .encrypt_sha256(ciphertext.as_mut_slice(), &mut chipertext_len, text_slice)
            .is_ok());

        let mut plaintext: Vec<u8> = vec![0_u8; 256];
        let mut plaintext_len: usize = plaintext.len();
        assert!(privkey
            .decrypt_sha256(
                plaintext.as_mut_slice(),
                &mut plaintext_len,
                ciphertext.as_slice()
            )
            .is_ok());

        assert_eq!(plaintext[..plaintext_len], text_slice[..])
    }

}
