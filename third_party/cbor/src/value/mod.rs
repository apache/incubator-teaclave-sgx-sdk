//! CBOR values, keys and serialization routines.

pub mod value;
pub mod ser;

pub use self::value::{ObjectKey, Value, from_value};
pub use self::ser::to_value;
