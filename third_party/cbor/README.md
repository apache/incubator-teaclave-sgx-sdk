# Serde CBOR Serialization Library
[![Build Status](https://travis-ci.org/pyfisch/cbor.svg?branch=master)](https://travis-ci.org/pyfisch/cbor)
[![Crates.io](https://img.shields.io/crates/v/serde_cbor.svg)](https://crates.io/crates/serde_cbor)
[Documentation](https://pyfisch.github.io/cbor/serde_cbor/)

This crate is a Rust library for parsing and generating the
[CBOR](http://cbor.io/) (Concise Binary Object Representation)
file format. It is built upon [Serde](https://github.com/serde-rs/serde),
a high performance generic serialization framework.

## About CBOR
CBOR is a binary encoding based on a superset of the JSON data model.
It supports all the standard JSON types plus binary data, big numbers,
non-string keys, time values and custom data types using tagging of values.
CBOR is always shorter than the corresponding JSON representation and easier
and faster to parse.

## License
Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
