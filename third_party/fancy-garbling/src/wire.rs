use std::prelude::v1::*;
use std;
use rand::{AES, Rng};
use base_conversion;
use numbers;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Wire {
    Mod2 { val: u128 },
    ModN { q: u8, ds: Vec<u8> },
}

impl Wire {
    pub fn digits(&self) -> Vec<u8> {
        match self {
            Wire::Mod2 { .. } => unimplemented!(),
            Wire::ModN { ds, .. } => ds.clone(),
        }
    }

    pub fn modulus(&self) -> u8 {
        match *self {
            Wire::Mod2 { .. } => 2,
            Wire::ModN { q, .. } => q,
        }
    }

    pub fn from_u128(inp: u128, q: u8) -> Self {
        if q == 2 {
            Wire::Mod2 { val: inp }

        } else if base_conversion::lookup_defined_for_mod(q) {
            let bytes = unsafe { std::mem::transmute::<u128, [u8;16]>(inp) };
            let mut ds = vec![0; numbers::digits_per_u128(q)];

            for i in 0..16 {
                let c = base_conversion::lookup_digits_mod_at_position(bytes[i], q, i);
                numbers::base_q_add(&mut ds, &c, q);
            }

            Wire::ModN { q, ds }

        } else {
            Wire::ModN { q, ds: numbers::padded_base_q_128(inp, q) }
        }
    }

    pub fn as_u128(&self) -> u128 {
        match *self {
            Wire::Mod2 { val } => val,
            Wire::ModN { q, ref ds } => numbers::from_base_q(ds, q),
        }
    }

    pub fn zero(modulus: u8) -> Self {
        match modulus {
            1 => panic!("[wire::zero] mod 1 not allowed!"),
            2 => Wire::Mod2 { val: 0 },
            _ => Wire::from_u128(0, modulus),
        }
    }

    pub fn rand_delta(rng: &mut Rng, modulus: u8) -> Self {
        let mut w = Self::rand(rng, modulus);
        match w {
            Wire::Mod2 { ref mut val }    => *val |= 1,
            Wire::ModN { ref mut ds, .. } => ds[0] = 1,
        }
        w
    }

    pub fn color(&self) -> u8 {
        match *self {
            Wire::Mod2 { val }        => (val & 1) as u8,
            Wire::ModN { ref ds, .. } => ds[0],
        }
    }

    pub fn plus(&self, other: &Self) -> Self {
        match (self, other) {
            (&Wire::Mod2 { val: x }, &Wire::Mod2 { val: y }) => {
                Wire::Mod2 { val: x ^ y }
            }

            (&Wire::ModN { q: xmod, ds: ref xs }, &Wire::ModN { q: ymod, ds: ref ys }) => {
                assert_eq!(xmod, ymod);
                assert_eq!(xs.len(), ys.len());
                let n = xs.len();
                let mut zs = vec![0;n];
                for i in 0..n {
                    let mut z = xs[i] as u16 + ys[i] as u16;
                    if z >= xmod as u16 {
                        z -= xmod as u16;
                    }
                    zs[i] = z as u8;
                }
                Wire::ModN { q: xmod, ds: zs }
            }

            _ => panic!("[wire::plus] unequal moduli!"),
        }
    }

    pub fn cmul(&self, c: u8) -> Self {
        match *self {
            Wire::Mod2 { .. } => {
                if c % 2 == 0 {
                    Wire::zero(2)
                } else {
                    self.clone()
                }
            }

            Wire::ModN { q, ref ds } => {
                let n = ds.len();
                let mut zs = vec![0;n];
                for i in 0..n {
                    zs[i] = ((ds[i] as u16 * c as u16) % q as u16) as u8;
                }
                Wire::ModN { q, ds: zs }
            }
        }
    }

    pub fn negate(&self) -> Self {
        match *self {
            Wire::Mod2 { .. } => self.clone(),
            Wire::ModN { q, ref ds }  => {
                let n = ds.len();
                let mut zs = vec![0;n];
                for i in 0..n {
                    if ds[i] > 0 {
                        zs[i] = q - ds[i];
                    }
                }
                Wire::ModN { q, ds: zs }
            }
        }
    }

    pub fn minus(&self, other: &Self) -> Self {
        self.plus(&other.negate())
    }

    pub fn rand(rng: &mut Rng, modulus: u8) -> Self {
        Self::from_u128(rng.gen_u128(), modulus)
    }

