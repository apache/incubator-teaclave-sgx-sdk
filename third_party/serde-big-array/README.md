## serde-big-array

[![docs](https://docs.rs/serde-big-array/badge.svg)](https://docs.rs/crate/serde-big-array)
[![crates.io](https://img.shields.io/crates/v/serde-big-array.svg)](https://crates.io/crates/serde-big-array)
[![dependency status](https://deps.rs/repo/github/est31/serde-big-array/status.svg)](https://deps.rs/repo/github/est31/serde-big-array)

Big array helper for serde. The purpose of this crate is to make (de-)serializing arrays of sizes > 32 easy. This solution is needed until [const generics](https://github.com/rust-lang/rust/issues/44580) are becoming stable.

Bases on [this](https://github.com/serde-rs/serde/issues/631#issuecomment-322677033) snippet.

```Rust
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate serde_big_array;

big_array! { BigArray; }

#[derive(Serialize, Deserialize)]
struct S {
    #[serde(with = "BigArray")]
    arr: [u8; 64],
}

#[test]
fn test() {
    let s = S { arr: [1; 64] };
    let j = serde_json::to_string(&s).unwrap();
    let s_back = serde_json::from_str::<S>(&j).unwrap();
    assert!(&s.arr[..] == &s_back.arr[..]);
}
```

### MSRV

The minimum supported Rust version (MSRV) is Rust 1.20.0.

### License
[license]: #license

This crate is distributed under the terms of both the MIT license
and the Apache License (Version 2.0), at your option.

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.

#### License of your contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in the work by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
