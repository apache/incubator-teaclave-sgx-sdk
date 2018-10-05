use std::prelude::v1::*;
use std::fmt::{self, Display};

use serde;

use any::Any;
use error::Error;

/// Deserialize a value of type `T` from the given trait object.
///
/// ```rust
/// extern crate erased_serde;
/// extern crate serde_json;
/// extern crate serde_cbor;
///
/// use std::collections::BTreeMap as Map;
///
/// use erased_serde::Deserializer;
///
/// fn main() {
///     static JSON: &'static [u8] = br#"{"A": 65, "B": 66}"#;
///     static CBOR: &'static [u8] = &[162, 97, 65, 24, 65, 97, 66, 24, 66];
///
///     // Construct some deserializers.
///     let json = &mut serde_json::de::Deserializer::from_slice(JSON);
///     let cbor = &mut serde_cbor::de::Deserializer::from_slice(CBOR);
///
///     // The values in this map are boxed trait objects, which is not possible
///     // with the normal serde::Deserializer because of object safety.
///     let mut formats: Map<&str, Box<Deserializer>> = Map::new();
///     formats.insert("json", Box::new(Deserializer::erase(json)));
///     formats.insert("cbor", Box::new(Deserializer::erase(cbor)));
///
///     // Pick a Deserializer out of the formats map.
///     let format = formats.get_mut("json").unwrap();
///
///     let data: Map<String, usize> = erased_serde::deserialize(format).unwrap();
///
///     println!("{}", data["A"] + data["B"]);
/// }
/// ```
pub fn deserialize<'de, T>(deserializer: &mut Deserializer<'de>) -> Result<T, Error>
    where T: serde::Deserialize<'de>
{
    serde::Deserialize::deserialize(deserializer)
}

// TRAITS //////////////////////////////////////////////////////////////////////

pub trait DeserializeSeed<'de> {
    fn erased_deserialize_seed(&mut self, &mut Deserializer<'de>) -> Result<Out, Error>;
}

