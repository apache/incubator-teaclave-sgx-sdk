use std::prelude::v1::*;
use circuit::{Builder, Circuit, Ref};
use numbers::{self, crt, inv, crt_inv, factor, product};
use std::rc::Rc;

#[derive(Clone, Copy)]
pub struct BundleRef(usize);

pub struct WireBundle {
    wires: Vec<Ref>,
    primes: Rc<Vec<u8>>,
}

pub struct Bundler {
    builder: Option<Builder>,
    bundles: Vec<WireBundle>,
    inputs: Vec<BundleRef>,
    outputs: Vec<BundleRef>,
}

#[allow(non_snake_case)]
impl Bundler {
    pub fn new() -> Self {
        Self::from_builder(Builder::new())
    }

    pub fn ninputs(&self) -> usize {
        self.inputs.len()
    }

    pub fn from_builder(b: Builder) -> Self {
        Bundler {
            builder: Some(b),
            bundles: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    fn add_bundle(&mut self, wires: Vec<Ref>, primes: Rc<Vec<u8>>) -> BundleRef {
        assert_eq!(wires.len(), primes.len());
        let bun_ref = self.bundles.len();
        let bun = WireBundle { wires, primes };
        self.bundles.push(bun);
        BundleRef(bun_ref)
    }

    pub fn input(&mut self, modulus: u128) -> BundleRef {
        let ps = factor(modulus);
        let mut ws = Vec::with_capacity(ps.len());
        for &p in &ps {
            ws.push(self.builder.as_mut().expect("need to own a builder!").input(p));
        }
        let bun_ref = self.add_bundle(ws, Rc::new(ps));
        self.inputs.push(bun_ref);
        bun_ref
    }

    pub fn output(&mut self, xref: BundleRef) {
        let b = self.builder.as_mut().expect("need a builder!");
        let ws = &self.bundles[xref.0].wires;
        for &x in ws {
            b.output(x);
        }
        self.outputs.push(xref);
    }

    pub fn output_ref(&mut self, xref: Ref) {
        self.borrow_mut_builder().output(xref);
    }

    pub fn output_refs(&mut self, xs: &[Ref]) {
        self.borrow_mut_builder().outputs(xs);
    }

    pub fn encode(&self, xs: &[u128]) -> Vec<u8> {
        let mut inps = Vec::new();
        for (&x, &xref) in xs.iter().zip(self.inputs.iter()) {
            inps.append(&mut crt(&self.bundles[xref.0].primes, x));
        }
        inps
    }

    pub fn decode(&self, outs: &[u8]) -> Vec<u128> {
        let mut outs = outs.to_vec();
        let mut res = Vec::with_capacity(self.outputs.len());
        for &zref in self.outputs.iter() {
            let z = &self.bundles[zref.0];
            let rest = outs.split_off(z.primes.len());
            res.push(crt_inv(&z.primes, &outs));
            outs = rest;
        }
        res
    }

    pub fn take_builder(&mut self) -> Builder {
        self.builder.take().expect("need to own a builder!")
    }

    pub fn put_builder(&mut self, b: Builder) {
        self.builder = Some(b);
    }

    pub fn borrow_mut_builder(&mut self) -> &mut Builder {
        self.builder.as_mut().expect("need to own a builder!")
    }

    pub fn borrow_builder(&self) -> &Builder {
        self.builder.as_ref().expect("need to own a builder!")
    }

    pub fn finish(&mut self) -> Circuit {
        self.take_builder().finish()
    }

    pub fn borrow_circ(&self) -> &Circuit {
        self.borrow_builder().borrow_circ()
    }

    pub fn add(&mut self, xref: BundleRef, yref: BundleRef) -> BundleRef {
        assert_eq!(self.bundles[xref.0].wires.len(), self.bundles[yref.0].wires.len());
        let mut zwires;
        {
            let xbun = &self.bundles[xref.0];
            let ybun = &self.bundles[yref.0];
            zwires = Vec::with_capacity(xbun.wires.len());
            let b = self.builder.as_mut().expect("need to own a builder!");
            for (&x, &y) in xbun.wires.iter().zip(ybun.wires.iter()) {
                zwires.push(b.add(x,y));
            }
        }
        let ps = self.bundles[xref.0].primes.clone();
        self.add_bundle(zwires, ps)
    }

    pub fn sub(&mut self, xref: BundleRef, yref: BundleRef) -> BundleRef {
        assert_eq!(self.bundles[xref.0].wires.len(), self.bundles[yref.0].wires.len());
        let mut zwires;
        {
            let xbun = &self.bundles[xref.0];
            let ybun = &self.bundles[yref.0];
            zwires = Vec::with_capacity(xbun.wires.len());
            let b = self.builder.as_mut().expect("need to own a builder!");
            for (&x, &y) in xbun.wires.iter().zip(ybun.wires.iter()) {
                zwires.push(b.sub(x,y));
            }
        }
        let ps = self.bundles[xref.0].primes.clone();
        self.add_bundle(zwires, ps)
    }

    pub fn cmul(&mut self, xref: BundleRef, c: u128) -> BundleRef {
        let mut zwires;
        {
            let xbun = &self.bundles[xref.0];
            zwires = Vec::with_capacity(xbun.wires.len());
            let cs = crt(&xbun.primes, c);
            for (&x, &c) in xbun.wires.iter().zip(cs.iter()) {
                zwires.push(self.builder.as_mut().expect("need a builder!").cmul(x,c));
            }
        }
        let ps = self.bundles[xref.0].primes.clone();
        self.add_bundle(zwires, ps)
    }

    pub fn cdiv(&mut self, xref: BundleRef, c: u8) -> BundleRef {
        let mut zwires;
        {
            let xbun = &self.bundles[xref.0];
            zwires = Vec::with_capacity(xbun.wires.len());
            for (&x, &p) in xbun.wires.iter().zip(xbun.primes.iter()) {
                if c % p == 0 {
                    zwires.push(self.builder.as_mut().expect("need a builder!").cmul(x,0));
                } else {
                    let d = inv(c as i16, p as i16) as u8;
                    zwires.push(self.builder.as_mut().expect("need a builder!").cmul(x,d));
                }
            }
        }
        let ps = self.bundles[xref.0].primes.clone();
        self.add_bundle(zwires, ps)
    }

    pub fn cexp(&mut self, xref: BundleRef, c: u8) -> BundleRef {
        assert!(c < 10); // to prevent overfolows
        let mut zwires;
        {
            let xbun = &self.bundles[xref.0];
            zwires = Vec::with_capacity(xbun.wires.len());
            let b = self.builder.as_mut().expect("need a builder!");
            for (&x, &p) in xbun.wires.iter().zip(xbun.primes.iter()) {
                let tab = (0..p).map(|x| {
                    ((x as u64).pow(c as u32) % p as u64) as u8
                }).collect();
                zwires.push(b.proj(x, p, tab));
            }
        }
        let ps = self.bundles[xref.0].primes.clone();
        self.add_bundle(zwires, ps)
    }

    pub fn rem(&mut self, xref: BundleRef, p: u8) -> BundleRef {
        let i = self.bundles[xref.0].primes.iter().position(|&q| p == q)
                    .expect("p is not one of the primes in this bundle!");
        let zwires;
        {
            let xbun = &self.bundles[xref.0];
            let x = xbun.wires[i];
            // zwires = Vec::with_capacity(xbun.wires.len());
            let b = self.builder.as_mut().expect("need a builder!");
            zwires = xbun.primes.iter().map(|&q| {
                b.mod_change(x, q)
            }).collect();
        }
        let ps = self.bundles[xref.0].primes.clone();
        self.add_bundle(zwires, ps)
    }

    pub fn mul_dlog(&mut self, args: &[BundleRef]) -> BundleRef {
        assert!(!args.is_empty());
        let nwires = self.bundles[args[0].0].wires.len();
        assert!(args.iter().all(|&a| self.bundles[a.0].wires.len() == nwires));
        let mut zwires;
        {
            zwires = Vec::with_capacity(self.bundles[args[0].0].wires.len());
            for i in 0..nwires {
                let ith_wires: Vec<Ref> = args.iter().map(|&x| {
                    self.bundles[x.0].wires[i]
                }).collect();
                let b = self.builder.as_mut().expect("need a builder!");
                zwires.push(b.mul_dlog(&ith_wires));
            }
        }
        let ps = self.bundles[args[0].0].primes.clone();
        self.add_bundle(zwires, ps)
    }

    pub fn mul(&mut self, xref: BundleRef, yref: BundleRef) -> BundleRef {
        let zwires;
        {
            let xbun = &self.bundles[xref.0];
            let ybun = &self.bundles[yref.0];
            let b = self.builder.as_mut().expect("need a builder!");
            zwires = xbun.wires.iter().zip(ybun.wires.iter()).map(|(&x,&y)| {
                b.half_gate(x,y)
            }).collect();
        }
        let ps = self.bundles[xref.0].primes.clone();
        self.add_bundle(zwires, ps)
    }

    pub fn eq(&mut self, xref: BundleRef, yref: BundleRef) -> Ref {
        let xbun = &self.bundles[xref.0];
        let ybun = &self.bundles[yref.0];
        let mut zs = Vec::with_capacity(xbun.wires.len());
        let b = self.builder.as_mut().expect("need a builder!");
        for i in 0..xbun.wires.len() {
            let z = b.sub(xbun.wires[i], ybun.wires[i]);
            let mut eq_zero_tab = vec![0; xbun.primes[i] as usize];
            eq_zero_tab[0] = 1;
            zs.push(b.proj(z, xbun.wires.len() as u8 + 1, eq_zero_tab));
        }
        b._and_many(&zs)
    }

    pub fn crt_to_pmr(&mut self, xref: BundleRef) -> BundleRef {
        let gadget_projection_tt = |p, q| -> Vec<u8> {
            let pq = p as u32 + q as u32 - 1;
            let mut tab = Vec::with_capacity(pq as usize);
            for z in 0 .. pq {
                let mut x = 0;
                let mut y = 0;
                'outer: for i in 0..p as u32 {
                    for j in 0..q as u32 {
                        if (i + pq - j) % pq == z {
                            x = i;
                            y = j;
                            break 'outer;
                        }
                    }
                }
                assert_eq!((x + pq - y) % pq, z);
                tab.push((((x * q as u32 * inv(q as i16, p as i16) as u32 +
                            y * p as u32 * inv(p as i16, q as i16) as u32) / p as u32) % q as u32) as u8);
            }
            tab
        };

        let gadget = |b: &mut Builder, x: Ref, y: Ref| -> Ref {
            let p  = b.circ.moduli[x];
            let q  = b.circ.moduli[y];
            let x_ = b.mod_change(x, p+q-1);
            let y_ = b.mod_change(y, p+q-1);
            let z  = b.sub(x_, y_);
            b.proj(z, q, gadget_projection_tt(p,q))
        };

        let n = self.bundles[xref.0].primes.len();
        let mut x = vec![vec![None; n+1]; n+1];

        for j in 0..n {
            x[0][j+1] = Some(self.bundles[xref.0].wires[j]);
        }

        for i in 1..=n {
            for j in i+1..=n {
                let b = self.builder.as_mut().expect("need a builder!");
                let z = gadget(b, x[i-1][i].unwrap(), x[i-1][j].unwrap());
                x[i][j] = Some(z);
            }
        }

        let mut zwires = Vec::with_capacity(n);
        for i in 0..n {
            zwires.push(x[i][i+1].unwrap());
        }
        let ps = self.bundles[xref.0].primes.clone();
        self.add_bundle(zwires, ps)
    }

    pub fn less_than_pmr(&mut self, xref: BundleRef, yref: BundleRef) -> Ref {
        let z = self.sub(xref, yref);
        let pmr = self.crt_to_pmr(z);
        let n = self.bundles[pmr.0].wires.len();
        let w = self.bundles[pmr.0].wires[n-1];
        let q_in = self.bundles[pmr.0].primes[n-1];
        let mut tab = vec![1; q_in as usize];
        tab[0] = 0;
        self.borrow_mut_builder().proj(w, 2, tab)
    }

    pub fn parity(&mut self, xref: BundleRef) -> Ref {
        let q = product(&self.bundles[xref.0].primes);
        let M = 2*q;

        // number of bits to keep in the projection
        let nbits = 5;

        // used to round
        let new_mod = (2 as u8).pow(nbits as u32);

        let project = |x: Ref, c: u8, b: &mut Builder| -> Ref {
            let p = b.circ.moduli[x];
            let Mi = M / p as u128;

            // crt coef
            let h = inv((Mi % p as u128) as i16, p as i16) as f32;

            let mut tab = Vec::with_capacity(p as usize);
            for x in 0..p {
                let y = ((x+c)%p) as f32 * h / p as f32;
                let truncated_y = (new_mod as f32 * y.fract()).round() as u8;
                tab.push(truncated_y);
            }

            b.proj(x, new_mod, tab)
        };

        let mut C = q/4;
        C += C % 2;
        let C_crt = crt(&self.bundles[xref.0].primes, C);

        let xs: Vec<Ref> = self.bundles[xref.0].wires.to_vec();

        let mut b = self.take_builder();
        let mut z = None;

        for (&x, &c) in xs.iter().zip(C_crt.iter()) {
            let y = project(x, c, &mut b);
            match z {
                None       => z = Some(y),
                Some(prev) => z = Some(b.add(prev,y)),
            }
        }

        let tab = (0..new_mod).map(|x| (x >= new_mod/2) as u8).collect();
        let out = b.proj(z.unwrap(), 2, tab);
        self.put_builder(b);
        out
    }

    pub fn bits(&mut self, xref: BundleRef, nbits: usize) -> Vec<Ref> {
        let mut bits = Vec::with_capacity(nbits as usize);
        let ps = self.bundles[xref.0].primes.clone();
        let mut x = xref;
        for _ in 0..nbits {
            let b = self.parity(x);
            bits.push(b);

            let wires = ps.iter().map(|&p| self.borrow_mut_builder().mod_change(b,p)).collect();
            let bs = self.add_bundle(wires, ps.clone());

            x = self.sub(x, bs);
            x = self.cdiv(x, 2);
        }
        bits
    }

    pub fn less_than_bits(&mut self, xref: BundleRef, yref: BundleRef, nbits: usize) -> Ref
    {
        let xbits = self.bits(xref, nbits);
        let ybits = self.bits(yref, nbits);
        self.borrow_mut_builder().binary_subtraction(&xbits, &ybits).1
    }

    pub fn sgn(&mut self, xref: BundleRef, ndigits: usize) -> Ref {
        let q = product(&self.bundles[xref.0].primes);

        let base = 4; // base of the addition in the gadget
        let M = (base as u128).pow(ndigits as u32);

        // gets the nbits of round(M*x*alpha/P) mod M
        let project = |x: Ref, b: &mut Builder| -> Vec<Ref> {
            let p = b.circ.moduli[x];
            let crt_coef = inv(((q / p as u128) % p as u128) as i32, p as i32);

            let mut tabs = vec![Vec::with_capacity(p as usize); ndigits];

            for x in 0..p {
                let y = (M as f32 * x as f32 * crt_coef as f32 / p as f32).round() as u128 % M;
                let ds = numbers::padded_base_q(y, base, ndigits);

                for i in 0..ndigits {
                    tabs[i].push(ds[i]);
                }
            }

            tabs.into_iter().map(|tt| b.proj(x, base, tt)).collect()
        };

        let mut b = self.take_builder();

        let xs: Vec<Ref> = self.bundles[xref.0].wires.to_vec();

        let init = project(xs[0], &mut b);

        let ds: Vec<Ref> = xs.into_iter().skip(1).fold(init, |acc, x| {
            let bs = project(x, &mut b);
            // b.binary_addition_no_carry(&bs, &acc)
            b.base_q_addition_no_carry(&bs, &acc)
        });

        // let z = *ds.last().unwrap();

        let tt = (0..base).map(|x| (x > base/2) as u8).collect();
        let z = b.proj(*ds.last().unwrap(), 2, tt);

        self.put_builder(b);
        z
    }
}

#[cfg(test)]
mod tests {
    use garble::garble;
    use high_level::Bundler;
    use numbers::{u128_to_bits, inv, factor, modulus_with_width};
    use rand::Rng;

