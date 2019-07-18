# http_req-sgx

Relevant changes:

- Supports only `rust-tls`

## Original README.md

# http_req

[![Build Status](https://travis-ci.org/jayjamesjay/http_req.svg?branch=master)](https://travis-ci.org/jayjamesjay/http_req)
[![Crates.io](https://img.shields.io/badge/crates.io-v0.5.0-orange.svg?longCache=true)](https://crates.io/crates/http_req)
[![Docs.rs](https://docs.rs/http_req/badge.svg)](https://docs.rs/http_req/0.5.0/http_req/)

Simple and lightweight HTTP client with built-in HTTPS support.

## Requirements

http_req by default uses [rust-native-tls](https://github.com/sfackler/rust-native-tls),
which uses TLS framework provided by OS on Windows and macOS, and OpenSSL
on all other platforms. But it also supports [rus-tls](https://crates.io/crates/rustls).

## Example

Basic GET request

```rust
use http_req::request;

fn main() {
    let mut writer = Vec::new(); //container for body of a response
    let res = request::get("https://doc.rust-lang.org/", &mut writer).unwrap();

    println!("Status: {} {}", res.status_code(), res.reason());
}
```

## How to use with `rustls`:

In order to use `http_req` with `rustls` in your project, add following lines to `Cargo.toml`:

```toml
[dependencies]
http_req  = {version="^0.5", default-features = false, features = ["rust-tls"]}
```

## License

Licensed under [MIT](https://github.com/jayjamesjay/http_req/blob/master/LICENSE).