/// An object-safe equivalent of Serde's `Deserializer` trait.
///
/// Any implementation of Serde's `Deserializer` can be converted to an
/// `&erased_serde::Deserializer` or `Box<erased_serde::Deserializer>` trait
/// object using `erased_serde::Deserializer::erase`.
///
/// ```rust
/// extern crate erased_serde;
/// extern crate serde_json;
/// extern crate serde_cbor;
///
/// use std::collections::BTreeMap as Map;
///
/// use erased_serde::Deserializer;
///
/// fn main() {
///     static JSON: &'static [u8] = br#"{"A": 65, "B": 66}"#;
///     static CBOR: &'static [u8] = &[162, 97, 65, 24, 65, 97, 66, 24, 66];
///
///     // Construct some deserializers.
///     let json = &mut serde_json::de::Deserializer::from_slice(JSON);
///     let cbor = &mut serde_cbor::de::Deserializer::from_slice(CBOR);
///
///     // The values in this map are boxed trait objects, which is not possible
///     // with the normal serde::Deserializer because of object safety.
///     let mut formats: Map<&str, Box<Deserializer>> = Map::new();
///     formats.insert("json", Box::new(Deserializer::erase(json)));
///     formats.insert("cbor", Box::new(Deserializer::erase(cbor)));
///
///     // Pick a Deserializer out of the formats map.
///     let format = formats.get_mut("json").unwrap();
///
///     let data: Map<String, usize> = erased_serde::deserialize(format).unwrap();
///
///     println!("{}", data["A"] + data["B"]);
/// }
/// ```
pub trait Deserializer<'de> {
    fn erased_deserialize_any(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_bool(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_u8(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_u16(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_u32(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_u64(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_i8(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_i16(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_i32(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_i64(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    serde_if_integer128! {
        fn erased_deserialize_i128(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_u128(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    }
    fn erased_deserialize_f32(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_f64(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_char(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_str(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_string(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_bytes(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_byte_buf(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_option(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_unit(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_unit_struct(&mut self, name: &'static str, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_newtype_struct(&mut self, name: &'static str, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_seq(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_tuple(&mut self, len: usize, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_tuple_struct(&mut self, name: &'static str, len: usize, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_map(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_struct(&mut self, name: &'static str, fields: &'static [&'static str], &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_identifier(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_enum(&mut self, name: &'static str, variants: &'static [&'static str], &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_deserialize_ignored_any(&mut self, &mut Visitor<'de>) -> Result<Out, Error>;
    fn erased_is_human_readable(&self) -> bool;
}

pub trait Visitor<'de> {
    fn erased_expecting(&self, &mut fmt::Formatter) -> fmt::Result;
    fn erased_visit_bool(&mut self, bool) -> Result<Out, Error>;
    fn erased_visit_i8(&mut self, i8) -> Result<Out, Error>;
    fn erased_visit_i16(&mut self, i16) -> Result<Out, Error>;
    fn erased_visit_i32(&mut self, i32) -> Result<Out, Error>;
    fn erased_visit_i64(&mut self, i64) -> Result<Out, Error>;
    fn erased_visit_u8(&mut self, u8) -> Result<Out, Error>;
    fn erased_visit_u16(&mut self, u16) -> Result<Out, Error>;
    fn erased_visit_u32(&mut self, u32) -> Result<Out, Error>;
    fn erased_visit_u64(&mut self, u64) -> Result<Out, Error>;
    serde_if_integer128! {
        fn erased_visit_i128(&mut self, i128) -> Result<Out, Error>;
        fn erased_visit_u128(&mut self, u128) -> Result<Out, Error>;
    }
    fn erased_visit_f32(&mut self, f32) -> Result<Out, Error>;
    fn erased_visit_f64(&mut self, f64) -> Result<Out, Error>;
    fn erased_visit_char(&mut self, char) -> Result<Out, Error>;
    fn erased_visit_str(&mut self, &str) -> Result<Out, Error>;
    fn erased_visit_borrowed_str(&mut self, &'de str) -> Result<Out, Error>;
    fn erased_visit_string(&mut self, String) -> Result<Out, Error>;
    fn erased_visit_bytes(&mut self, &[u8]) -> Result<Out, Error>;
    fn erased_visit_borrowed_bytes(&mut self, &'de [u8]) -> Result<Out, Error>;
    fn erased_visit_byte_buf(&mut self, Vec<u8>) -> Result<Out, Error>;
    fn erased_visit_none(&mut self) -> Result<Out, Error>;
    fn erased_visit_some(&mut self, &mut Deserializer<'de>) -> Result<Out, Error>;
    fn erased_visit_unit(&mut self) -> Result<Out, Error>;
    fn erased_visit_newtype_struct(&mut self, &mut Deserializer<'de>) -> Result<Out, Error>;
    fn erased_visit_seq(&mut self, &mut SeqAccess<'de>) -> Result<Out, Error>;
    fn erased_visit_map(&mut self, &mut MapAccess<'de>) -> Result<Out, Error>;
    fn erased_visit_enum(&mut self, &mut EnumAccess<'de>) -> Result<Out, Error>;
}

pub trait SeqAccess<'de> {
    fn erased_next_element(&mut self, &mut DeserializeSeed<'de>) -> Result<Option<Out>, Error>;
    fn erased_size_hint(&self) -> Option<usize>;
}

pub trait MapAccess<'de> {
    fn erased_next_key(&mut self, &mut DeserializeSeed<'de>) -> Result<Option<Out>, Error>;
    fn erased_next_value(&mut self, &mut DeserializeSeed<'de>) -> Result<Out, Error>;
    fn erased_next_entry(&mut self, key: &mut DeserializeSeed<'de>, value: &mut DeserializeSeed<'de>) -> Result<Option<(Out, Out)>, Error>;
    fn erased_size_hint(&self) -> Option<usize>;
}

pub trait EnumAccess<'de> {
    fn erased_variant_seed(&mut self, &mut DeserializeSeed<'de>) -> Result<(Out, Variant<'de>), Error>;
}

impl<'de> Deserializer<'de> {
    /// Convert any Serde `Deserializer` to a trait object.
    ///
    /// ```rust
    /// extern crate erased_serde;
    /// extern crate serde_json;
    /// extern crate serde_cbor;
    ///
    /// use std::collections::BTreeMap as Map;
    ///
    /// use erased_serde::Deserializer;
    ///
    /// fn main() {
    ///     static JSON: &'static [u8] = br#"{"A": 65, "B": 66}"#;
    ///     static CBOR: &'static [u8] = &[162, 97, 65, 24, 65, 97, 66, 24, 66];
    ///
    ///     // Construct some deserializers.
    ///     let json = &mut serde_json::de::Deserializer::from_slice(JSON);
    ///     let cbor = &mut serde_cbor::de::Deserializer::from_slice(CBOR);
    ///
    ///     // The values in this map are boxed trait objects, which is not possible
    ///     // with the normal serde::Deserializer because of object safety.
    ///     let mut formats: Map<&str, Box<Deserializer>> = Map::new();
    ///     formats.insert("json", Box::new(Deserializer::erase(json)));
    ///     formats.insert("cbor", Box::new(Deserializer::erase(cbor)));
    ///
    ///     // Pick a Deserializer out of the formats map.
    ///     let format = formats.get_mut("json").unwrap();
    ///
    ///     let data: Map<String, usize> = erased_serde::deserialize(format).unwrap();
    ///
    ///     println!("{}", data["A"] + data["B"]);
    /// }
    /// ```
    pub fn erase<D>(deserializer: D) -> erase::Deserializer<D>
        where D: serde::Deserializer<'de>
    {
        erase::Deserializer { state: Some(deserializer) }
    }
}

// OUT /////////////////////////////////////////////////////////////////////////

pub struct Out(Any);

impl Out {
    fn new<T>(t: T) -> Self {
        Out(Any::new(t))
    }

    fn take<T>(self) -> T {
        self.0.take()
    }
}

// IMPL ERASED SERDE FOR SERDE /////////////////////////////////////////////////

mod erase {
    pub struct DeserializeSeed<D> {
        pub(crate) state: Option<D>,
    }
    impl<D> DeserializeSeed<D> {
        pub(crate) fn take(&mut self) -> D {
            self.state.take().unwrap()
        }
    }

    pub struct Deserializer<D> {
        pub(crate) state: Option<D>,
    }
    impl<D> Deserializer<D> {
        pub(crate) fn take(&mut self) -> D {
            self.state.take().unwrap()
        }
        pub(crate) fn as_ref(&self) -> &D {
            self.state.as_ref().unwrap()
        }
    }

    pub struct Visitor<D> {
        pub(crate) state: Option<D>,
    }
    impl<D> Visitor<D> {
        pub(crate) fn take(&mut self) -> D {
            self.state.take().unwrap()
        }
        pub(crate) fn as_ref(&self) -> &D {
            self.state.as_ref().unwrap()
        }
    }

    pub struct SeqAccess<D> {
        pub(crate) state: D,
    }
    impl<D> SeqAccess<D> {
        pub(crate) fn as_ref(&self) -> &D {
            &self.state
        }
        pub(crate) fn as_mut(&mut self) -> &mut D {
            &mut self.state
        }
    }

    pub struct MapAccess<D> {
        pub(crate) state: D,
    }
    impl<D> MapAccess<D> {
        pub(crate) fn as_ref(&self) -> &D {
            &self.state
        }
        pub(crate) fn as_mut(&mut self) -> &mut D {
            &mut self.state
        }
    }

    pub struct EnumAccess<D> {
        pub(crate) state: Option<D>,
    }
    impl<D> EnumAccess<D> {
        pub(crate) fn take(&mut self) -> D {
            self.state.take().unwrap()
        }
    }
}

impl<'de, T> DeserializeSeed<'de> for erase::DeserializeSeed<T> where T: serde::de::DeserializeSeed<'de> {
    fn erased_deserialize_seed(&mut self, deserializer: &mut Deserializer<'de>) -> Result<Out, Error> {
        self.take().deserialize(deserializer).map(Out::new)
    }
}

impl<'de, T> Deserializer<'de> for erase::Deserializer<T> where T: serde::Deserializer<'de> {
    fn erased_deserialize_any(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_any(visitor).map_err(erase)
    }
    fn erased_deserialize_bool(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_bool(visitor).map_err(erase)
    }
    fn erased_deserialize_u8(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_u8(visitor).map_err(erase)
    }
    fn erased_deserialize_u16(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_u16(visitor).map_err(erase)
    }
    fn erased_deserialize_u32(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_u32(visitor).map_err(erase)
    }
    fn erased_deserialize_u64(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_u64(visitor).map_err(erase)
    }
    fn erased_deserialize_i8(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_i8(visitor).map_err(erase)
    }
    fn erased_deserialize_i16(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_u16(visitor).map_err(erase)
    }
    fn erased_deserialize_i32(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_i32(visitor).map_err(erase)
    }
    fn erased_deserialize_i64(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_i64(visitor).map_err(erase)
    }
    serde_if_integer128! {
        fn erased_deserialize_i128(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
            self.take().deserialize_i128(visitor).map_err(erase)
        }
        fn erased_deserialize_u128(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
            self.take().deserialize_u128(visitor).map_err(erase)
        }
    }
    fn erased_deserialize_f32(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_f32(visitor).map_err(erase)
    }
    fn erased_deserialize_f64(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_f64(visitor).map_err(erase)
    }
    fn erased_deserialize_char(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_char(visitor).map_err(erase)
    }
    fn erased_deserialize_str(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_str(visitor).map_err(erase)
    }
    fn erased_deserialize_string(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_string(visitor).map_err(erase)
    }
    fn erased_deserialize_bytes(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_bytes(visitor).map_err(erase)
    }
    fn erased_deserialize_byte_buf(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_byte_buf(visitor).map_err(erase)
    }
    fn erased_deserialize_option(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_option(visitor).map_err(erase)
    }
    fn erased_deserialize_unit(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_unit(visitor).map_err(erase)
    }
    fn erased_deserialize_unit_struct(&mut self, name: &'static str, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_unit_struct(name, visitor).map_err(erase)
    }
    fn erased_deserialize_newtype_struct(&mut self, name: &'static str, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_newtype_struct(name, visitor).map_err(erase)
    }
    fn erased_deserialize_seq(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_seq(visitor).map_err(erase)
    }
    fn erased_deserialize_tuple(&mut self, len: usize, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_tuple(len, visitor).map_err(erase)
    }
    fn erased_deserialize_tuple_struct(&mut self, name: &'static str, len: usize, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_tuple_struct(name, len, visitor).map_err(erase)
    }
    fn erased_deserialize_map(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_map(visitor).map_err(erase)
    }
    fn erased_deserialize_struct(&mut self, name: &'static str, fields: &'static [&'static str], visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_struct(name, fields, visitor).map_err(erase)
    }
    fn erased_deserialize_identifier(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_identifier(visitor).map_err(erase)
    }
    fn erased_deserialize_enum(&mut self, name: &'static str, variants: &'static [&'static str], visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_enum(name, variants, visitor).map_err(erase)
    }
    fn erased_deserialize_ignored_any(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
        self.take().deserialize_ignored_any(visitor).map_err(erase)
    }
    fn erased_is_human_readable(&self) -> bool {
        self.as_ref().is_human_readable()
    }
}

impl<'de, T> Visitor<'de> for erase::Visitor<T> where T: serde::de::Visitor<'de> {
    fn erased_expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().expecting(formatter)
    }
    fn erased_visit_bool(&mut self, v: bool) -> Result<Out, Error> {
        self.take().visit_bool(v).map(Out::new)
    }
    fn erased_visit_i8(&mut self, v: i8) -> Result<Out, Error> {
        self.take().visit_i8(v).map(Out::new)
    }
    fn erased_visit_i16(&mut self, v: i16) -> Result<Out, Error> {
        self.take().visit_i16(v).map(Out::new)
    }
    fn erased_visit_i32(&mut self, v: i32) -> Result<Out, Error> {
        self.take().visit_i32(v).map(Out::new)
    }
    fn erased_visit_i64(&mut self, v: i64) -> Result<Out, Error> {
        self.take().visit_i64(v).map(Out::new)
    }
    fn erased_visit_u8(&mut self, v: u8) -> Result<Out, Error> {
        self.take().visit_u8(v).map(Out::new)
    }
    fn erased_visit_u16(&mut self, v: u16) -> Result<Out, Error> {
        self.take().visit_u16(v).map(Out::new)
    }
    fn erased_visit_u32(&mut self, v: u32) -> Result<Out, Error> {
        self.take().visit_u32(v).map(Out::new)
    }
    fn erased_visit_u64(&mut self, v: u64) -> Result<Out, Error> {
        self.take().visit_u64(v).map(Out::new)
    }
    serde_if_integer128! {
        fn erased_visit_i128(&mut self, v: i128) -> Result<Out, Error> {
            self.take().visit_i128(v).map(Out::new)
        }
        fn erased_visit_u128(&mut self, v: u128) -> Result<Out, Error> {
            self.take().visit_u128(v).map(Out::new)
        }
    }
    fn erased_visit_f32(&mut self, v: f32) -> Result<Out, Error> {
        self.take().visit_f32(v).map(Out::new)
    }
    fn erased_visit_f64(&mut self, v: f64) -> Result<Out, Error> {
        self.take().visit_f64(v).map(Out::new)
    }
    fn erased_visit_char(&mut self, v: char) -> Result<Out, Error> {
        self.take().visit_char(v).map(Out::new)
    }
    fn erased_visit_str(&mut self, v: &str) -> Result<Out, Error> {
        self.take().visit_str(v).map(Out::new)
    }
    fn erased_visit_borrowed_str(&mut self, v: &'de str) -> Result<Out, Error> {
        self.take().visit_borrowed_str(v).map(Out::new)
    }
    fn erased_visit_string(&mut self, v: String) -> Result<Out, Error> {
        self.take().visit_string(v).map(Out::new)
    }
    fn erased_visit_bytes(&mut self, v: &[u8]) -> Result<Out, Error> {
        self.take().visit_bytes(v).map(Out::new)
    }
    fn erased_visit_borrowed_bytes(&mut self, v: &'de [u8]) -> Result<Out, Error> {
        self.take().visit_borrowed_bytes(v).map(Out::new)
    }
    fn erased_visit_byte_buf(&mut self, v: Vec<u8>) -> Result<Out, Error> {
        self.take().visit_byte_buf(v).map(Out::new)
    }
    fn erased_visit_none(&mut self) -> Result<Out, Error> {
        self.take().visit_none().map(Out::new)
    }
    fn erased_visit_some(&mut self, deserializer: &mut Deserializer<'de>) -> Result<Out, Error> {
        self.take().visit_some(deserializer).map(Out::new)
    }
    fn erased_visit_unit(&mut self) -> Result<Out, Error> {
        self.take().visit_unit().map(Out::new)
    }
    fn erased_visit_newtype_struct(&mut self, deserializer: &mut Deserializer<'de>) -> Result<Out, Error> {
        self.take().visit_newtype_struct(deserializer).map(Out::new)
    }
    fn erased_visit_seq(&mut self, seq: &mut SeqAccess<'de>) -> Result<Out, Error> {
        self.take().visit_seq(seq).map(Out::new)
    }
    fn erased_visit_map(&mut self, map: &mut MapAccess<'de>) -> Result<Out, Error> {
        self.take().visit_map(map).map(Out::new)
    }
    fn erased_visit_enum(&mut self, data: &mut EnumAccess<'de>) -> Result<Out, Error> {
        self.take().visit_enum(data).map(Out::new)
    }
}

impl<'de, T> SeqAccess<'de> for erase::SeqAccess<T> where T: serde::de::SeqAccess<'de> {
    fn erased_next_element(&mut self, seed: &mut DeserializeSeed<'de>) -> Result<Option<Out>, Error> {
        self.as_mut().next_element_seed(seed).map_err(erase)
    }
    fn erased_size_hint(&self) -> Option<usize> {
        self.as_ref().size_hint()
    }
}

impl<'de, T> MapAccess<'de> for erase::MapAccess<T> where T: serde::de::MapAccess<'de> {
    fn erased_next_key(&mut self, seed: &mut DeserializeSeed<'de>) -> Result<Option<Out>, Error> {
        self.as_mut().next_key_seed(seed).map_err(erase)
    }
    fn erased_next_value(&mut self, seed: &mut DeserializeSeed<'de>) -> Result<Out, Error> {
        self.as_mut().next_value_seed(seed).map_err(erase)
    }
    fn erased_next_entry(&mut self, k: &mut DeserializeSeed<'de>, v: &mut DeserializeSeed<'de>) -> Result<Option<(Out, Out)>, Error> {
        self.as_mut().next_entry_seed(k, v).map_err(erase)
    }
    fn erased_size_hint(&self) -> Option<usize> {
        self.as_ref().size_hint()
    }
}

impl<'de, T> EnumAccess<'de> for erase::EnumAccess<T> where T: serde::de::EnumAccess<'de> {
    fn erased_variant_seed(&mut self, seed: &mut DeserializeSeed<'de>) -> Result<(Out, Variant<'de>), Error> {
        self.take().variant_seed(seed).map(|(out, variant)| {
            use serde::de::VariantAccess;
            let erased = Variant {
                data: Any::new(variant),
                unit_variant: |a| {
                    a.take::<T::Variant>().unit_variant().map_err(erase)
                },
                visit_newtype: |a, seed| {
                    a.take::<T::Variant>().newtype_variant_seed(seed).map_err(erase)
                },
                tuple_variant: |a, len, visitor| {
                    a.take::<T::Variant>().tuple_variant(len, visitor).map_err(erase)
                },
                struct_variant: |a, fields, visitor| {
                    a.take::<T::Variant>().struct_variant(fields, visitor).map_err(erase)
                },
            };
            (out, erased)
        }).map_err(erase)
    }
}

// IMPL SERDE FOR ERASED SERDE /////////////////////////////////////////////////

impl<'de, 'a> serde::de::DeserializeSeed<'de> for &'a mut DeserializeSeed<'de> {
    type Value = Out;
    fn deserialize<D>(self, deserializer: D) -> Result<Out, D::Error> where D: serde::Deserializer<'de> {
        let mut erased = erase::Deserializer { state: Some(deserializer) };
        self.erased_deserialize_seed(&mut erased).map_err(unerase)
    }
}

macro_rules! impl_deserializer_for_trait_object {
    ({$($generics:tt)*} {$($mut:tt)*} $ty:ty) => {
        impl <$($generics)*> serde::Deserializer<'de> for $ty {
            type Error = Error;
            fn deserialize_any<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_any(&mut erased).map(Out::take)
            }
            fn deserialize_bool<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_bool(&mut erased).map(Out::take)
            }
            fn deserialize_u8<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_u8(&mut erased).map(Out::take)
            }
            fn deserialize_u16<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_u16(&mut erased).map(Out::take)
            }
            fn deserialize_u32<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_u32(&mut erased).map(Out::take)
            }
            fn deserialize_u64<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_u64(&mut erased).map(Out::take)
            }
            fn deserialize_i8<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_i8(&mut erased).map(Out::take)
            }
            fn deserialize_i16<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_i16(&mut erased).map(Out::take)
            }
            fn deserialize_i32<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_i32(&mut erased).map(Out::take)
            }
            fn deserialize_i64<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_i64(&mut erased).map(Out::take)
            }
            serde_if_integer128! {
                fn deserialize_i128<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                    let mut erased = erase::Visitor { state: Some(visitor) };
                    self.erased_deserialize_i128(&mut erased).map(Out::take)
                }
                fn deserialize_u128<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                    let mut erased = erase::Visitor { state: Some(visitor) };
                    self.erased_deserialize_u128(&mut erased).map(Out::take)
                }
            }
            fn deserialize_f32<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_f32(&mut erased).map(Out::take)
            }
            fn deserialize_f64<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_f64(&mut erased).map(Out::take)
            }
            fn deserialize_char<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_char(&mut erased).map(Out::take)
            }
            fn deserialize_str<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_str(&mut erased).map(Out::take)
            }
            fn deserialize_string<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_string(&mut erased).map(Out::take)
            }
            fn deserialize_bytes<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_bytes(&mut erased).map(Out::take)
            }
            fn deserialize_byte_buf<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_byte_buf(&mut erased).map(Out::take)
            }
            fn deserialize_option<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_option(&mut erased).map(Out::take)
            }
            fn deserialize_unit<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_unit(&mut erased).map(Out::take)
            }
            fn deserialize_unit_struct<V>($($mut)* self, name: &'static str, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_unit_struct(name, &mut erased).map(Out::take)
            }
            fn deserialize_newtype_struct<V>($($mut)* self, name: &'static str, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_newtype_struct(name, &mut erased).map(Out::take)
            }
            fn deserialize_seq<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_seq(&mut erased).map(Out::take)
            }
            fn deserialize_tuple<V>($($mut)* self, len: usize, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_tuple(len, &mut erased).map(Out::take)
            }
            fn deserialize_tuple_struct<V>($($mut)* self, name: &'static str, len: usize, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_tuple_struct(name, len, &mut erased).map(Out::take)
            }
            fn deserialize_map<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_map(&mut erased).map(Out::take)
            }
            fn deserialize_struct<V>($($mut)* self, name: &'static str, fields: &'static [&'static str], visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_struct(name, fields, &mut erased).map(Out::take)
            }
            fn deserialize_identifier<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_identifier(&mut erased).map(Out::take)
            }
            fn deserialize_enum<V>($($mut)* self, name: &'static str, variants: &'static [&'static str], visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_enum(name, variants, &mut erased).map(Out::take)
            }
            fn deserialize_ignored_any<V>($($mut)* self, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
                let mut erased = erase::Visitor { state: Some(visitor) };
                self.erased_deserialize_ignored_any(&mut erased).map(Out::take)
            }
            fn is_human_readable(&self) -> bool {
                self.erased_is_human_readable()
            }
        }
    };
}

impl_deserializer_for_trait_object!({'de, 'a} {} &'a mut Deserializer<'de>);
impl_deserializer_for_trait_object!({'de, 'a} {} &'a mut (Deserializer<'de> + Send));
impl_deserializer_for_trait_object!({'de, 'a} {} &'a mut (Deserializer<'de> + Sync));
impl_deserializer_for_trait_object!({'de, 'a} {} &'a mut (Deserializer<'de> + Send + Sync));
impl_deserializer_for_trait_object!({'de} {mut} Box<Deserializer<'de>>);
impl_deserializer_for_trait_object!({'de} {mut} Box<Deserializer<'de> + Send>);
impl_deserializer_for_trait_object!({'de} {mut} Box<Deserializer<'de> + Sync>);
impl_deserializer_for_trait_object!({'de} {mut} Box<Deserializer<'de> + Send + Sync>);

impl<'de, 'a> serde::de::Visitor<'de> for &'a mut Visitor<'de> {
    type Value = Out;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        (**self).erased_expecting(formatter)
    }
    fn visit_bool<E>(self, v: bool) -> Result<Out, E> where E: serde::de::Error {
        self.erased_visit_bool(v).map_err(unerase)
    }
    fn visit_i8<E>(self, v: i8) -> Result<Out, E> where E: serde::de::Error {
        self.erased_visit_i8(v).map_err(unerase)
    }
    fn visit_i16<E>(self, v: i16) -> Result<Out, E> where E: serde::de::Error {
        self.erased_visit_i16(v).map_err(unerase)
    }
    fn visit_i32<E>(self, v: i32) -> Result<Out, E> where E: serde::de::Error {
        self.erased_visit_i32(v).map_err(unerase)
    }
    fn visit_i64<E>(self, v: i64) -> Result<Out, E> where E: serde::de::Error {
        self.erased_visit_i64(v).map_err(unerase)
    }
    fn visit_u8<E>(self, v: u8) -> Result<Out, E> where E: serde::de::Error {
        self.erased_visit_u8(v).map_err(unerase)
    }
    fn visit_u16<E>(self, v: u16) -> Result<Out, E> where E: serde::de::Error {
        self.erased_visit_u16(v).map_err(unerase)
    }
    fn visit_u32<E>(self, v: u32) -> Result<Out, E> where E: serde::de::Error {
        self.erased_visit_u32(v).map_err(unerase)
    }
    fn visit_u64<E>(self, v: u64) -> Result<Out, E> where E: serde::de::Error {
        self.erased_visit_u64(v).map_err(unerase)
    }
    serde_if_integer128! {
        fn visit_i128<E>(self, v: i128) -> Result<Out, E> where E: serde::de::Error {
            self.erased_visit_i128(v).map_err(unerase)
        }
        fn visit_u128<E>(self, v: u128) -> Result<Out, E> where E: serde::de::Error {
            self.erased_visit_u128(v).map_err(unerase)
        }
    }
    fn visit_f32<E>(self, v: f32) -> Result<Out, E> where E: serde::de::Error {
        self.erased_visit_f32(v).map_err(unerase)
    }
    fn visit_f64<E>(self, v: f64) -> Result<Out, E> where E: serde::de::Error {
        self.erased_visit_f64(v).map_err(unerase)
    }
    fn visit_char<E>(self, v: char) -> Result<Out, E> where E: serde::de::Error {
        self.erased_visit_char(v).map_err(unerase)
    }
    fn visit_str<E>(self, v: &str) -> Result<Out, E> where E: serde::de::Error {
        self.erased_visit_str(v).map_err(unerase)
    }
    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Out, E> where E: serde::de::Error {
        self.erased_visit_borrowed_str(v).map_err(unerase)
    }
    fn visit_string<E>(self, v: String) -> Result<Out, E> where E: serde::de::Error {
        self.erased_visit_string(v).map_err(unerase)
    }
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Out, E> where E: serde::de::Error {
        self.erased_visit_bytes(v).map_err(unerase)
    }
    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Out, E> where E: serde::de::Error {
        self.erased_visit_borrowed_bytes(v).map_err(unerase)
    }
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Out, E> where E: serde::de::Error {
        self.erased_visit_byte_buf(v).map_err(unerase)
    }
    fn visit_none<E>(self) -> Result<Out, E> where E: serde::de::Error {
        self.erased_visit_none().map_err(unerase)
    }
    fn visit_some<D>(self, deserializer: D) -> Result<Out, D::Error> where D: serde::Deserializer<'de> {
        let mut erased = erase::Deserializer { state: Some(deserializer) };
        self.erased_visit_some(&mut erased).map_err(unerase)
    }
    fn visit_unit<E>(self) -> Result<Out, E> where E: serde::de::Error {
        self.erased_visit_unit().map_err(unerase)
    }
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Out, D::Error> where D: serde::Deserializer<'de> {
        let mut erased = erase::Deserializer { state: Some(deserializer) };
        self.erased_visit_newtype_struct(&mut erased).map_err(unerase)
    }
    fn visit_seq<V>(self, seq: V) -> Result<Out, V::Error> where V: serde::de::SeqAccess<'de> {
        let mut erased = erase::SeqAccess { state: seq };
        self.erased_visit_seq(&mut erased).map_err(unerase)
    }
    fn visit_map<V>(self, map: V) -> Result<Out, V::Error> where V: serde::de::MapAccess<'de> {
        let mut erased = erase::MapAccess { state: map };
        self.erased_visit_map(&mut erased).map_err(unerase)
    }
    fn visit_enum<V>(self, data: V) -> Result<Out, V::Error> where V: serde::de::EnumAccess<'de> {
        let mut erased = erase::EnumAccess { state: Some(data) };
        self.erased_visit_enum(&mut erased).map_err(unerase)
    }
}

