Erased Serde
============

[![Build Status](https://api.travis-ci.org/dtolnay/erased-serde.svg?branch=master)](https://travis-ci.org/dtolnay/erased-serde)
[![Latest Version](https://img.shields.io/crates/v/erased-serde.svg)](https://crates.io/crates/erased-serde)

This crate provides type-erased versions of Serde's `Serialize`, `Serializer`
and `Deserializer` traits that can be used as [trait objects].

[trait objects]: https://doc.rust-lang.org/book/first-edition/trait-objects.html

- [`erased_serde::Serialize`](https://docs.rs/erased-serde/0.3/erased_serde/trait.Serialize.html)
- [`erased_serde::Serializer`](https://docs.rs/erased-serde/0.3/erased_serde/trait.Serializer.html)
- [`erased_serde::Deserializer`](https://docs.rs/erased-serde/0.3/erased_serde/trait.Deserializer.html)

The usual Serde `Serialize`, `Serializer` and `Deserializer` traits cannot be
used as trait objects like `&Serialize` or boxed trait objects like
`Box<Serialize>` because of Rust's ["object safety" rules]. In particular, all
three traits contain generic methods which cannot be made into a trait object.

["object safety" rules]: http://huonw.github.io/blog/2015/01/object-safety/

**The traits in this crate work seamlessly with any existing Serde `Serialize`
and `Deserialize` type and any existing Serde `Serializer` and `Deserializer`
format.**

```toml
[dependencies]
serde = "1.0"
erased-serde = "0.3"
```

## Serialization

```rust
extern crate erased_serde;
extern crate serde_json;
extern crate serde_cbor;

use std::collections::BTreeMap as Map;
use std::io;

use erased_serde::{Serialize, Serializer};

fn main() {
    // Construct some serializers.
    let json = &mut serde_json::ser::Serializer::new(io::stdout());
    let cbor = &mut serde_cbor::ser::Serializer::new(io::stdout());

    // The values in this map are boxed trait objects. Ordinarily this would not
    // be possible with serde::Serializer because of object safety, but type
    // erasure makes it possible with erased_serde::Serializer.
    let mut formats: Map<&str, Box<Serializer>> = Map::new();
    formats.insert("json", Box::new(Serializer::erase(json)));
    formats.insert("cbor", Box::new(Serializer::erase(cbor)));

    // These are boxed trait objects as well. Same thing here - type erasure
    // makes this possible.
    let mut values: Map<&str, Box<Serialize>> = Map::new();
    values.insert("vec", Box::new(vec!["a", "b"]));
    values.insert("int", Box::new(65536));

    // Pick a Serializer out of the formats map.
    let format = formats.get_mut("json").unwrap();

    // Pick a Serialize out of the values map.
    let value = values.get("vec").unwrap();

    // This line prints `["a","b"]` to stdout.
    value.erased_serialize(format).unwrap();
}
```

## Deserialization

```rust
extern crate erased_serde;
extern crate serde_json;
extern crate serde_cbor;

use std::collections::BTreeMap as Map;

use erased_serde::Deserializer;

fn main() {
    static JSON: &'static [u8] = br#"{"A": 65, "B": 66}"#;
    static CBOR: &'static [u8] = &[162, 97, 65, 24, 65, 97, 66, 24, 66];

    // Construct some deserializers.
    let json = &mut serde_json::de::Deserializer::from_slice(JSON);
    let cbor = &mut serde_cbor::de::Deserializer::from_slice(CBOR);

    // The values in this map are boxed trait objects, which is not possible
    // with the normal serde::Deserializer because of object safety.
    let mut formats: Map<&str, Box<Deserializer>> = Map::new();
    formats.insert("json", Box::new(Deserializer::erase(json)));
    formats.insert("cbor", Box::new(Deserializer::erase(cbor)));

    // Pick a Deserializer out of the formats map.
    let format = formats.get_mut("json").unwrap();

    let data: Map<String, usize> = erased_serde::deserialize(format).unwrap();

    println!("{}", data["A"] + data["B"]);
}
```

## How it works

This crate is based on a general technique for building trait objects of traits
that have generic methods (like all of Serde's traits). [This example code]
demonstrates the technique applied to a simplified case of a single generic
method. [Try it in the playground.]

[This example code]: https://github.com/dtolnay/erased-serde/blob/master/explanation/main.rs
[Try it in the playground.]: https://play.rust-lang.org/?gist=2e839b72e3660eeee988b271bf487fc8

In erased-serde things are a bit more complicated than in the example for three
reasons but the idea is the same.

- We need to deal with trait methods that take `self` by value -- effectively by
  implementing the object-safe trait for `Option<T>` where `T` implements the
  real trait.
- We need to deal with traits that have associated types like `Serializer::Ok`
  and `Visitor::Value` -- by carefully short-term stashing things behind a
  pointer.
- We need to support trait methods that have a generic type in the return type
  but none of the argument types, like `SeqAccess::next_element` -- this can be
  flipped around into a callback style where the return value is instead passed
  on to a generic argument.

In the future maybe the Rust compiler will be able to apply this technique
automatically to any trait that is not already object safe by the current rules.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