    const NTESTS: usize = 1;

    // test harnesses {{{
    fn test_garbling(b: &Bundler, inp: &[u128], should_be: &[u128]) {
        let c = b.borrow_builder().borrow_circ();
        let (gb, ev) = garble(&c);
        println!("number of ciphertexts: {}", ev.size());
        let enc_inp = b.encode(inp);
        assert_eq!(b.decode(&c.eval(&enc_inp)), should_be);
        let xs = gb.encode(&enc_inp);
        let ys = ev.eval(c, &xs);
        assert_eq!(b.decode(&gb.decode(&ys)), should_be);
    }

    fn test_garbling_high_to_low(b: &Bundler, inp: &[u128], should_be: &[u8]) {
        let c = b.borrow_builder().borrow_circ();
        let (gb, ev) = garble(&c);
        println!("number of ciphertexts: {}", ev.size());
        let enc_inp = b.encode(inp);
        let pt_outs: Vec<u8> = c.eval(&enc_inp);
        assert_eq!(pt_outs, should_be);
        let xs = gb.encode(&enc_inp);
        let ys = ev.eval(c, &xs);
        let gb_outs: Vec<u8> = gb.decode(&ys);
        assert_eq!(gb_outs, should_be);
    }

    //}}}
    #[test] //input_output_equal {{{
    fn input_output_equal() {
        let mut rng = Rng::new();
        for _ in 0..NTESTS {
            let q = rng.gen_usable_composite_modulus();

            let mut b = Bundler::new();
            let inp = b.input(q);
            b.output(inp);

            let x = rng.gen_u128() % q;
            test_garbling(&mut b, &[x], &[x]);
        }
    }

