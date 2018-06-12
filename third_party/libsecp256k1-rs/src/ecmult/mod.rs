use group::{Affine, Jacobian, AffineStorage, globalz_set_table_gej};
use field::Field;
use scalar::Scalar;

pub const WINDOW_A: usize = 5;
pub const WINDOW_G: usize = 16;
pub const ECMULT_TABLE_SIZE_A: usize = 1 << (WINDOW_A - 2);
pub const ECMULT_TABLE_SIZE_G: usize = 1 << (WINDOW_G - 2);
pub const WNAF_BITS: usize = 256;

/// Context for accelerating the computation of a*P + b*G.
pub struct ECMultContext {
    pre_g: [AffineStorage; ECMULT_TABLE_SIZE_G],
}

/// Context for accelerating the computation of a*G.
pub struct ECMultGenContext {
    prec: [[AffineStorage; 16]; 64],
    blind: Scalar,
    initial: Jacobian,
}

/// A static ECMult context.
pub static ECMULT_CONTEXT: ECMultContext = ECMultContext {
    pre_g: include!("const.rs"),
};

/// A static ECMultGen context.
pub static ECMULT_GEN_CONTEXT: ECMultGenContext = ECMultGenContext {
    prec: include!("const_gen.rs"),
    blind: Scalar([2217680822, 850875797, 1046150361, 1330484644,
                   4015777837, 2466086288, 2052467175, 2084507480]),
    initial: Jacobian {
        x: field_const_raw!(586608, 43357028, 207667908, 262670128, 142222828, 38529388, 267186148, 45417712, 115291924, 13447464),
        y: field_const_raw!(12696548, 208302564, 112025180, 191752716, 143238548, 145482948, 228906000, 69755164, 243572800, 210897016),
        z: field_const_raw!(3685368, 75404844, 20246216, 5748944, 73206666, 107661790, 110806176, 73488774, 5707384, 104448710),
        infinity: false,
    }
};

pub fn odd_multiples_table(prej: &mut [Jacobian],
                       zr: &mut [Field],
                       a: &Jacobian) {
    debug_assert!(prej.len() == zr.len());
    debug_assert!(prej.len() > 0);
    debug_assert!(!a.is_infinity());

    let d = a.double_var(None);
    let d_ge = Affine {
        x: d.x.clone(),
        y: d.y.clone(),
        infinity: false,
    };

    let mut a_ge = Affine::default();
    a_ge.set_gej_zinv(a, &d.z);
    prej[0].x = a_ge.x;
    prej[0].y = a_ge.y;
    prej[0].z = a.z.clone();
    prej[0].infinity = false;

    zr[0] = d.z.clone();
    for i in 1..prej.len() {
        prej[i] = prej[i-1].add_ge_var(&d_ge, Some(&mut zr[i]));
    }

    let l = &prej.last().unwrap().z * &d.z;
    prej.last_mut().unwrap().z = l;
}

fn odd_multiples_table_globalz_windowa(pre: &mut [Affine; ECMULT_TABLE_SIZE_A],
                                       globalz: &mut Field,
                                       a: &Jacobian) {
    let mut prej: [Jacobian; ECMULT_TABLE_SIZE_A] = Default::default();
    let mut zr: [Field; ECMULT_TABLE_SIZE_A] = Default::default();

    odd_multiples_table(&mut prej, &mut zr, a);
    globalz_set_table_gej(pre, globalz, &prej, &zr);
}

fn table_get_ge(r: &mut Affine, pre: &[Affine], n: i32, w: usize) {
    debug_assert!(n & 1 == 1);
    debug_assert!(n >= -((1 << (w-1)) - 1));
    debug_assert!(n <=  ((1 << (w-1)) - 1));
    if n > 0 {
        *r = pre[((n-1)/2) as usize].clone();
    } else {
        *r = pre[((-n-1)/2) as usize].neg();
    }
}

fn table_get_ge_const(r: &mut Affine, pre: &[Affine], n: i32, w: usize) {
    let abs_n = n * (if n > 0 { 1 } else { 0 } * 2 - 1);
    let idx_n = abs_n / 2;
    debug_assert!(n & 1 == 1);
    debug_assert!(n >= -((1 << (w-1)) - 1));
    debug_assert!(n <=  ((1 << (w-1)) - 1));
    for m in 0..pre.len() {
        r.x.cmov(&pre[m].x, m == idx_n as usize);
        r.y.cmov(&pre[m].y, m == idx_n as usize);
    }
    r.infinity = false;
    let neg_y = r.y.neg(1);
    r.y.cmov(&neg_y, n != abs_n);
}

