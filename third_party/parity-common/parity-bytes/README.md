## `no_std` support

This crate has a feature, `std`, that is enabled by default. To use this crate
in a `no_std` context, add the following to your `Cargo.toml` (still requires allocator though):

```toml
[dependencies]
parity-bytes = { version = "0.1", default-features = false }
```

Until allocator api is stabilized, this type of use is limited to nightly Rust.