    //}}}
    #[test] // addition {{{
    fn addition() {
        let mut rng = Rng::new();
        let q = rng.gen_usable_composite_modulus();

        let mut b = Bundler::new();
        let x = b.input(q);
        let y = b.input(q);
        let z = b.add(x,y);
        b.output(z);

        for _ in 0..NTESTS {
            let x = rng.gen_u128() % q;
            let y = rng.gen_u128() % q;
            test_garbling(&mut b, &[x,y], &[(x+y)%q]);
        }
    }
    //}}}
    #[test] // subtraction {{{
    fn subtraction() {
        let mut rng = Rng::new();

            let q = rng.gen_usable_composite_modulus();

            let mut b = Bundler::new();
            let x = b.input(q);
            let y = b.input(q);
            let z = b.sub(x,y);
            b.output(z);

        for _ in 0..NTESTS {
            let x = rng.gen_u128() % q;
            let y = rng.gen_u128() % q;
            test_garbling(&mut b, &[x,y], &[(x+q-y)%q]);
        }
    }
    //}}}
    #[test] // scalar_multiplication {{{
    fn scalar_multiplication() {
        let mut rng = Rng::new();
        let q = rng.gen_usable_composite_modulus();
        let y = rng.gen_u64() as u128 % q;

        let mut b = Bundler::new();
        let x = b.input(q);
        let z = b.cmul(x,y);
        b.output(z);

        for _ in 0..NTESTS {
            let x = rng.gen_u64() as u128 % q;
            let should_be = x * y % q;
            test_garbling(&mut b, &[x], &[should_be]);
        }
    }
    //}}}
    #[test] // scalar_exponentiation {{{
    fn scalar_exponentiation() {
        let mut rng = Rng::new();
        let q = rng.gen_usable_composite_modulus();
        let y = rng.gen_byte() % 10;

        let mut b = Bundler::new();
        let x = b.input(q);
        let z = b.cexp(x,y);
        b.output(z);

        for _ in 0..NTESTS {
            let x = rng.gen_byte() as u128 % q;
            let should_be = x.pow(y as u32) % q;
            test_garbling(&mut b, &[x], &[should_be]);
        }
    }
    // }}}
    #[test] // remainder {{{
    fn remainder() {
        let mut rng = Rng::new();
        let ps = rng._gen_usable_composite_modulus();
        let q = ps.iter().fold(1, |acc, &x| (x as u128) * acc);
        let p = ps[rng.gen_byte() as usize % ps.len()];

        let mut b = Bundler::new();
        let x = b.input(q);
        let z = b.rem(x,p);
        b.output(z);

        for _ in 0..NTESTS {
            let x = rng.gen_u128() % q;
            let should_be = x % p as u128;
            test_garbling(&mut b, &[x], &[should_be]);
        }
    }
    //}}}
    #[test] // dlog_multiplication {{{
    fn dlog_multiplication() {
        let mut rng = Rng::new();
        let q = modulus_with_width(32);

        let mut b = Bundler::new();
        let x = b.input(q);
        let y = b.input(q);
        let z = b.mul_dlog(&[x,y]);
        b.output(z);

        for _ in 0..NTESTS {
            let x = rng.gen_u64() as u128 % q;
            let y = rng.gen_u64() as u128 % q;
            let should_be = x * y % q;
            test_garbling(&mut b, &[x,y], &[should_be]);
        }
    }
    //}}}
    #[test] // half_gate_multiplication {{{
    fn half_gate_multiplication() {
        let mut rng = Rng::new();
        let q = modulus_with_width(32);

        let mut b = Bundler::new();
        let x = b.input(q);
        let y = b.input(q);
        let z = b.mul(x,y);
        b.output(z);

        for _ in 0..NTESTS {
            let x = rng.gen_u64() as u128 % q;
            let y = rng.gen_u64() as u128 % q;
            let should_be = x * y % q;
            test_garbling(&mut b, &[x,y], &[should_be]);
        }
    }
    //}}}
    #[test] // equality {{{
    fn equality() {
        let mut rng = Rng::new();
        let q = rng.gen_usable_composite_modulus();

        let mut b = Bundler::new();
        let x = b.input(q);
        let y = b.input(q);
        let z = b.eq(x,y);
        b.output_ref(z);

        for _ in 0..NTESTS {
            let x = rng.gen_u128() % q;
            let y = rng.gen_u128() % q;
            let should_be = (x == y) as u8;
            test_garbling_high_to_low(&mut b, &[x,y], &[should_be]);
        }
    }
    //}}}
    #[test] // parity {{{
    fn parity() {
        let mut rng = Rng::new();
        let q = modulus_with_width(32);
        let mut b = Bundler::new();
        let x = b.input(q);
        let z = b.parity(x);
        b.output_ref(z);

        for _ in 0..NTESTS {
            let pt = rng.gen_u128() % (q/2);
            let should_be = (pt % 2) as u8;
            test_garbling_high_to_low(&mut b, &[pt], &[should_be]);
        }
    }
    //}}}
    #[test] // cdiv {{{
    fn cdiv() {
        let mut rng = Rng::new();
        let q = modulus_with_width(32);
        let mut b = Bundler::new();
        let x = b.input(q);
        let z = b.cdiv(x,2);
        b.output(z);

        for _ in 0..NTESTS {
            let mut pt = rng.gen_u128() % (q/2);
            pt += pt % 2;
            let should_be = pt / 2;
            test_garbling(&mut b, &[pt], &[should_be]);
        }
    }
    //}}}
    #[test] // bits {{{
    fn bits() {
        let mut rng = Rng::new();
        let q = modulus_with_width(32);
        let mut b = Bundler::new();
        let x = b.input(q);
        let zs = b.bits(x, 32);
        b.output_refs(&zs);

        for _ in 0..NTESTS {
            let pt = rng.gen_u128() % (q/2);
            let should_be = u128_to_bits(pt, 32);
            test_garbling_high_to_low(&mut b, &[pt], &should_be);
        }
    }
    //}}}
    #[test] // less_than_pmr {{{
    fn less_than_pmr() {
        let mut rng = Rng::new();
        let q = modulus_with_width(32);
        let ps = factor(q);
        let n = ps.len();
        let p = q / ps[n-1] as u128;

        let mut b = Bundler::new();
        let x = b.input(q);
        let y = b.input(q);
        let z = b.less_than_pmr(x,y);
        b.output_ref(z);

        for _ in 0..NTESTS {
            let x = rng.gen_u128() % p;
            let y = rng.gen_u128() % p;
            let should_be = (x < y) as u8;
            test_garbling_high_to_low(&mut b, &[x,y], &[should_be]);
        }
    }
    //}}}
    #[test] // less_than_bits {{{
    fn less_than_bits() {
        let mut rng = Rng::new();
        let q = modulus_with_width(32);
        let mut b = Bundler::new();
        let x = b.input(q);
        let y = b.input(q);
        let z = b.less_than_bits(x, y, 32);
        b.output_ref(z);

        for _ in 0..NTESTS {
            let pt_x = rng.gen_u32() as u128;
            let pt_y = rng.gen_u32() as u128;
            let should_be = (pt_x < pt_y) as u8;
            println!("q={}", q);
            println!("{} {}", pt_x, pt_y);
            test_garbling_high_to_low(&mut b, &[pt_x, pt_y], &[should_be]);
        }
    }
    //}}}
    #[test] // sgn {{{
    fn sgn() {
        let mut rng = Rng::new();
        let q = modulus_with_width(32);
        let mut b = Bundler::new();
        let x = b.input(q);
        let z = b.sgn(x,7);
        b.output_ref(z);

        for _ in 0..NTESTS {
            let pt = rng.gen_u128() % q;
            let should_be = (pt > q/2) as u8;
            test_garbling_high_to_low(&mut b, &[pt], &[should_be]);
        }
    }
    //}}}
    #[test] // pmr {{{
    fn pmr() {
        let mut rng = Rng::new();
        for _ in 0..NTESTS {
            let ps = rng._gen_usable_composite_modulus();
            let q = ps.iter().fold(1, |acc, &x| x as u128 * acc);

            let mut b = Bundler::new();
            let x = b.input(q);
            let z = b.crt_to_pmr(x);
            b.output(z);

            let pt = rng.gen_u128() % q;

            let should_be = to_pmr_pt(pt, &ps);

            test_garbling_high_to_low(&mut b, &[pt], &should_be)
        }
    }
    fn to_pmr_pt(x: u128, ps: &[u8]) -> Vec<u8> {
        let mut ds = vec![0;ps.len()];
        let mut q = 1;
        for i in 0..ps.len() {
            let p = ps[i] as u128;
            ds[i] = ((x / q) % p) as u8;
            q *= p;
        }
        ds
    }