impl<'de, 'a> serde::de::SeqAccess<'de> for &'a mut SeqAccess<'de> {
    type Error = Error;
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error> where T: serde::de::DeserializeSeed<'de> {
        let mut seed = erase::DeserializeSeed { state: Some(seed) };
        (**self).erased_next_element(&mut seed).map(|opt| opt.map(Out::take))
    }
    fn size_hint(&self) -> Option<usize> {
        (**self).erased_size_hint()
    }
}

impl<'de, 'a> serde::de::MapAccess<'de> for &'a mut MapAccess<'de> {
    type Error = Error;
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error> where K: serde::de::DeserializeSeed<'de> {
        let mut erased = erase::DeserializeSeed { state: Some(seed) };
        (**self).erased_next_key(&mut erased).map(|opt| opt.map(Out::take))
    }
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error> where V: serde::de::DeserializeSeed<'de> {
        let mut erased = erase::DeserializeSeed { state: Some(seed) };
        (**self).erased_next_value(&mut erased).map(Out::take)
    }
    fn size_hint(&self) -> Option<usize> {
        (**self).erased_size_hint()
    }
}

impl<'de, 'a> serde::de::EnumAccess<'de> for &'a mut EnumAccess<'de> {
    type Error = Error;
    type Variant = Variant<'de>;
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error> where V: serde::de::DeserializeSeed<'de> {
        let mut erased = erase::DeserializeSeed { state: Some(seed) };
        self.erased_variant_seed(&mut erased).map(|(out, variant)| (out.take(), variant))
    }
}

