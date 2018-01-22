extern crate base64;
extern crate rand;
extern crate ring;

use self::base64::*;
use self::rand::{Rng, SeedableRng, XorShiftRng};
use self::rand::distributions::{IndependentSample, Range};
use self::ring::digest;

pub fn random_config(data: &[u8]) -> Config {
    // use sha256 of data as rng seed so it's repeatable
    let sha = digest::digest(&digest::SHA256, data);
    let sha_bytes = sha.as_ref();

    let mut seed = [0; 4];
    for seed_u32_index in 0..4 {
        seed[seed_u32_index] = (sha_bytes[seed_u32_index * 4 + 0] as u32) << 24 |
            (sha_bytes[seed_u32_index * 4 + 1] as u32) << 16 |
            (sha_bytes[seed_u32_index * 4 + 2] as u32) << 8 |
            (sha_bytes[seed_u32_index * 4 + 3] as u32)
    }

    let mut rng = XorShiftRng::from_seed(seed);
    let line_len_range = Range::new(10, 100);

    let (line_wrap, strip_whitespace) = if rng.gen() {
        (LineWrap::NoWrap, rng.gen())
    } else {
        let line_len = line_len_range.ind_sample(&mut rng);

        let line_ending = if rng.gen() {
            LineEnding::LF
        } else {
            LineEnding::CRLF
        };

        // always strip whttespace if we're wrapping
        (LineWrap::Wrap(line_len, line_ending), true)
    };

    let charset = if rng.gen() {
        CharacterSet::UrlSafe
    } else {
        CharacterSet::Standard
    };

    Config::new(charset, rng.gen(), strip_whitespace, line_wrap)
}