    fn from_pmr_pt(xs: &[u8], ps: &[u8]) -> u128 {
        let mut x = 0;
        let mut q = 1;
        for (&d,&p) in xs.iter().zip(ps.iter()) {
            x += d as u128 * q;
            q *= p as u128;
        }
        x
    }

    fn gadget_projection_tt(p: u8, q: u8) -> Vec<u8> {
        let pq = p as u32 + q as u32 - 1;
        let mut tab = Vec::with_capacity(pq as usize);
        for z in 0 .. pq {
            let mut x = 0;
            let mut y = 0;
            'outer: for i in 0..p as u32 {
                for j in 0..q as u32 {
                    if (i + pq - j) % pq == z {
                        x = i;
                        y = j;
                        break 'outer;
                    }
                }
            }
            assert_eq!((x + pq - y) % pq, z);
            tab.push((((x * q as u32 * inv(q as i16, p as i16) as u32 +
                        y * p as u32 * inv(p as i16, q as i16) as u32) / p as u32) % q as u32) as u8);
        }
        tab
    }

    pub fn to_pmr_alg(inp:u128, ps: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let gadget = |x: u8, p: u8, y: u8, q: u8| {
            let pq = p as u16 + q as u16 - 1;
            let x_ = x as u16 % pq;
            let y_ = y as u16 % pq;
            let z  = (x_ + pq - y_) % pq;
            (gadget_projection_tt(p,q)[z as usize], q)
                // ((z % q as u16) as u8, q)
        };

        let n = ps.len();
        let mut x = vec![vec![None; n+1]; n+1];

        let reduce = |x: u128, p: u8| { (x % p as u128) as u8 };

        for j in 0..n {
            x[0][j+1] = Some( (reduce(inp, ps[j]), ps[j]) );
        }

        for i in 1..n+1 {
            for j in i+1..n+1 {
                let (z,q) = gadget(x[i-1][i].unwrap().0, x[i-1][i].unwrap().1,
                                   x[i-1][j].unwrap().0, x[i-1][j].unwrap().1);
                x[i][j] = Some((z,q));
            }
        }

        let mut zs = Vec::with_capacity(n);
        let mut ps = Vec::with_capacity(n);
        for i in 0..n {
            zs.push(x[i][i+1].unwrap().0);
            ps.push(x[i][i+1].unwrap().1);
        }
        (zs, ps)
    }

    #[test]
    fn pmr_plaintext() {
        let mut rng = Rng::new();
        for _ in 0..NTESTS {
            let ps = rng._gen_usable_composite_modulus();
            let q = ps.iter().fold(1, |acc, &x| x as u128 * acc);
            let x = rng.gen_u128() % q;
            assert_eq!(x, from_pmr_pt(&to_pmr_pt(x, &ps), &ps));
            let (pmr, ps_) = to_pmr_alg(x, &ps);

            assert_eq!(ps, ps_);
            assert_eq!(to_pmr_pt(x, &ps), pmr);
        }
    }

    //}}}

}
