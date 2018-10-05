//! This crate provides type-erased versions of Serde's `Serialize`, `Serializer`
//! and `Deserializer` traits that can be used as [trait
//! objects](https://doc.rust-lang.org/book/trait-objects.html).
//!
//! The usual Serde `Serialize`, `Serializer` and `Deserializer` traits cannot
//! be used as trait objects like `&Serialize` or boxed trait objects like
//! `Box<Serialize>` because of Rust's ["object safety"
//! rules](http://huonw.github.io/blog/2015/01/object-safety/). In particular,
//! all three traits contain generic methods which cannot be made into a trait
//! object.
//!
//! **The traits in this crate work seamlessly with any existing Serde
//! `Serialize` and `Deserialize` type and any existing Serde `Serializer` and
//! `Deserializer` format.**
//!
//! ## Serialization
//!
//! ```rust
//! extern crate erased_serde;
//! extern crate serde_json;
//! extern crate serde_cbor;
//!
//! use std::collections::BTreeMap as Map;
//! use std::io;
//!
//! use erased_serde::{Serialize, Serializer};
//!
//! fn main() {
//!     // Construct some serializers.
//!     let json = &mut serde_json::ser::Serializer::new(io::stdout());
//!     let cbor = &mut serde_cbor::ser::Serializer::new(io::stdout());
//!
//!     // The values in this map are boxed trait objects. Ordinarily this would not
//!     // be possible with serde::Serializer because of object safety, but type
//!     // erasure makes it possible with erased_serde::Serializer.
//!     let mut formats: Map<&str, Box<Serializer>> = Map::new();
//!     formats.insert("json", Box::new(Serializer::erase(json)));
//!     formats.insert("cbor", Box::new(Serializer::erase(cbor)));
//!
//!     // These are boxed trait objects as well. Same thing here - type erasure
//!     // makes this possible.
//!     let mut values: Map<&str, Box<Serialize>> = Map::new();
//!     values.insert("vec", Box::new(vec!["a", "b"]));
//!     values.insert("int", Box::new(65536));
//!
//!     // Pick a Serializer out of the formats map.
//!     let format = formats.get_mut("json").unwrap();
//!
//!     // Pick a Serialize out of the values map.
//!     let value = values.get("vec").unwrap();
//!
//!     // This line prints `["a","b"]` to stdout.
//!     value.erased_serialize(format).unwrap();
//! }
//! ```
//!
//! ## Deserialization
//!
//! ```rust
//! extern crate erased_serde;
//! extern crate serde_json;
//! extern crate serde_cbor;
//!
//! use std::collections::BTreeMap as Map;
//!
//! use erased_serde::Deserializer;
//!
//! fn main() {
//!     static JSON: &'static [u8] = br#"{"A": 65, "B": 66}"#;
//!     static CBOR: &'static [u8] = &[162, 97, 65, 24, 65, 97, 66, 24, 66];
//!
//!     // Construct some deserializers.
//!     let json = &mut serde_json::de::Deserializer::from_slice(JSON);
//!     let cbor = &mut serde_cbor::de::Deserializer::from_slice(CBOR);
//!
//!     // The values in this map are boxed trait objects, which is not possible
//!     // with the normal serde::Deserializer because of object safety.
//!     let mut formats: Map<&str, Box<Deserializer>> = Map::new();
//!     formats.insert("json", Box::new(Deserializer::erase(json)));
//!     formats.insert("cbor", Box::new(Deserializer::erase(cbor)));
//!
//!     // Pick a Deserializer out of the formats map.
//!     let format = formats.get_mut("json").unwrap();
//!
//!     let data: Map<String, usize> = erased_serde::deserialize(format).unwrap();
//!
//!     println!("{}", data["A"] + data["B"]);
//! }
//! ```

#![doc(html_root_url = "https://docs.rs/erased-serde/0.3.6")]

#![cfg_attr(feature = "unstable-debug", feature(core_intrinsics))]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

#[macro_use]
extern crate serde;

#[cfg(test)]
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate serde_json;

#[macro_use]
mod macros;

mod any;
mod de;
mod error;
mod ser;

pub use de::{deserialize, Deserializer};
pub use error::Error;
pub use ser::{serialize, Serialize, Serializer};

// Not public API.
#[doc(hidden)]
pub mod private;
