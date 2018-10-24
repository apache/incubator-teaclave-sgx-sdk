use std::prelude::v1::*;
use numbers::{PRIMES, NPRIMES};
use extern_rand::Rand;
use extern_rand::Rng as OtherRng;
use extern_rand;
use std::mem;

pub struct Rng(extern_rand::ThreadRng);

impl Rng {
    pub fn new() -> Rng {
        Rng(extern_rand::thread_rng())
    }

    pub fn gen_bytes(&mut self, n: usize) -> Vec<u8> {
        let mut res = vec![0;n];
        self.0.fill_bytes(&mut res);
        res
    }

    pub fn gen_16_bytes(&mut self) -> [u8;16] {
        let mut res = [0;16];
        self.0.fill_bytes(&mut res);
        res
    }

    pub fn gen_usize(&mut self) -> usize {
        Rand::rand(&mut self.0)
    }

    pub fn gen_u32(&mut self) -> u32 {
        Rand::rand(&mut self.0)
    }

    pub fn gen_u64(&mut self) -> u64 {
        Rand::rand(&mut self.0)
    }

    pub fn gen_u128(&mut self) -> u128 {
        let low64:  u64 = Rand::rand(&mut self.0);
        let high64: u64 = Rand::rand(&mut self.0);
        let (high128, _) = (high64 as u128).overflowing_shl(64) ;
        high128 + low64 as u128
    }

    pub fn gen_byte(&mut self) -> u8 {
        Rand::rand(&mut self.0)
    }

    pub fn gen_bool(&mut self) -> bool {
        Rand::rand(&mut self.0)
    }

    pub fn gen_prime(&mut self) -> u8 {
        PRIMES[self.gen_byte() as usize % NPRIMES]
    }

    pub fn gen_modulus(&mut self) -> u8 {
        2 + (self.gen_byte() % 111)
    }

    pub fn gen_usable_composite_modulus(&mut self) -> u128 {
        self._gen_usable_composite_modulus().iter().fold(1, |acc, &x| {
            acc * x as u128
        })
    }

    pub fn _gen_usable_composite_modulus(&mut self) -> Vec<u8> {
        let mut x: u128 = 1;
        PRIMES.into_iter().cloned()
            .filter(|_| self.gen_bool()) // randomly take this prime
            .take_while(|&q| { // make sure that we don't overflow!
                match x.checked_mul(q as u128) {
                    None => false,
                    Some(y) => {
                        x = y;
                        true
                    },
                }
            }).collect()
    }
}

pub struct Aes {
    round_keys: [u8; 176],
}

impl Clone for Aes {
    fn clone(&self) -> Self {
        let mut new = Aes { round_keys: [0; 176] };
        new.round_keys[..176].clone_from_slice(&self.round_keys[..176]);
        new
    }
}

pub const AES: Aes = Aes {
    round_keys: [0x15, 0xb5, 0x32, 0xc2, 0xf1, 0x93, 0x1c, 0x94, 0xd7, 0x54,
                 0x87, 0x6d, 0xfe, 0x7e, 0x67, 0x26, 0xa7, 0xeb, 0x4f, 0x98,
                 0x19, 0x86, 0xcf, 0xcf, 0x80, 0xe6, 0xbb, 0xed, 0xf8, 0x8d,
                 0xe8, 0xc9, 0x12, 0x10, 0x4b, 0x44, 0x43, 0xd8, 0xb3, 0x5c,
                 0xf4, 0x67, 0x7b, 0x3c, 0x8d, 0xcb, 0x04, 0x7b, 0x57, 0x8c,
                 0xdb, 0xac, 0xae, 0xd1, 0xc9, 0xdc, 0x29, 0x5d, 0x20, 0x51,
                 0xcf, 0x6f, 0x5e, 0x25, 0x0c, 0xe1, 0xfd, 0x36, 0x50, 0xde,
                 0xff, 0xab, 0xdd, 0xfa, 0x4f, 0xe9, 0xe2, 0xcd, 0x2d, 0x23,
                 0x96, 0xf6, 0x76, 0x9d, 0xaf, 0x14, 0x18, 0xd2, 0x51, 0x7e,
                 0x4b, 0x1d, 0xf9, 0xf0, 0x86, 0x4a, 0x29, 0x1c, 0x77, 0xd9,
                 0x58, 0x93, 0xc6, 0xef, 0xbc, 0xec, 0x74, 0xbe, 0x84, 0xc1,
                 0x2f, 0xbf, 0x55, 0xc2, 0xeb, 0x3c, 0x56, 0xa9, 0x92, 0x1a,
                 0xb2, 0xc6, 0xf2, 0x38, 0x6e, 0x4d, 0xfb, 0xca, 0x8e, 0x07,
                 0x20, 0x19, 0xb9, 0x12, 0xd8, 0xaf, 0x95, 0xe1, 0x15, 0x6e,
                 0xd9, 0xd1, 0xe7, 0xef, 0x4c, 0x2b, 0x34, 0x4e, 0x25, 0x1a,
                 0x9a, 0x49, 0x07, 0xa5, 0x23, 0x69, 0xa7, 0x55, 0xe4, 0xaf,
                 0x1f, 0x44, 0xeb, 0x6e, 0xbc, 0x0b, 0x40, 0x0c, 0x7c, 0x58,
                 0xb7, 0x54, 0x9a, 0xa0, 0x9b, 0x32],
};