    pub fn hash(&self, tweak: u128) -> u128 {
        AES.hash(tweak, self.as_u128())
    }

    // hash to u128 and back to Wire
    pub fn hashback(&self, tweak: u128, new_mod: u8) -> Self {
        Self::from_u128(self.hash(tweak), new_mod)
    }

    pub fn hash2(&self, other: &Wire, tweak: u128) -> u128 {
        AES.hash2(tweak, self.as_u128(), other.as_u128())
    }

    pub fn hashback2(&self, other: &Wire, tweak: u128, new_modulus: u8) -> Self {
        Self::from_u128(self.hash2(other, tweak), new_modulus)
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use super::*;

    #[test]
    fn simple_packing() {
        let ref mut rng = Rng::new();
        for _ in 0..100 {
            let q = 2 + (rng.gen_byte() % 110);

            for i in 0..127 {
                let x = 1 << i;
                let w = Wire::from_u128(x, q);
                let z = w.as_u128();
                assert_eq!(z, x, "q={}, 2^{}", q, i);
            }
        }
    }

    #[test]
    fn packing() {
        let ref mut rng = Rng::new();
        for _ in 0..100 {
            let q = 2 + (rng.gen_byte() % 111);
            let w = rng.gen_u128();
            let x = Wire::from_u128(w, q);
            let y = x.as_u128();
            assert_eq!(w, y);
            let z = Wire::from_u128(y, q);
            assert_eq!(x, z);
        }
    }

    #[test]
    fn base_conversion_lookup_method() {
        let ref mut rng = Rng::new();
        for _ in 0..1000 {
            let q = 3 + (rng.gen_byte() % 110);
            let x = rng.gen_u128();
            let w = Wire::from_u128(x, q);
            let should_be = numbers::padded_base_q_128(x,q);
            assert_eq!(w.digits(), should_be, "x={} q={}", x, q);
        }
    }

    #[test]
    fn hash() {
        let mut rng = Rng::new();
        for _ in 0..100 {
            let q = 2 + (rng.gen_byte() % 110);
            let x = Wire::rand(&mut rng, q);
            let y = x.hashback(1, q);
            assert!(x != y);
            match y {
                Wire::Mod2 { val }    => assert!(val > 0),
                Wire::ModN { ds, .. } => assert!(!ds.iter().all(|&y| y == 0)),
            }
        }
    }

    #[test]
    fn negation() {
        let ref mut rng = Rng::new();
        for _ in 0..1000 {
            let q = rng.gen_byte();
            let x = Wire::rand(rng, q);
            let xneg = x.negate();
            println!("{:?}", xneg);
            let y = xneg.negate();
            assert_eq!(x, y);
        }
    }

    #[test]
    fn zero() {
        let mut rng = Rng::new();
        for _ in 0..1000 {
            let q = 3 + (rng.gen_byte() % 110);
            let z = Wire::zero(q);
            let ds = z.digits();
            assert_eq!(ds, vec![0;ds.len()], "q={}", q);
        }
    }

    #[test]
    fn subzero() {
        let mut rng = Rng::new();
        for _ in 0..1000 {
            let q = 2 + (rng.gen_byte() % 110);
            let x = Wire::rand(&mut rng, q);
            let z = Wire::zero(q);
            assert_eq!(x.minus(&x), z);
        }
    }

    #[test]
    fn pluszero() {
        let mut rng = Rng::new();
        for _ in 0..1000 {
            let q = 2 + (rng.gen_byte() % 110);
            let x = Wire::rand(&mut rng, q);
            assert_eq!(x.plus(&Wire::zero(q)), x);
        }
    }

    #[test]
    fn arithmetic() {
        let mut rng = Rng::new();
        for _ in 0..16 {
            let q = 2 + (rng.gen_byte() % 110);
            let x = Wire::rand(&mut rng, q);
            let y = Wire::rand(&mut rng, q);
            assert_eq!(x.cmul(0), Wire::zero(q));
            assert_eq!(x.cmul(q), Wire::zero(q));
            assert_eq!(x.plus(&x), x.cmul(2));
            assert_eq!(x.plus(&x).plus(&x), x.cmul(3));
            assert_eq!(x.negate().negate(), x);
            assert_eq!(x.plus(&x.negate()), Wire::zero(q));
            assert_eq!(x.minus(&y), x.plus(&y.negate()));
        }
    }
}
