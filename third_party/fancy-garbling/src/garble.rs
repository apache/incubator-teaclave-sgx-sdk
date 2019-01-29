use std::prelude::v1::*;
use circuit::{Circuit, Gate};
use rand::Rng;
use wire::Wire;

use std::collections::HashMap;

type GarbledGate = Vec<Wire>;

pub struct Garbler {
    deltas     : HashMap<u8, Wire>,
    inputs     : Vec<Wire>,
    outputs    : Vec<Vec<u128>>,
    rng        : Rng,
}

pub struct Evaluator {
    gates : Vec<GarbledGate>,
}

#[allow(non_snake_case)]
pub fn garble(c: &Circuit) -> (Garbler, Evaluator) {
    let mut gb = Garbler::new();

    let mut wires: Vec<Wire> = Vec::new();
    let mut gates: Vec<GarbledGate> = Vec::new();
    for i in 0..c.gates.len() {
        let q = c.moduli[i];
        let w = match c.gates[i] {
            Gate::Input { .. } => gb.input(q),

            Gate::Add { xref, yref } => wires[xref].plus(&wires[yref]),
            Gate::Sub { xref, yref } => wires[xref].minus(&wires[yref]),
            Gate::Cmul { xref, c }   => wires[xref].cmul(c),

            Gate::Proj { xref, ref tt, .. }  => {
                let X = wires[xref].clone();
                let (w,g) = gb.proj(&X, q, tt, i);
                gates.push(g);
                w
            }

            Gate::Yao { xref, yref, ref tt, .. } => {
                let X = wires[xref].clone();
                let Y = wires[yref].clone();
                let (w,g) = gb.yao(&X, &Y, q, tt, i);
                gates.push(g);
                w
            }
            Gate::HalfGate { xref, yref, .. }  => {
                let X = wires[xref].clone();
                let Y = wires[yref].clone();
                let (w,g) = gb.half_gate(&X, &Y, q, i);
                gates.push(g);
                w
            }
        };
        wires.push(w); // add the new zero-wire
    }
    for (i, &r) in c.output_refs.iter().enumerate() {
        let X = wires[r].clone();
        gb.output(&X, i);
    }

    (gb, Evaluator::new(gates))
}

#[allow(non_snake_case)]
impl Garbler {
    pub fn new() -> Self {
        Garbler {
            deltas: HashMap::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            rng: Rng::new(),
        }
    }

    fn delta(&mut self, q: u8) -> Wire {
        if !self.deltas.contains_key(&q) {
            let w = Wire::rand_delta(&mut self.rng, q);
            self.deltas.insert(q, w.clone());
            w
        } else {
            self.deltas[&q].clone()
        }
    }

    pub fn input(&mut self, q: u8) -> Wire {
        let w = Wire::rand(&mut self.rng, q);
        self.inputs.push(w.clone());
        w
    }

    pub fn output(&mut self, X: &Wire, output_num: usize) {
        let mut cts = Vec::new();
        let q = X.modulus();
        let D = &self.delta(q);
        for k in 0..q {
            let t = output_tweak(output_num, k);
            cts.push(X.plus(&D.cmul(k)).hash(t));
        }
        self.outputs.push(cts);
    }

    pub fn proj(&mut self, A: &Wire, q_out: u8, tt: &[u8], gate_num: usize)
        -> (Wire, GarbledGate)
    {
        let q_in = A.modulus();
        // we have to fill in the vector in an unkonwn order because of the
        // color bits. Since some of the values in gate will be void
        // temporarily, we use Vec<Option<..>>
        let mut gate = vec![None; q_in as usize - 1];

        let tao = A.color();        // input zero-wire
        let g = tweak(gate_num);    // gate tweak

        // output zero-wire
        // W_g^0 <- -H(g, W_{a_1}^0 - \tao\Delta_m) - \phi(-\tao)\Delta_n
        let C = A.minus(&self.delta(q_in).cmul(tao))
                 .hashback(g, q_out)
                 .negate()
                 .minus(&self.delta(q_out).cmul(tt[((q_in - tao) % q_in) as usize]));

        for x in 0..q_in {
            let ix = (tao as usize + x as usize) % q_in as usize;
            if ix == 0 { continue }
            let A_ = A.plus(&self.delta(q_in).cmul(x));
            let C_ = C.plus(&self.delta(q_out).cmul(tt[x as usize]));
            let ct = A_.hashback(g, q_out).plus(&C_);
            gate[ix-1] = Some(ct);
        }

        // unwrap the Option elems inside the Vec
        let gate = gate.into_iter().map(Option::unwrap).collect();
        (C, gate)
    }