fn table_get_ge_storage(r: &mut Affine, pre: &[AffineStorage], n: i32, w: usize) {
    debug_assert!(n & 1 == 1);
    debug_assert!(n >= -((1 << (w-1)) - 1));
    debug_assert!(n <=  ((1 << (w-1)) - 1));
    if n > 0 {
        *r = pre[((n-1)/2) as usize].clone().into();
    } else {
        *r = pre[((-n-1)/2) as usize].clone().into();
        *r = r.neg();
    }
}

pub fn ecmult_wnaf(wnaf: &mut [i32], a: &Scalar, w: usize) -> i32 {
    let mut s = a.clone();
    let mut last_set_bit: i32 = -1;
    let mut bit = 0;
    let mut sign = 1;
    let mut carry = 0;

    debug_assert!(wnaf.len() <= 256);
    debug_assert!(w >= 2 && w <= 31);

    for i in 0..wnaf.len() {
        wnaf[i] = 0;
    }

    if s.bits(255, 1) > 0 {
        s = s.neg();
        sign = -1;
    }

    while bit < wnaf.len() {
        let mut now;
        let mut word;
        if s.bits(bit, 1) == carry as u32 {
            bit += 1;
            continue;
        }

        now = w;
        if now > wnaf.len() - bit {
            now = wnaf.len() - bit;
        }

        word = (s.bits_var(bit, now) as i32) + carry;

        carry = (word >> (w-1)) & 1;
        word -= carry << w;

        wnaf[bit] = sign * word;
        last_set_bit = bit as i32;

        bit += now;
    }
    debug_assert!(carry == 0);
    debug_assert!({
        let mut t = true;
        while bit < 256 {
            t = t && (s.bits(bit, 1) == 0);
            bit += 1;
        }
        t
    });
    last_set_bit + 1
}

pub fn ecmult_wnaf_const(wnaf: &mut [i32], a: &Scalar, w: usize) -> i32 {
    let mut s = a.clone();
    let mut word = 0;

    /* Note that we cannot handle even numbers by negating them to be
     * odd, as is done in other implementations, since if our scalars
     * were specified to have width < 256 for performance reasons,
     * their negations would have width 256 and we'd lose any
     * performance benefit. Instead, we use a technique from Section
     * 4.2 of the Okeya/Tagaki paper, which is to add either 1 (for
     * even) or 2 (for odd) to the number we are encoding, returning a
     * skew value indicating this, and having the caller compensate
     * after doing the multiplication. */

    /* Negative numbers will be negated to keep their bit
     * representation below the maximum width */
    let flip = s.is_high();
    /* We add 1 to even numbers, 2 to odd ones, noting that negation
     * flips parity */
    let bit = flip ^ !s.is_even();
    /* We add 1 to even numbers, 2 to odd ones, noting that negation
     * flips parity */
    let neg_s = s.neg();
    let not_neg_one = !neg_s.is_one();
    s.cadd_bit(if bit { 1 } else { 0 }, not_neg_one);
    /* If we had negative one, flip == 1, s.d[0] == 0, bit == 1, so
     * caller expects that we added two to it and flipped it. In fact
     * for -1 these operations are identical. We only flipped, but
     * since skewing is required (in the sense that the skew must be 1
     * or 2, never zero) and flipping is not, we need to change our
     * flags to claim that we only skewed. */
    let mut global_sign = s.cond_neg_mut(flip);
    global_sign *= if not_neg_one { 1 } else { 0 } * 2 - 1;
    let skew = 1 << (if bit { 1 } else { 0 });

    let mut u_last: i32 = s.shr_int(w) as i32;
    let mut u: i32 = 0;
    while word * w < WNAF_BITS {
        u = s.shr_int(w) as i32;
        let even = (u & 1) == 0;
        let sign = 2 * (if u_last > 0 { 1 } else { 0 }) - 1;
        u += sign * if even { 1 } else { 0 };
        u_last -= sign * if even { 1 } else { 0 } * (1 << w);

        wnaf[word] = (u_last as i32 * global_sign as i32) as i32;
        word += 1;

        u_last = u;
    }
    wnaf[word] = u * global_sign as i32;

    debug_assert!(s.is_zero());
    let wnaf_size = (WNAF_BITS + w - 1) / w;
    debug_assert!(word == wnaf_size);

    skew
}