impl Aes {
    pub fn new(key: u128) -> Self {
        let key_bytes = u128_to_bytes(key);
        Self::from_bytes(key_bytes)
    }

    pub fn from_rng(rng: &mut Rng) -> Self {
        Self::from_bytes(rng.gen_16_bytes())
    }

    pub fn from_bytes(key_bytes: [u8;16]) -> Self {
        let mut round_keys = [0u8; 176];
        unsafe {
            aesni_setup_round_key_128(key_bytes.as_ptr(), round_keys.as_mut_ptr());
        }
        Aes { round_keys }
    }

    pub fn hash(&self, t: u128, x: u128) -> u128 {
        let y = poly_double(x) ^ t;
        self.eval_u128(x) ^ y
    }

    pub fn hash2(&self, t: u128, x: u128, y: u128) -> u128 {
        let z = x ^ poly_double(y);
        self.hash(z, t)
    }

    pub fn eval_u128(&self, x: u128) -> u128 {
        let inp_bytes = u128_to_bytes(x);
        bytes_to_u128(self.eval(inp_bytes))
    }

    pub fn eval(&self, inp_bytes: [u8;16]) -> [u8;16] {
        let mut out_bytes = [0; 16];
        unsafe {
            aesni_encrypt_block(10, inp_bytes.as_ptr(), self.round_keys.as_ptr(), out_bytes.as_mut_ptr());
        }
        out_bytes
    }
}

extern {
    fn aesni_setup_round_key_128(key: *const u8, round_key: *mut u8);
    fn aesni_encrypt_block(rounds: u8, input: *const u8, round_keys: *const u8, output: *mut u8);
}

// irr128 = x^128 + x^7 + x^2 + x + 1
fn poly_double(x: u128) -> u128 {
    let (y, overflow) = x.overflowing_shl(1);
    if overflow {
        // if there is overflow, mod by irr128
        y ^ 0x87
    } else {
        y
    }
}

fn u128_to_bytes(x: u128) -> [u8;16] {
    unsafe {
        mem::transmute(x)
    }
}

fn bytes_to_u128(bytes: [u8;16]) -> u128 {
    unsafe {
        mem::transmute(bytes)
    }
}

#[cfg(test)]
mod tests {
    use rand::{Aes, Rng, bytes_to_u128, u128_to_bytes};

    #[test]
    fn u128_conversion() {
        let mut rng = Rng::new();
        for _ in 0..128 {
            let x = rng.gen_u128();
            assert_eq!(x, bytes_to_u128(u128_to_bytes(x)));
            let bs = rng.gen_16_bytes();
            assert_eq!(bs, u128_to_bytes(bytes_to_u128(bs)));
        }
        assert_eq!(1, bytes_to_u128([1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]));
    }

    #[test]
    fn aes_zero_correct() {
        let aes = Aes::new(0);
        let res = aes.hash(0, 0);
        let should_be = bytes_to_u128([102, 233, 75, 212, 239, 138, 44, 59, 136, 76, 250, 89, 202,
                                      52, 43, 46]);
        assert_eq!(res, should_be);
    }

    #[test]
    fn random_aes_correct() {
        let key = [0x06, 0xa2, 0xf9, 0xe0, 0x79, 0x27, 0x6a, 0x08,
                   0x04, 0x34, 0xb6, 0x61, 0xba, 0xee, 0xdc, 0xef];
        let inp = [0x00, 0xd6, 0x18, 0x23, 0x4f, 0x1b, 0x61, 0xce,
                   0x3b, 0xde, 0x41, 0x04, 0xc5, 0x93, 0xb6, 0x1c];
        let should_be = [0x84, 0xd1, 0xc3, 0x11, 0x07, 0x5a, 0x96, 0x4a,
                         0x13, 0xf8, 0x83, 0x35, 0xf9, 0x04, 0x9d, 0x4a];

        let aes = Aes::from_bytes(key);
        let out = aes.eval(inp);
        assert_eq!(out, should_be);

        let aes = Aes::new(bytes_to_u128(key));
        let out = aes.eval_u128(bytes_to_u128(inp));
        assert_eq!(out, bytes_to_u128(should_be));
    }

    #[test]
    fn rng() {
        let mut rng = Rng::new();
        let aes = Aes::new(rng.gen_u128());
        aes.hash(0, rng.gen_u128());
        let i = rng.gen_u128();
        assert!(i > 0);
    }
}
