//! ![](http://terhix.com/doc/json-rust-logo-small.png)
//!
//! # json-rust
//!
//! Parse and serialize [JSON](http://json.org/) with ease.
//!
//! **[Changelog](https://github.com/maciejhirsz/json-rust/releases) -**
//! **[Complete Documentation](http://terhix.com/doc/json/) -**
//! **[Cargo](https://crates.io/crates/json) -**
//! **[Repository](https://github.com/maciejhirsz/json-rust)**
//!
//! ## Why?
//!
//! JSON is a very loose format where anything goes - arrays can hold mixed
//! types, object keys can change types between API calls or not include
//! some keys under some conditions. Mapping that to idiomatic Rust structs
//! introduces friction.
//!
//! This crate intends to avoid that friction.
//!
//! ```rust
//! # #[macro_use] extern crate json;
//! # fn main() {
//! let parsed = json::parse(r#"
//!
//! {
//!     "code": 200,
//!     "success": true,
//!     "payload": {
//!         "features": [
//!             "awesome",
//!             "easyAPI",
//!             "lowLearningCurve"
//!         ]
//!     }
//! }
//!
//! "#).unwrap();
//!
//! let instantiated = object!{
//!     "code" => 200,
//!     "success" => true,
//!     "payload" => object!{
//!         "features" => array![
//!             "awesome",
//!             "easyAPI",
//!             "lowLearningCurve"
//!         ]
//!     }
//! };
//!
//! assert_eq!(parsed, instantiated);
//! # }
//! ```
//!
//! ## First class citizen
//!
//! Using macros and indexing, it's easy to work with the data.
//!
//! ```rust
//! # #[macro_use] extern crate json;
//! # fn main() {
//! let mut data = object!{
//!     "foo" => false,
//!     "bar" => json::Null,
//!     "answer" => 42,
//!     "list" => array![json::Null, "world", true]
//! };
//!
//! // Partial equality is implemented for most raw types:
//! assert!(data["foo"] == false);
//!
//! // And it's type aware, `null` and `false` are different values:
//! assert!(data["bar"] != false);
//!
//! // But you can use any Rust number types:
//! assert!(data["answer"] == 42);
//! assert!(data["answer"] == 42.0);
//! assert!(data["answer"] == 42isize);
//!
//! // Access nested structures, arrays and objects:
//! assert!(data["list"][0].is_null());
//! assert!(data["list"][1] == "world");
//! assert!(data["list"][2] == true);
//!
//! // Error resilient - accessing properties that don't exist yield null:
//! assert!(data["this"]["does"]["not"]["exist"].is_null());
//!
//! // Mutate by assigning:
//! data["list"][0] = "Hello".into();
//!
//! // Use the `dump` method to serialize the data:
//! assert_eq!(data.dump(), r#"{"foo":false,"bar":null,"answer":42,"list":["Hello","world",true]}"#);
//!
//! // Or pretty print it out:
//! println!("{:#}", data);
//! # }
//! ```
//!
//! ## Serialize with `json::stringify(value)`
//!
//! Primitives:
//!
//! ```
//! // str slices
//! assert_eq!(json::stringify("foobar"), "\"foobar\"");
//!
//! // Owned strings
//! assert_eq!(json::stringify("foobar".to_string()), "\"foobar\"");
//!
//! // Any number types
//! assert_eq!(json::stringify(42), "42");
//!
//! // Booleans
//! assert_eq!(json::stringify(true), "true");
//! assert_eq!(json::stringify(false), "false");
//! ```
//!
//! Explicit `null` type `json::Null`:
//!
//! ```
//! assert_eq!(json::stringify(json::Null), "null");
//! ```
//!
//! Optional types:
//!
//! ```
//! let value: Option<String> = Some("foo".to_string());
//! assert_eq!(json::stringify(value), "\"foo\"");
//!
//! let no_value: Option<String> = None;
//! assert_eq!(json::stringify(no_value), "null");
//! ```
//!
//! Vector:
//!
//! ```
//! let data = vec![1,2,3];
//! assert_eq!(json::stringify(data), "[1,2,3]");
//! ```
//!
//! Vector with optional values:
//!
//! ```
//! let data = vec![Some(1), None, Some(2), None, Some(3)];
//! assert_eq!(json::stringify(data), "[1,null,2,null,3]");
//! ```
//!
//! Pushing to arrays:
//!
//! ```
//! let mut data = json::JsonValue::new_array();
//!
//! data.push(10);
//! data.push("foo");
//! data.push(false);
//!
//! assert_eq!(data.dump(), r#"[10,"foo",false]"#);
//! ```
//!
//! Putting fields on objects:
//!
//! ```
//! let mut data = json::JsonValue::new_object();
//!
//! data["answer"] = 42.into();
//! data["foo"] = "bar".into();
//!
//! assert_eq!(data.dump(), r#"{"answer":42,"foo":"bar"}"#);
//! ```
//!
//! `array!` macro:
//!
//! ```
//! # #[macro_use] extern crate json;
//! # fn main() {
//! let data = array!["foo", "bar", 100, true, json::Null];
//! assert_eq!(data.dump(), r#"["foo","bar",100,true,null]"#);
//! # }
//! ```
//!
//! `object!` macro:
//!
//! ```
//! # #[macro_use] extern crate json;
//! # fn main() {
//! let data = object!{
//!     "name"    => "John Doe",
//!     "age"     => 30,
//!     "canJSON" => true
//! };
//! assert_eq!(
//!     data.dump(),
//!     r#"{"name":"John Doe","age":30,"canJSON":true}"#
//! );
//! # }
//! ```

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