impl ECMultContext {
    pub fn ecmult(
        &self, r: &mut Jacobian, a: &Jacobian, na: &Scalar, ng: &Scalar
    ) {
        let mut tmpa = Affine::default();
        let mut pre_a: [Affine; ECMULT_TABLE_SIZE_A] = Default::default();
        let mut z = Field::default();
        let mut wnaf_na = [0i32; 256];
        let mut wnaf_ng = [0i32; 256];
        let bits_na = ecmult_wnaf(&mut wnaf_na, na, WINDOW_A);
        let mut bits = bits_na;
        odd_multiples_table_globalz_windowa(&mut pre_a, &mut z, a);

        let bits_ng = ecmult_wnaf(&mut wnaf_ng, &ng, WINDOW_G);
        if bits_ng > bits {
            bits = bits_ng;
        }

        r.set_infinity();
        for i in (0..bits).rev() {
            let mut n;
            *r = r.double_var(None);

            n = wnaf_na[i as usize];
            if i < bits_na && n != 0 {
                table_get_ge(&mut tmpa, &pre_a, n, WINDOW_A);
                *r = r.add_ge_var(&tmpa, None);
            }
            n = wnaf_ng[i as usize];
            if i < bits_ng && n != 0 {
                table_get_ge_storage(&mut tmpa, &self.pre_g, n, WINDOW_G);
                *r = r.add_zinv_var(&tmpa, &z);
            }
        }

        if !r.is_infinity() {
            r.z *= &z;
        }
    }

    pub fn ecmult_const(
        &self, r: &mut Jacobian, a: &Affine, scalar: &Scalar
    ) {
        const WNAF_SIZE: usize = (WNAF_BITS + (WINDOW_A - 1) - 1) / (WINDOW_A - 1);

        let mut tmpa = Affine::default();
        let mut pre_a: [Affine; ECMULT_TABLE_SIZE_A] = Default::default();
        let mut z = Field::default();

        let mut wnaf_1 = [0i32; 1 + WNAF_SIZE];

        let sc = scalar.clone();
        let skew_1 = ecmult_wnaf_const(&mut wnaf_1, &sc, WINDOW_A - 1);

        /* Calculate odd multiples of a.  All multiples are brought to
         * the same Z 'denominator', which is stored in Z. Due to
         * secp256k1' isomorphism we can do all operations pretending
         * that the Z coordinate was 1, use affine addition formulae,
         * and correct the Z coordinate of the result once at the end.
         */
        r.set_ge(a);
        odd_multiples_table_globalz_windowa(&mut pre_a, &mut z, r);
        for i in 0..ECMULT_TABLE_SIZE_A {
            pre_a[i].y.normalize_weak();
        }

        /* first loop iteration (separated out so we can directly set
         * r, rather than having it start at infinity, get doubled
         * several times, then have its new value added to it) */
        let i = wnaf_1[WNAF_SIZE];
        debug_assert!(i != 0);
        table_get_ge_const(&mut tmpa, &pre_a, i, WINDOW_A);
        r.set_ge(&tmpa);

        /* remaining loop iterations */
        for i in (0..WNAF_SIZE).rev() {
            for _ in 0..(WINDOW_A - 1) {
                let r2 = r.clone();
                r.double_nonzero_in_place(&r2, None);
            }

            let n = wnaf_1[i];
            table_get_ge_const(&mut tmpa, &pre_a, n, WINDOW_A);
            debug_assert!(n != 0);
            *r = r.add_ge(&tmpa);
        }

        r.z *= &z;

        /* Correct for wNAF skew */
        let mut correction = a.clone();
        let mut correction_1_stor: AffineStorage;
        let a2_stor: AffineStorage;
        let mut tmpj = Jacobian::default();
        tmpj.set_ge(&correction);
        tmpj = tmpj.double_var(None);
        correction.set_gej(&tmpj);
        correction_1_stor = a.clone().into();
        a2_stor = correction.into();

        /* For odd numbers this is 2a (so replace it), for even ones a (so no-op) */
        correction_1_stor.cmov(&a2_stor, skew_1 == 2);

        /* Apply the correction */
        correction = correction_1_stor.into();
        correction = correction.neg();
        *r = r.add_ge(&correction)
    }
}

impl ECMultGenContext {
    pub fn ecmult_gen(
        &self, r: &mut Jacobian, gn: &Scalar
    ) {
        let mut adds = AffineStorage::default();
        *r = self.initial.clone();

        let mut gnb = gn + &self.blind;
        let mut add = Affine::default();
        add.infinity = false;

        for j in 0..64 {
            let mut bits = gnb.bits(j * 4, 4);
            for i in 0..16 {
                adds.cmov(&self.prec[j][i], i as u32 == bits);
            }
            add = adds.clone().into();
            *r = r.add_ge(&add);
            #[allow(unused_assignments)]
            {
                bits = 0;
            }
        }
        add.clear();
        gnb.clear();
    }
}