pub struct Variant<'de> {
    data: Any,
    unit_variant: fn(Any) -> Result<(), Error>,
    visit_newtype: fn(Any, seed: &mut DeserializeSeed<'de>) -> Result<Out, Error>,
    tuple_variant: fn(Any, len: usize, visitor: &mut Visitor<'de>) -> Result<Out, Error>,
    struct_variant: fn(Any, fields: &'static [&'static str], visitor: &mut Visitor<'de>) -> Result<Out, Error>,
}

impl<'de> serde::de::VariantAccess<'de> for Variant<'de> {
    type Error = Error;
    fn unit_variant(self) -> Result<(), Error> {
        (self.unit_variant)(self.data)
    }
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error> where T: serde::de::DeserializeSeed<'de> {
        let mut erased = erase::DeserializeSeed { state: Some(seed) };
        (self.visit_newtype)(self.data, &mut erased).map(Out::take)
    }
    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
        let mut erased = erase::Visitor { state: Some(visitor) };
        (self.tuple_variant)(self.data, len, &mut erased).map(Out::take)
    }
    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value, Error> where V: serde::de::Visitor<'de> {
        let mut erased = erase::Visitor { state: Some(visitor) };
        (self.struct_variant)(self.data, fields, &mut erased).map(Out::take)
    }
}