    fn yao(&mut self, A: &Wire, B: &Wire, q: u8, tt: &[Vec<u8>], gate_num: usize)
        -> (Wire, GarbledGate)
    {
        let xmod = A.modulus() as usize;
        let ymod = B.modulus() as usize;
        let mut gate = vec![None; xmod * ymod - 1];
        // gate tweak
        let g = tweak(gate_num);
        // sigma is the output truth value of the 0,0-colored wirelabels
        let sigma = tt[((xmod - A.color() as usize) % xmod) as usize]
                      [((ymod - B.color() as usize) % ymod) as usize];
        // we use the row reduction trick here
        let B_delta = self.delta(ymod as u8);
        let C = A.minus(&self.delta(xmod as u8).cmul(A.color()))
                 .hashback2(&B.minus(&B_delta.cmul(B.color())), g, q)
                 .negate()
                 .minus(&self.delta(q).cmul(sigma));
        for x in 0..xmod {
            let A_ = A.plus(&self.delta(xmod as u8).cmul(x as u8));
            for y in 0..ymod {
                let ix = ((A.color() as usize + x) % xmod) * ymod +
                         ((B.color() as usize + y) % ymod);
                if ix == 0 { continue }
                assert_eq!(gate[ix-1], None);
                let B_ = B.plus(&self.delta(ymod as u8).cmul(y as u8));
                let C_ = C.plus(&self.delta(q).cmul(tt[x][y]));
                let ct = A_.hashback2(&B_,g, q).plus(&C_);
                gate[ix-1] = Some(ct);
            }
        }
        let gate = gate.into_iter().map(Option::unwrap).collect();
        (C, gate)
    }

    pub fn half_gate(&mut self, A: &Wire, B: &Wire, q: u8, gate_num: usize)
        -> (Wire, GarbledGate)
    {
        let mut gate = vec![None; 2 * q as usize - 2];
        let g = tweak(gate_num);

        let r = B.color(); // secret value known only to the garbler (ev knows r+b)

        let D = &self.delta(q); // delta for this modulus

        // X = -H(A+aD) - arD such that a + A.color == 0
        let alpha = q - A.color(); // alpha = -A.color
        let X = A.plus(&D.cmul(alpha)).hashback(g,q).negate()
                 .plus(&D.cmul((alpha as u16 * r as u16 % q as u16) as u8));

        // Y = -H(B + bD) + brA
        let beta = q - B.color();
        let Y = B.plus(&D.cmul(beta)).hashback(g,q).negate()
                 .plus(&A.cmul((beta + r) % q));

        for i in 0..q {
            // garbler's half-gate: outputs X-arD
            // G = H(A+aD) + X+a(-r)D = H(A+aD) + X-arD
            let a = i; // a: truth value of wire X
            let A_ = A.plus(&self.delta(q).cmul(a));
            if A_.color() != 0 {
                let tao = (a as u16 * (q - r) as u16 % q as u16) as u8;
                let G = A_.hashback(g,q).plus(&X.plus(&D.cmul(tao)));
                gate[A_.color() as usize - 1] = Some(G);
            }

            // evaluator's half-gate: outputs Y+a(r+b)D
            // G = H(B+bD) + Y-(b+r)A
            let B_ = B.plus(&D.cmul(i));
            if B_.color() != 0 {
                let G = B_.hashback(g,q).plus(&Y.minus(&A.cmul((i+r)%q)));
                gate[(q + B_.color()) as usize - 2] = Some(G);
            }
        }
        let gate = gate.into_iter().map(Option::unwrap).collect();
        (X.plus(&Y), gate) // output zero wire
    }

    pub fn encode(&self, inputs: &[u8]) -> Vec<Wire> {
        assert_eq!(inputs.len(), self.inputs.len());
        let mut xs = Vec::new();
        for i in 0..inputs.len() {
            let x = inputs[i];
            let X = self.inputs[i].clone();
            let D = self.deltas[&X.modulus()].clone();
            xs.push(X.plus(&D.cmul(x)));
        }
        xs
    }

