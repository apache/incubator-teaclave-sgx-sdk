#[macro_use]
extern crate criterion;
extern crate fancy_garbling;

use criterion::Criterion;
use std::time::Duration;

use fancy_garbling::rand::Rng;
use fancy_garbling::garble::garble;
use fancy_garbling::circuit::Builder;

fn bench_projection_garble(c: &mut Criterion, q: u8) {
    c.bench_function(&format!("garbling::proj{}_gb", q), move |bench| {
        let mut tab = Vec::new();
        for i in 0..q {
            tab.push((i + 1) % q);
        }
        let mut b = Builder::new();
        let x = b.input(q);
        let _ = b.input(q);
        let z = b.proj(x, q, tab);
        b.output(z);
        let c = b.finish();

        bench.iter(|| {
            let (gb, _ev) = garble(&c);
            criterion::black_box(gb);
        });
    });
}

fn bench_projection_eval(c: &mut Criterion, q: u8) {
    c.bench_function(&format!("garbling::proj{}_ev", q), move |bench| {
        let ref mut rng = Rng::new();

        let mut tab = Vec::new();
        for i in 0..q {
            tab.push((i + 1) % q);
        }
        let mut b = Builder::new();
        let x = b.input(q);
        let _ = b.input(q);
        let z = b.proj(x, q, tab);
        b.output(z);
        let c = b.finish();

        let (gb, ev) = garble(&c);
        let x = rng.gen_byte() % q;
        let y = rng.gen_byte() % q;
        let xs = gb.encode(&[x,y]);

        bench.iter(|| {
            let ys = ev.eval(&c, &xs);
            criterion::black_box(ys);
        });
    });
}

fn proj17_gb(c: &mut Criterion) { bench_projection_garble(c,17) }
fn proj17_ev(c: &mut Criterion) { bench_projection_eval(c,17) }

criterion_group!{
    name = garbling;
    config = Criterion::default().warm_up_time(Duration::from_millis(100));
    targets = proj17_gb, proj17_ev
}

criterion_main!(garbling);
