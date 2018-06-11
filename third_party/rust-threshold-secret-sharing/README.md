# Threshold Secret Sharing

[![Build Status](https://travis-ci.org/snipsco/rust-threshold-secret-sharing.svg?branch=master)](https://travis-ci.org/snipsco/rust-threshold-secret-sharing)
[![Latest version](https://img.shields.io/crates/v/threshold-secret-sharing.svg)](https://img.shields.io/crates/v/threshold-secret-sharing.svg)
[![License: MIT/Apache2](https://img.shields.io/badge/license-MIT%2fApache2-blue.svg)](https://img.shields.io/badge/license-MIT%2fApache2-blue.svg)

Efficient pure-Rust library for [secret sharing](https://en.wikipedia.org/wiki/Secret_sharing), offering efficient share generation and reconstruction for both traditional Shamir sharing and packet sharing. For now, secrets and shares are fixed as prime field elements represented by `i64` values.


# Installation


## Cargo
```toml
[dependencies]
threshold-secret-sharing = "0.2"
```


## GitHub
```bash
git clone https://github.com/snipsco/rust-threshold-secret-sharing
cd rust-threshold-secret-sharing
cargo build --release
```


# Examples
Several examples are included in the `examples/` directory. Run each with `cargo` using e.g.
```sh
cargo run --example shamir
```
for the Shamir example below.


## Shamir sharing
Using the Shamir scheme is relatively straight-forward.

When choosing parameters, `threshold` and `share_count` must be chosen to satisfy security requirements, and `prime` must be large enough to correctly encode the value to be shared (and such that `prime >= share_count + 1`).

When reconstructing the secret, indices must be explicitly provided to identify the shares; these correspond to the indices the shares had in the vector returned by `share()`.

```rust
extern crate threshold_secret_sharing as tss;

fn main() {
  // create instance of the Shamir scheme
  let ref tss = tss::shamir::ShamirSecretSharing {
    threshold: 8,           // privacy threshold
    share_count: 20,        // total number of shares to generate
    prime: 41               // prime field to use
  };

  let secret = 5;

  // generate shares for secret
  let all_shares = tss.share(secret);

  // artificially remove some of the shares
  let number_of_recovered_shared = 10;
  assert!(number_of_recovered_shared >= tss.reconstruct_limit());
  let recovered_indices: Vec<usize> = (0..number_of_recovered_shared).collect();
  let recovered_shares: &[i64] = &all_shares[0..number_of_recovered_shared];

  // reconstruct using remaining subset of shares
  let reconstructed_secret = tss.reconstruct(&recovered_indices, recovered_shares);
  assert_eq!(reconstructed_secret, secret);
}
```


## Packed sharing
If many secrets are to be secret shared, it may be beneficial to use the packed scheme where several secrets are packed into each share. While still very computational efficient, one downside is that the parameters are somewhat restricted.

Specifically, the parameters are split in *scheme parameters* and *implementation parameters*:
- the former, like in Shamir sharing, determines the abstract properties of the scheme, yet now also with a `secret_count` specifying how many secrets are to be packed into each share; the reconstruction limit is implicitly defined as `secret_count + threshold + 1`
- the latter is related to the implementation (currently based on the Fast Fourier Transform) and requires not only a `prime` specifying the field, but also two principal roots of unity within that field, which must be respectively a power of 2 and a power of 3

Due to this increased complexity, providing helper functions for finding suitable parameters are in progress. For now, a few fixed fields are included in the `packed` module as illustrated in the example below:

- `PSS_4_8_3`, `PSS_4_26_3`, `PSS_155_728_100`, `PSS_155_19682_100`

with format `PSS_T_N_D` for sharing `D` secrets into `N` shares with a threshold of `T`.

```rust
extern crate threshold_secret_sharing as tss;

fn main() {
  // use predefined parameters
  let ref tss = tss::packed::PSS_4_26_3;

  // generate shares for a vector of secrets
  let secrets = [1, 2, 3];
  let all_shares = tss.share(&secrets);

  // artificially remove some of the shares; keep only the first 8
  let indices: Vec<usize> = (0..8).collect();
  let shares = &all_shares[0..8];

  // reconstruct using remaining subset of shares
  let recovered_secrets = tss.reconstruct(&indices, shares);
  assert_eq!(recovered_secrets, vec![1, 2, 3]);
}
```


## Homomorphic properties
Both the Shamir and the packed scheme enjoy certain homomorphic properties: shared secrets can be transformed by manipulating the shares. Both addition and multiplications work, yet notice that the reconstruction limit in the case of multiplication goes up by a factor of two for each application.

```rust
extern crate threshold_secret_sharing as tss;

fn main() {
  // use predefined parameters
  let ref tss = tss::PSS_4_26_3;

  // generate shares for first vector of secrets
  let secrets_1 = [1, 2, 3];
  let shares_1 = tss.share(&secrets_1);

  // generate shares for second vector of secrets
  let secrets_2 = [4, 5, 6];
  let shares_2 = tss.share(&secrets_2);

  // combine shares pointwise to get shares of the sum of the secrets
  let shares_sum: Vec<i64> = shares_1.iter().zip(&shares_2)
    .map(|(a, b)| (a + b) % tss.prime).collect();

  // artificially remove some of the shares; keep only the first 8
  let indices: Vec<usize> = (0..8).collect();
  let shares = &shares_sum[0..8];

  // reconstruct using remaining subset of shares
  let recovered_secrets = tss.reconstruct(&indices, shares);
  assert_eq!(recovered_secrets, vec![5, 7, 9]);
}
```

# Parameter generation
While it's straight-forward to instantiate the Shamir scheme, as mentioned above the packed scheme is more tricky and a few helper methods are provided as a result. Since some applications needs only a fixed choice of parameters, these helper methods are optional and only included if the `paramgen` feature is activated during compilation:
```
cargo build --features paramgen
```
which also adds several extra dependencies.


# Performance
So far most performance efforts has been focused on share generation for the packed scheme, with some obvious enhancements for reconstruction in the process of being implemented. As an example, sharing 100 secrets into approximately 20,000 shares with the packed scheme runs in around 31ms on a recent laptop, and in around 590ms on a Raspberry Pi 3.

These numbers were obtained by running
```
cargo bench
```
using the nightly toolchain.

# License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