use std::result;
use std::string::String;
use std::vec::Vec;

mod codegen;
mod parser;
mod value;
mod error;
mod util;

pub mod short;
pub mod object;
pub mod number;

pub use error::Error;
pub use value::JsonValue;
pub use value::JsonValue::Null;

/// Result type used by this crate.
///
///
/// *Note:* Since 0.9.0 the old `JsonResult` type is deprecated. Always use
/// `json::Result` instead.
pub type Result<T> = result::Result<T, Error>;

pub mod iterators {
    /// Iterator over members of `JsonValue::Array`.
    pub type Members<'a> = ::std::slice::Iter<'a, super::JsonValue>;

    /// Mutable iterator over members of `JsonValue::Array`.
    pub type MembersMut<'a> = ::std::slice::IterMut<'a, super::JsonValue>;

    /// Iterator over key value pairs of `JsonValue::Object`.
    pub type Entries<'a> = super::object::Iter<'a>;

    /// Mutable iterator over key value pairs of `JsonValue::Object`.
    pub type EntriesMut<'a> = super::object::IterMut<'a>;
}

#[deprecated(since="0.9.0", note="use `json::Error` instead")]
pub use Error as JsonError;

#[deprecated(since="0.9.0", note="use `json::Result` instead")]
pub use Result as JsonResult;

pub use parser::parse;

pub type Array = Vec<JsonValue>;

/// Convenience for `JsonValue::from(value)`
pub fn from<T>(value: T) -> JsonValue where T: Into<JsonValue> {
    value.into()
}

/// Pretty prints out the value as JSON string.
pub fn stringify<T>(root: T) -> String where T: Into<JsonValue> {
    let root: JsonValue = root.into();
    root.dump()
}

/// Pretty prints out the value as JSON string. Second argument is a
/// number of spaces to indent new blocks with.
pub fn stringify_pretty<T>(root: T, spaces: u16) -> String where T: Into<JsonValue> {
    let root: JsonValue = root.into();
    root.pretty(spaces)
}

/// Helper macro for creating instances of `JsonValue::Array`.
///
/// ```
/// # #[macro_use] extern crate json;
/// # fn main() {
/// let data = array!["foo", 42, false];
///
/// assert_eq!(data[0], "foo");
/// assert_eq!(data[1], 42);
/// assert_eq!(data[2], false);
///
/// assert_eq!(data.dump(), r#"["foo",42,false]"#);
/// # }
/// ```
#[macro_export]
macro_rules! array {
    [] => ($crate::JsonValue::new_array());

    [ $( $item:expr ),* ] => ({
        let mut array = Vec::new();

        $(
            array.push($item.into());
        )*

        $crate::JsonValue::Array(array)
    })
}

/// Helper macro for creating instances of `JsonValue::Object`.
///
/// ```
/// # #[macro_use] extern crate json;
/// # fn main() {
/// let data = object!{
///     "foo" => 42,
///     "bar" => false
/// };
///
/// assert_eq!(data["foo"], 42);
/// assert_eq!(data["bar"], false);
///
/// assert_eq!(data.dump(), r#"{"foo":42,"bar":false}"#);
/// # }
/// ```
#[macro_export]
macro_rules! object {
    // Empty object.
    {} => ($crate::JsonValue::new_object());

    // Non-empty object, no trailing comma.
    //
    // In this implementation, key/value pairs separated by commas.
    { $( $key:expr => $value:expr ),* } => {
        object!( $(
            $key => $value,
        )* )
    };

    // Non-empty object, trailing comma.
    //
    // In this implementation, the comma is part of the value.
    { $( $key:expr => $value:expr, )* } => ({
        use $crate::object::Object;

        let mut object = Object::new();

        $(
            object.insert($key, $value.into());
        )*

        $crate::JsonValue::Object(object)
    })
}

