![](http://terhix.com/doc/json-rust-logo-small.png)

# json-rust

[![Travis shield](https://travis-ci.org/maciejhirsz/json-rust.svg)](https://travis-ci.org/maciejhirsz/json-rust)
[![Crates.io version shield](https://img.shields.io/crates/v/json.svg)](https://crates.io/crates/json)
[![Crates.io license shield](https://img.shields.io/crates/l/json.svg)](https://crates.io/crates/json)

Parse and serialize [JSON](http://json.org/) with ease.

**[Changelog](https://github.com/maciejhirsz/json-rust/releases) -**
**[Complete Documentation](https://docs.rs/json/) -**
**[Cargo](https://crates.io/crates/json) -**
**[Repository](https://github.com/maciejhirsz/json-rust)**


## Why?

JSON is a very loose format where anything goes - arrays can hold mixed
types, object keys can change types between API calls or not include
some keys under some conditions. Mapping that to idiomatic Rust structs
introduces friction.

This crate intends to avoid that friction.

```rust
let parsed = json::parse(r#"

{
    "code": 200,
    "success": true,
    "payload": {
        "features": [
            "awesome",
            "easyAPI",
            "lowLearningCurve"
        ]
    }
}

"#).unwrap();

let instantiated = object!{
    "code" => 200,
    "success" => true,
    "payload" => object!{
        "features" => array![
            "awesome",
            "easyAPI",
            "lowLearningCurve"
        ]
    }
};

assert_eq!(parsed, instantiated);
```

## First class citizen

Using macros and indexing, it's easy to work with the data.

```rust
let mut data = object!{
    "foo" => false,
    "bar" => json::Null,
    "answer" => 42,
    "list" => array![json::Null, "world", true]
};

// Partial equality is implemented for most raw types:
assert!(data["foo"] == false);

// And it's type aware, `null` and `false` are different values:
assert!(data["bar"] != false);

// But you can use any Rust number types:
assert!(data["answer"] == 42);
assert!(data["answer"] == 42.0);
assert!(data["answer"] == 42isize);

// Access nested structures, arrays and objects:
assert!(data["list"][0].is_null());
assert!(data["list"][1] == "world");
assert!(data["list"][2] == true);

// Error resilient - accessing properties that don't exist yield null:
assert!(data["this"]["does"]["not"]["exist"].is_null());

// Mutate by assigning:
data["list"][0] = "Hello".into();

// Use the `dump` method to serialize the data:
assert_eq!(data.dump(), r#"{"foo":false,"bar":null,"answer":42,"list":["Hello","world",true]}"#);

// Or pretty print it out:
println!("{:#}", data);
```

## Installation

Just add it to your `Cargo.toml` file:

```toml
[dependencies]
json = "*"
```

Then import it in your `main.rs` / `lib.rs` file:

```rust
#[macro_use]
extern crate json;
```

## Performance and Conformance

There used to be a statement here saying that performance is not the main goal of this
crate. It is definitely one of them now.

While this crate doesn't provide a way to parse JSON to native Rust structs, it does a
lot to optimize its performance for DOM parsing, stringifying and manipulation. It does
[very well in benchmarks](https://github.com/serde-rs/json-benchmark), in some cases it
can even outperform parsing to structs.

This crate implements the standard according to the [
RFC 7159](https://tools.ietf.org/html/rfc7159) and
[ECMA-404](http://www.ecma-international.org/publications/files/ECMA-ST/ECMA-404.pdf)
documents. For the best interoperability numbers are treated stored as 64bit precision
mantissa with 16 bit decimal exponent for floating point representation.

## License

This crate is distributed under the terms of both the MIT license
and the Apache License (Version 2.0). Choose whichever one works best for you.

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