    pub fn decode(&self, ws: &[Wire]) -> Vec<u8> {
        assert_eq!(ws.len(), self.outputs.len());
        let mut outs = Vec::new();
        for i in 0..ws.len() {
            let q = ws[i].modulus();
            for k in 0..q {
                let h = ws[i].hash(output_tweak(i,k));
                if h == self.outputs[i][k as usize] {
                    outs.push(k);
                    break;
                }
            }
        }
        assert_eq!(ws.len(), outs.len());
        outs
    }
}

#[allow(non_snake_case)]
impl Evaluator {
    pub fn new(gates: Vec<GarbledGate>) -> Self {
        Evaluator { gates }
    }

    pub fn size(&self) -> usize {
        let mut c = 0;
        for g in self.gates.iter() {
            c += g.len();
        }
        c
    }

    pub fn eval(&self, c: &Circuit, inputs: &[Wire]) -> Vec<Wire> {
        let mut wires: Vec<Wire> = Vec::new();
        for i in 0..c.gates.len() {
            let q = c.moduli[i];
            let w = match c.gates[i] {

                Gate::Input { id }       => inputs[id].clone(),
                Gate::Add { xref, yref } => wires[xref].plus(&wires[yref]),
                Gate::Sub { xref, yref } => wires[xref].minus(&wires[yref]),
                Gate::Cmul { xref, c }   => wires[xref].cmul(c),

                Gate::Proj { xref, id, .. } => {
                    let x = &wires[xref];
                    if x.color() == 0 {
                        x.hashback(i as u128, q).negate()
                    } else {
                        let ct = &self.gates[id][x.color() as usize - 1];
                        ct.minus(&x.hashback(i as u128, q))
                    }
                }

                Gate::Yao { xref, yref, id, .. } => {
                    let a = &wires[xref];
                    let b = &wires[yref];
                    if a.color() == 0 && b.color() == 0 {
                        a.hashback2(&b, tweak(i), q).negate()
                    } else {
                        let ix = a.color() as usize * c.moduli[yref] as usize + b.color() as usize;
                        let ct = &self.gates[id][ix - 1];
                        ct.minus(&a.hashback2(&b, tweak(i), q))
                    }
                }

                Gate::HalfGate { xref, yref, id } => {
                    let g = tweak(i);

                    // garbler's half gate
                    let A = &wires[xref];
                    let L = if A.color() == 0 {
                        A.hashback(g,q).negate()
                    } else {
                        let ct_left = &self.gates[id][A.color() as usize - 1];
                        ct_left.minus(&A.hashback(g,q))
                    };

                    // evaluator's half gate
                    let B = &wires[yref];
                    let R = if B.color() == 0 {
                        B.hashback(g,q).negate()
                    } else {
                        let ct_right = &self.gates[id][(q + B.color()) as usize - 2];
                        ct_right.minus(&B.hashback(g,q))

                    };
                    L.plus(&R.plus(&A.cmul(B.color())))
                }
            };
            wires.push(w);
        }

        c.output_refs.iter().map(|&r| {
            wires[r].clone()
        }).collect()
    }
}

fn tweak(i: usize) -> u128 {
    i as u128
}

fn output_tweak(i: usize, k: u8) -> u128 {
    let (left, _) = (i as u128).overflowing_shl(64);
    left + k as u128
}


#[cfg(test)]
mod tests {
    use circuit::{Circuit, Builder};
    use garble::garble;
    use rand::Rng;

