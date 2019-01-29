#[macro_use]
extern crate criterion;
extern crate fancy_garbling;

use criterion::Criterion;
use std::time::Duration;

use fancy_garbling::rand::Rng;
use fancy_garbling::garble::garble;
use fancy_garbling::high_level::Bundler;
use fancy_garbling::numbers::modulus_with_width;

fn bench_gb<F:'static>(cr: &mut Criterion, name: &str, gen_bundler: F) where F: Fn(u128) -> Bundler {
    cr.bench_function(name, move |bench| {
        let q = modulus_with_width(32);
        let c = gen_bundler(q).finish();
        bench.iter(|| {
            let (gb, _ev) = garble(&c);
            criterion::black_box(gb);
        });
    });
}

fn bench_ev<F:'static>(cr: &mut Criterion, name: &str, gen_bundler: F) where F: Fn(u128) -> Bundler{
    cr.bench_function(name, move |bench| {
        let q = modulus_with_width(32);
        let mut b = gen_bundler(q);
        let c = b.finish();

        let mut rng = Rng::new();
        let inps = (0..b.ninputs()).map(|_| rng.gen_u128() % q).collect::<Vec<_>>();
        let (gb, ev) = garble(&c);
        let enc_inp = b.encode(&inps);
        let xs = gb.encode(&enc_inp);

        bench.iter(|| {
            let ys = ev.eval(&c, &xs);
            criterion::black_box(ys);
        });
    });
}

fn add_bundler(q: u128) -> Bundler {
    let mut b = Bundler::new();
    let x = b.input(q);
    let y = b.input(q);
    let z = b.add(x,y);
    b.output(z);
    b
}

fn mul_bundler(q: u128) -> Bundler {
    let mut b = Bundler::new();
    let x = b.input(q);
    let y = b.input(q);
    let z = b.mul(x,y);
    b.output(z);
    b
}

fn mul_dlog_bundler(q: u128) -> Bundler {
    let mut b = Bundler::new();
    let x = b.input(q);
    let y = b.input(q);
    let z = b.mul_dlog(&[x,y]);
    b.output(z);
    b
}

fn parity_bundler(q: u128) -> Bundler {
    let mut b = Bundler::new();
    let x = b.input(q);
    let z = b.parity(x);
    b.output_ref(z);
    b
}

fn add(cr: &mut Criterion) {
    bench_gb(cr, "high_level::add_gb", add_bundler);
    bench_ev(cr, "high_level::add_ev", add_bundler);
}

fn mul(cr: &mut Criterion) {
    bench_gb(cr, "high_level::mul_gb", mul_bundler);
    bench_ev(cr, "high_level::mul_ev", mul_bundler);
}

fn mul_dlog(cr: &mut Criterion) {
    bench_gb(cr, "high_level::mul_dlog_gb", mul_dlog_bundler);
    bench_ev(cr, "high_level::mul_dlog_ev", mul_dlog_bundler);
}

fn parity(cr: &mut Criterion) {
    bench_gb(cr, "high_level::parity_gb", parity_bundler);
    bench_ev(cr, "high_level::parity_ev", parity_bundler);
}

criterion_group!{
    name = high_level;
    config = Criterion::default().warm_up_time(Duration::from_millis(100));
    targets = add, mul, mul_dlog, parity
}

criterion_main!(high_level);
