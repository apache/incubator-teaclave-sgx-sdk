sgx_untrusted_time
====

Utilities for working with time-related functions in Rust SGX

## Notes

This library is no longer actively maintained, but bugfixes will be added ([details](https://github.com/rust-lang-deprecated/time/issues/136)).

In case you're looking for something a little fresher and more actively maintained have a look at the [`chrono`](https://github.com/lifthrasiir/rust-chrono) crate.

## Attention

**This library only supports UTC timezone.**

## Usage

Put this in your `Cargo.toml`:

```toml
[dependencies]
sgx_untrusted_time = "0.1"
```

And this in your crate root:

```rust
extern crate sgx_untrusted_time;
```