    // helper {{{
    fn garble_test_helper<F>(f: F)
        where F: Fn(u8) -> Circuit
    {
        let mut rng = Rng::new();
        for _ in 0..16 {
            let q = rng.gen_prime();
            let c = &f(q);
            let (gb, ev) = garble(&c);
            println!("number of ciphertexts for mod {}: {}", q, ev.size());
            for _ in 0..64 {
                let inps = &(0..c.ninputs()).map(|i| {
                    rng.gen_byte() % c.input_mod(i)
                }).collect::<Vec<u8>>();
                let xs = &gb.encode(inps);
                let ys = &ev.eval(c, xs);
                assert_eq!(gb.decode(ys)[0], c.eval(inps)[0], "q={}", q);
            }
        }
    }
//}}}
    #[test] // add {{{
    fn add() {
        garble_test_helper(|q| {
            let mut b = Builder::new();
            let x = b.input(q);
            let y = b.input(q);
            let z = b.add(x,y);
            b.output(z);
            b.finish()
        });
    }
//}}}
    #[test] // add_many {{{
    fn add_many() {
        garble_test_helper(|q| {
            let mut b = Builder::new();
            let xs = b.inputs(16, q);
            let z = b.add_many(&xs);
            b.output(z);
            b.finish()
        });
    }
//}}}
    #[test] // or_many {{{
    fn or_many() {
        garble_test_helper(|_| {
            let mut b = Builder::new();
            let xs = b.inputs(16, 2);
            let z = b.or_many(&xs);
            b.output(z);
            b.finish()
        });
    }
//}}}
    #[test] // and_many {{{
    fn and_many() {
        garble_test_helper(|_| {
            let mut b = Builder::new();
            let xs = b.inputs(16, 2);
            let z = b.and_many(&xs);
            b.output(z);
            b.finish()
        });
    }
//}}}
    #[test] // sub {{{
    fn sub() {
        garble_test_helper(|q| {
            let mut b = Builder::new();
            let x = b.input(q);
            let y = b.input(q);
            let z = b.sub(x,y);
            b.output(z);
            b.finish()
        });
    }
//}}}
    #[test] // cmul {{{
    fn cmul() {
        garble_test_helper(|q| {
            let mut b = Builder::new();
            let x = b.input(q);
            let _ = b.input(q);
            let z = b.cmul(x, 2);
            b.output(z);
            b.finish()
        });
    }
//}}}
    #[test] // proj_cycle {{{
    fn proj_cycle() {
        garble_test_helper(|q| {
            let mut tab = Vec::new();
            for i in 0..q {
                tab.push((i + 1) % q);
            }
            let mut b = Builder::new();
            let x = b.input(q);
            let _ = b.input(q);
            let z = b.proj(x, q, tab);
            b.output(z);
            b.finish()
        });
    }
//}}}
    #[test] // proj_rand {{{
    fn proj_rand() {
        garble_test_helper(|q| {
            let mut rng = Rng::new();
            let mut tab = Vec::new();
            for _ in 0..q {
                tab.push(rng.gen_byte() % q);
            }
            let mut b = Builder::new();
            let x = b.input(q);
            let _ = b.input(q);
            let z = b.proj(x, q, tab);
            b.output(z);
            b.finish()
        });
    }
//}}}
    #[test] // mod_change {{{
    fn mod_change() {
        garble_test_helper(|q| {
            let mut b = Builder::new();
            let x = b.input(q);
            let z = b.mod_change(x,q*2);
            b.output(z);
            b.finish()
        });
    }
//}}}
    #[test] // yao {{{
    fn yao() {
        garble_test_helper(|q| {
            let mut b = Builder::new();
            let x = b.input(q);
            let y = b.input(q);
            let mut tt = Vec::new();
            for a in 0..q {
                let mut tt_ = Vec::new();
                for b in 0..q {
                    tt_.push((a as u16 * b as u16 % q as u16) as u8);
                }
                tt.push(tt_);
            }
            let z = b.yao(x, y, q, tt);
            b.output(z);
            b.finish()
        });
    }
//}}}
    #[test] // mul_dlog {{{
    fn mul_dlog() {
        garble_test_helper(|q| {
            let mut b = Builder::new();
            let x = b.input(q);
            let y = b.input(q);
            let z = b.mul_dlog(&[x,y]);
            b.output(z);
            b.finish()
        });
    }
//}}}
    #[test] // half_gate {{{
    fn half_gate() {
        garble_test_helper(|q| {
            let mut b = Builder::new();
            let x = b.input(q);
            let y = b.input(q);
            let z = b.half_gate(x,y);
            b.output(z);
            b.finish()
        });
    }
//}}}
    #[test] // base_4_addition_no_carry {{{
    fn base_q_addition_no_carry() {
        garble_test_helper(|q| {
            let mut b = Builder::new();
            let n = 16;
            let xs = b.inputs(n,q);
            let ys = b.inputs(n,q);
            let zs = b.base_q_addition_no_carry(&xs, &ys);
            b.outputs(&zs);
            b.finish()
        });
    }
//}}}
}