// IMPL ERASED SERDE FOR ERASED SERDE //////////////////////////////////////////

macro_rules! deref_erased_deserializer {
    ($($imp:tt)+) => {
        impl $($imp)+ {
            fn erased_deserialize_any(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_any(visitor)
            }
            fn erased_deserialize_bool(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_bool(visitor)
            }
            fn erased_deserialize_u8(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_u8(visitor)
            }
            fn erased_deserialize_u16(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_u16(visitor)
            }
            fn erased_deserialize_u32(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_u32(visitor)
            }
            fn erased_deserialize_u64(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_u64(visitor)
            }
            fn erased_deserialize_i8(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_i8(visitor)
            }
            fn erased_deserialize_i16(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_i16(visitor)
            }
            fn erased_deserialize_i32(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_i32(visitor)
            }
            fn erased_deserialize_i64(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_i64(visitor)
            }
            serde_if_integer128! {
                fn erased_deserialize_i128(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                    (**self).erased_deserialize_i128(visitor)
                }
                fn erased_deserialize_u128(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                    (**self).erased_deserialize_u128(visitor)
                }
            }
            fn erased_deserialize_f32(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_f32(visitor)
            }
            fn erased_deserialize_f64(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_f64(visitor)
            }
            fn erased_deserialize_char(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_char(visitor)
            }
            fn erased_deserialize_str(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_str(visitor)
            }
            fn erased_deserialize_string(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_string(visitor)
            }
            fn erased_deserialize_bytes(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_bytes(visitor)
            }
            fn erased_deserialize_byte_buf(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_byte_buf(visitor)
            }
            fn erased_deserialize_option(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_option(visitor)
            }
            fn erased_deserialize_unit(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_unit(visitor)
            }
            fn erased_deserialize_unit_struct(&mut self, name: &'static str, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_unit_struct(name, visitor)
            }
            fn erased_deserialize_newtype_struct(&mut self, name: &'static str, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_newtype_struct(name, visitor)
            }
            fn erased_deserialize_seq(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_seq(visitor)
            }
            fn erased_deserialize_tuple(&mut self, len: usize, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_tuple(len, visitor)
            }
            fn erased_deserialize_tuple_struct(&mut self, name: &'static str, len: usize, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_tuple_struct(name, len, visitor)
            }
            fn erased_deserialize_map(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_map(visitor)
            }
            fn erased_deserialize_struct(&mut self, name: &'static str, fields: &'static [&'static str], visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_struct(name, fields, visitor)
            }
            fn erased_deserialize_identifier(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_identifier(visitor)
            }
            fn erased_deserialize_enum(&mut self, name: &'static str, variants: &'static [&'static str], visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_enum(name, variants, visitor)
            }
            fn erased_deserialize_ignored_any(&mut self, visitor: &mut Visitor<'de>) -> Result<Out, Error> {
                (**self).erased_deserialize_ignored_any(visitor)
            }
            fn erased_is_human_readable(&self) -> bool {
                (**self).erased_is_human_readable()
            }
        }
    };
}

deref_erased_deserializer!(<'de, 'a> Deserializer<'de> for Box<Deserializer<'de> + 'a>);
deref_erased_deserializer!(<'de, 'a> Deserializer<'de> for Box<Deserializer<'de> + Send + 'a>);
deref_erased_deserializer!(<'de, 'a> Deserializer<'de> for Box<Deserializer<'de> + Sync + 'a>);
deref_erased_deserializer!(<'de, 'a> Deserializer<'de> for Box<Deserializer<'de> + Send + Sync + 'a>);
deref_erased_deserializer!(<'de, 'a, T: ?Sized + Deserializer<'de>> Deserializer<'de> for &'a mut T);

// ERROR ///////////////////////////////////////////////////////////////////////

fn erase<E>(e: E) -> Error
    where E: Display
{
    serde::de::Error::custom(e)
}

fn unerase<E>(e: Error) -> E
    where E: serde::de::Error
{
    use std::error::Error;
    E::custom(e.description())
}

// TEST ////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::fmt::Debug;

    fn test_json<'de, T>(json: &'de [u8])
        where T: serde::Deserialize<'de> + PartialEq + Debug
    {
        let expected: T = serde_json::from_slice(json).unwrap();

        // test borrowed trait object
        {
            let mut de = serde_json::Deserializer::from_slice(json);
            let de: &mut Deserializer = &mut Deserializer::erase(&mut de);
            assert_eq!(expected, deserialize::<T>(de).unwrap());
        }

        // test boxed trait object
        {
            let mut de = serde_json::Deserializer::from_slice(json);
            let mut de: Box<Deserializer> = Box::new(Deserializer::erase(&mut de));
            assert_eq!(expected, deserialize::<T>(&mut de).unwrap());
        }
    }

    #[test]
    fn test_value() {
        test_json::<serde_json::Value>(br#"["a", 1, [true], {"a": 1}]"#);
    }

    #[test]
    fn test_struct() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct S {
            f: usize,
        }

        test_json::<S>(br#"{"f":256}"#);
    }

    #[test]
    fn test_enum() {
        #[derive(Deserialize, PartialEq, Debug)]
        enum E {
            Unit,
            Newtype(bool),
            Tuple(bool, bool),
            Struct { t: bool, f: bool },
        }

        test_json::<E>(br#""Unit""#);
        test_json::<E>(br#"{"Newtype":true}"#);
        test_json::<E>(br#"{"Tuple":[true,false]}"#);
        test_json::<E>(br#"{"Struct":{"t":true,"f":false}}"#);
    }

    #[test]
    fn test_borrowed() {
        let bytes = br#""borrowed""#.to_owned();
        test_json::<&str>(&bytes);
    }

    #[test]
    fn assert_deserializer() {
        fn assert<'de, T: serde::Deserializer<'de>>() {}

        assert::<&mut Deserializer>();
        assert::<&mut (Deserializer + Send)>();
        assert::<&mut (Deserializer + Sync)>();
        assert::<&mut (Deserializer + Send + Sync)>();
        assert::<&mut (Deserializer + Sync + Send)>();

        assert::<Box<Deserializer>>();
        assert::<Box<Deserializer + Send>>();
        assert::<Box<Deserializer + Sync>>();
        assert::<Box<Deserializer + Send + Sync>>();
        assert::<Box<Deserializer + Sync + Send>>();
    }
}
