use sha2::Sha256;
use digest::{FixedOutput, Input};
use group::{Affine, Jacobian};
use scalar::Scalar;
use ecmult::ECMultContext;

impl ECMultContext {
    pub fn ecdh_raw(&self, point: &Affine, scalar: &Scalar) -> Option<[u8; 32]> {
        let mut pt = point.clone();
        let s = scalar.clone();

        let mut result = [0u8; 32];
        if s.is_zero() {
            return None;
        }

        let mut res = Jacobian::default();
        self.ecmult_const(&mut res, &pt, &s);
        pt.set_gej(&res);

        pt.x.normalize();
        pt.y.normalize();

        let x = pt.x.b32();
        let y = 0x02 | (if pt.y.is_odd() { 1 } else { 0 });

        let mut sha = Sha256::default();
        sha.process(&[y]);
        sha.process(&x);
        let generic = sha.fixed_result();

        for i in 0..32 {
            result[i] = generic[i];
        }

        Some(result)
    }
}
