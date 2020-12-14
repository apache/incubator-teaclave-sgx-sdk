use crypto::rsgx_create_rsa_key_pair;
use crypto::{SgxRsaPrivKey, SgxRsaPubKey};
use itertools::Itertools;
use sgx_types::{sgx_status_t, size_t, SgxResult};
pub const SGX_RSA2048_KEY_SIZE: size_t     = 256;
pub const SGX_RSA2048_PRI_EXP_SIZE: size_t = 256;
pub const SGX_RSA2048_PUB_EXP_SIZE: size_t = 4;
pub const SGX_RSA2048_DEFAULT_E: [u8;SGX_RSA2048_PUB_EXP_SIZE]    = [0x01, 0x00, 0x00, 0x01]; // 16777217
use std::fmt;

use std::prelude::v1::*;
use crate::RsaKeyPair;
use serde_derive::*;

big_array! { BigArray; }

/// Data structure of RSA 2048 Keypair (RSA-OAEP).
/// RSA 2048 Keypair provides block cipher encryption/decryption
/// implementations. The block size of plain text is 190 bytes
/// and the block size of cipher text is 256 bytes.
/// 190 byte = 2048bit - 2*SHA256_BYTE - 2
#[cfg(any(feature = "mesalock_sgx", target_env = "sgx"))]
#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(crate = "serde_sgx")]
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
#[cfg(not(any(feature = "mesalock_sgx", target_env = "sgx")))]
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl RsaKeyPair for Rsa2048KeyPair {
    fn new() -> SgxResult<Self> {
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

    fn new_with_e(e: u32) -> SgxResult<Self> {
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

    fn to_privkey(self) -> SgxResult<SgxRsaPrivKey> {
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

    fn to_pubkey(self) -> SgxResult<SgxRsaPubKey> {
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

    fn encrypt_buffer(self, plaintext: &[u8], ciphertext: &mut Vec<u8>) -> SgxResult<usize> {
        let pubkey = self.to_pubkey()?;
        let bs = 256;

        // The magic number 190 comes from RSA-OAEP:
        // #define RSA_2048_KEY_BYTE    256
        // #define SHA_SIZE_BIT         256
        // #define RSAOAEP_ENCRYPT_MAXLEN RSA_2048_KEY_BYTE - 2*SHA_SIZE_BIT/8 - 2 /* 190 */
        // RSA_2048_KEY_BYTE - 2*SHA_SIZE_BIT/8 - 2
        let bs_plain = bs - 2 * 256 / 8 - 2;
        let count = (plaintext.len() + bs_plain - 1) / bs_plain;
        ciphertext.resize(bs * count, 0);

        for i in 0..count {
            let cipher_slice = &mut ciphertext[i * bs..i * bs + bs];
            let mut out_len = bs;
            let plain_slice =
                &plaintext[i * bs_plain..std::cmp::min(i * bs_plain + bs_plain, plaintext.len())];

            pubkey.encrypt_sha256(cipher_slice, &mut out_len, plain_slice)?;
        }

        Ok(ciphertext.len())
    }

    fn decrypt_buffer(self, ciphertext: &[u8], plaintext: &mut Vec<u8>) -> SgxResult<usize> {
        let privkey = self.to_privkey()?;
        let bs = 256;
        if ciphertext.len() % bs != 0 {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        // The magic number 190 comes from RSA-OAEP:
        // #define RSA_2048_KEY_BYTE    256
        // #define SHA_SIZE_BIT         256
        // #define RSAOAEP_ENCRYPT_MAXLEN RSA_2048_KEY_BYTE - 2*SHA_SIZE_BIT/8 - 2 /* 190 */
        // RSA_2048_KEY_BYTE - 2*SHA_SIZE_BIT/8 - 2
        //let bs_plain = 256 - 2 * 256 / 8 - 2;
        let bs_plain = bs;
        let count = ciphertext.len() / bs;
        plaintext.clear();

        for i in 0..count {
            let cipher_slice = &ciphertext[i * bs..i * bs + bs];
            let plain_slice = &mut vec![0;bs_plain];
            let mut plain_len = bs_plain;

            privkey.decrypt_sha256(plain_slice, &mut plain_len, cipher_slice)?;
            let mut decoded_vec = plain_slice[..plain_len].to_vec();
            plaintext.append(&mut decoded_vec);
        }

        Ok(plaintext.len())
    }

}

impl Rsa2048KeyPair {
    pub fn export_pubkey(self) -> SgxResult<Rsa2048PubKey> {
        Ok(Rsa2048PubKey {
            n: self.n,
            e: self.e,
        })
    }
}

#[cfg(any(feature = "mesalock_sgx", target_env = "sgx"))]
#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(crate = "serde_sgx")]
pub struct Rsa2048PubKey {
    #[serde(with = "BigArray")]
    n: [u8; SGX_RSA2048_KEY_SIZE],
    e: [u8; SGX_RSA2048_PUB_EXP_SIZE],
}
#[cfg(not(any(feature = "mesalock_sgx", target_env = "sgx")))]
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Rsa2048PubKey {
    #[serde(with = "BigArray")]
    n: [u8; SGX_RSA2048_KEY_SIZE],
    e: [u8; SGX_RSA2048_PUB_EXP_SIZE],
}

impl Default for Rsa2048PubKey {
    fn default() -> Self {
        Rsa2048PubKey {
            n: [0; SGX_RSA2048_KEY_SIZE],
            e: SGX_RSA2048_DEFAULT_E,
        }
    }
}

impl Rsa2048PubKey {
    pub fn to_pubkey(self) -> SgxResult<SgxRsaPubKey> {
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

    pub fn encrypt_buffer(self, plaintext: &[u8], ciphertext: &mut Vec<u8>) -> SgxResult<usize> {
        let pubkey = self.to_pubkey()?;
        let bs = 256;

        // The magic number 190 comes from RSA-OAEP:
        // #define RSA_2048_KEY_BYTE    256
        // #define SHA_SIZE_BIT         256
        // #define RSAOAEP_ENCRYPT_MAXLEN RSA_2048_KEY_BYTE - 2*SHA_SIZE_BIT/8 - 2 /* 190 */
        // RSA_2048_KEY_BYTE - 2*SHA_SIZE_BIT/8 - 2
        let bs_plain = bs - 2 * 256 / 8 - 2;
        let count = (plaintext.len() + bs_plain - 1) / bs_plain;
        ciphertext.resize(bs * count, 0);

        for i in 0..count {
            let cipher_slice = &mut ciphertext[i * bs..i * bs + bs];
            let mut out_len = bs;
            let plain_slice =
                &plaintext[i * bs_plain..std::cmp::min(i * bs_plain + bs_plain, plaintext.len())];

            pubkey.encrypt_sha256(cipher_slice, &mut out_len, plain_slice)?;
        }

        Ok(ciphertext.len())
    }
}

impl fmt::Debug for Rsa2048PubKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#"Rsa2048KeyPair: {{ n:{:02X}, e:{:02X} }}"#,
            self.n.iter().format(""),
            self.e.iter().format(""))
    }
}


#[cfg(test)]
mod tests {
    extern crate rdrand;
    extern crate rand_core;
    extern crate test;

    use self::rdrand::RdRand;
    use self::rand_core::RngCore;
    use crate::RsaKeyPair;
    use crate::rsa2048::Rsa2048KeyPair;
    use crate::rsa2048::Rsa2048PubKey;
    use crate::rsa2048::SgxRsaPrivKey;
    use crate::rsa2048::SgxRsaPubKey;
    use crypto::rsgx_create_rsa_key_pair;
    use sgx_types::sgx_status_t;
    use self::test::Bencher;

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
        assert_eq!(keypair.unwrap_err(), sgx_status_t::SGX_ERROR_UNEXPECTED); }

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

    #[test]
    fn buffer_enc_dec() {
        let plaintext: Vec<u8> = "A".repeat(1000).into_bytes();
        let mut ciphertext: Vec<u8> = Vec::new();
        let kp = Rsa2048KeyPair::new().unwrap();
        assert!(kp.encrypt_buffer(&plaintext, &mut ciphertext).is_ok());
        let mut decrypted: Vec<u8> = Vec::new();
        assert!(kp.decrypt_buffer(&ciphertext, &mut decrypted).is_ok());
        assert_eq!("A".repeat(1000), String::from_utf8(decrypted).unwrap());
    }

    #[test]
    fn export_test() {
        let plaintext: Vec<u8> = "T".repeat(1000).into_bytes();
        let mut ciphertext: Vec<u8> = Vec::new();
        let kp = Rsa2048KeyPair::new().unwrap();
        let exported_pub_key = kp.export_pubkey();
        assert!(exported_pub_key.is_ok());

        let exported_pub_key = exported_pub_key.unwrap();
        let serialized_pub_key = serde_json::to_string(&exported_pub_key).unwrap();
        let deserialized_pub_key: Rsa2048PubKey = serde_json::from_str(&serialized_pub_key).unwrap();

        assert!(deserialized_pub_key.encrypt_buffer(&plaintext, &mut ciphertext).is_ok());
        let mut decrypted: Vec<u8> = Vec::new();
        assert!(kp.decrypt_buffer(&ciphertext, &mut decrypted).is_ok());
        assert_eq!("T".repeat(1000), String::from_utf8(decrypted).unwrap());
    }

    #[bench]
    fn encrypt_speed_bench(b: &mut Bencher) {
        let mut rng = RdRand::new().unwrap();
        let mut buffer = vec![0;1*1024*1024];
        let kp = Rsa2048KeyPair::new().unwrap();
        let mut ciphertext: Vec<u8> = Vec::new();
        rng.fill_bytes(&mut buffer);
        b.iter(|| kp.encrypt_buffer(&buffer, &mut ciphertext));
    }

    #[bench]
    fn decrypt_speed_bench(b: &mut Bencher) {
        let mut rng = RdRand::new().unwrap();
        let mut buffer = vec![0;1*1024*1024];
        let kp = Rsa2048KeyPair::new().unwrap();
        let mut ciphertext: Vec<u8> = Vec::new();
        rng.fill_bytes(&mut buffer);
        kp.encrypt_buffer(&buffer, &mut ciphertext).unwrap();
        let mut decrypted: Vec<u8> = Vec::new();
        b.iter(|| kp.decrypt_buffer(&ciphertext, &mut decrypted));
    }
}
