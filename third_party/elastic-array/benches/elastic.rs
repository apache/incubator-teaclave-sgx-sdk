//! should be started with:
//! ```bash
//! rustup run nightly cargo bench
//! ```

#![feature(test)]
extern crate test;
extern crate rand;

extern crate elastic_array;

use test::Bencher;
use rand::random;

use elastic_array::ElasticArray1024;

const LEN: usize = 2048;

fn gen_data() -> [u8; LEN] {
	let mut arr = [0u8; LEN];
	for i in 0..LEN {
		arr[i] = random::<u8>();
	}
	arr
}

type BytesShort = ElasticArray1024<u8>;

pub struct BytesVec1024 {
	vec: Vec<u8>
}

impl BytesVec1024 {
	pub fn new() -> BytesVec1024 {
		BytesVec1024 {
			vec: vec![]
		}
	}

	pub fn push(&mut self, e: u8) {
		self.vec.push(e);
	}
}

#[bench]
fn bench_elastic_array(b: &mut Bencher) {
	let data = gen_data();
	b.iter(|| {
		let f = test::black_box(0);
		let n = test::black_box(LEN);
		let mut bytes = BytesShort::new();
		for i in f..n {
			bytes.push(data[i]);
		}
	});
}

#[bench]
fn bench_vector(b: &mut Bencher) {
	let data = gen_data();
	b.iter(|| {
		let f = test::black_box(0);
		let n = test::black_box(LEN);
		let mut v = BytesVec1024::new();
		for i in f..n {
			v.push(data[i]);
		}
	});
}
