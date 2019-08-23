use std::prelude::v1::*;
use std::fmt::Display;
use std::marker::PhantomData;

use serde;
use serde::ser::{SerializeSeq, SerializeTuple, SerializeTupleStruct,
                 SerializeTupleVariant, SerializeMap, SerializeStruct,
                 SerializeStructVariant};

use any::Any;
use error::Error;

// TRAITS //////////////////////////////////////////////////////////////////////

/// An object-safe equivalent of Serde's `Serialize` trait.
///
/// Any implementation of Serde's `Serialize` converts seamlessly to an
/// `&erased_serde::Serialize` or `Box<erased_serde::Serialize>` trait object.
///
/// ```rust
/// extern crate erased_serde;
/// extern crate serde_json;
/// extern crate serde_cbor;
///
/// use std::collections::BTreeMap as Map;
/// use std::io;
///
/// use erased_serde::{Serialize, Serializer};
///
/// fn main() {
///     // Construct some serializers.
///     let json = &mut serde_json::ser::Serializer::new(io::stdout());
///     let cbor = &mut serde_cbor::ser::Serializer::new(io::stdout());
///
///     // The values in this map are boxed trait objects. Ordinarily this would not
///     // be possible with serde::Serializer because of object safety, but type
///     // erasure makes it possible with erased_serde::Serializer.
///     let mut formats: Map<&str, Box<Serializer>> = Map::new();
///     formats.insert("json", Box::new(Serializer::erase(json)));
///     formats.insert("cbor", Box::new(Serializer::erase(cbor)));
///
///     // These are boxed trait objects as well. Same thing here - type erasure
///     // makes this possible.
///     let mut values: Map<&str, Box<Serialize>> = Map::new();
///     values.insert("vec", Box::new(vec!["a", "b"]));
///     values.insert("int", Box::new(65536));
///
///     // Pick a Serializer out of the formats map.
///     let format = formats.get_mut("json").unwrap();
///
///     // Pick a Serialize out of the values map.
///     let value = values.get("vec").unwrap();
///
///     // This line prints `["a","b"]` to stdout.
///     value.erased_serialize(format).unwrap();
/// }
/// ```
pub trait Serialize {
    fn erased_serialize(&self, &mut dyn Serializer) -> Result<Ok, Error>;
}

/// An object-safe equivalent of Serde's `Serializer` trait.
///
/// Any implementation of Serde's `Serializer` can be converted to an
/// `&erased_serde::Serializer` or `Box<erased_serde::Serializer>` trait object
/// using `erased_serde::Serializer::erase`.
///
/// ```rust
/// extern crate erased_serde;
/// extern crate serde_json;
/// extern crate serde_cbor;
///
/// use std::collections::BTreeMap as Map;
/// use std::io;
///
/// use erased_serde::{Serialize, Serializer};
///
/// fn main() {
///     // Construct some serializers.
///     let json = &mut serde_json::ser::Serializer::new(io::stdout());
///     let cbor = &mut serde_cbor::ser::Serializer::new(io::stdout());
///
///     // The values in this map are boxed trait objects. Ordinarily this would not
///     // be possible with serde::Serializer because of object safety, but type
///     // erasure makes it possible with erased_serde::Serializer.
///     let mut formats: Map<&str, Box<Serializer>> = Map::new();
///     formats.insert("json", Box::new(Serializer::erase(json)));
///     formats.insert("cbor", Box::new(Serializer::erase(cbor)));
///
///     // These are boxed trait objects as well. Same thing here - type erasure
///     // makes this possible.
///     let mut values: Map<&str, Box<Serialize>> = Map::new();
///     values.insert("vec", Box::new(vec!["a", "b"]));
///     values.insert("int", Box::new(65536));
///
///     // Pick a Serializer out of the formats map.
///     let format = formats.get_mut("json").unwrap();
///
///     // Pick a Serialize out of the values map.
///     let value = values.get("vec").unwrap();
///
///     // This line prints `["a","b"]` to stdout.
///     value.erased_serialize(format).unwrap();
/// }
/// ```
pub trait Serializer {
    fn erased_serialize_bool(&mut self, bool) -> Result<Ok, Error>;
    fn erased_serialize_i8(&mut self, i8) -> Result<Ok, Error>;
    fn erased_serialize_i16(&mut self, i16) -> Result<Ok, Error>;
    fn erased_serialize_i32(&mut self, i32) -> Result<Ok, Error>;
    fn erased_serialize_i64(&mut self, i64) -> Result<Ok, Error>;
    fn erased_serialize_u8(&mut self, u8) -> Result<Ok, Error>;
    fn erased_serialize_u16(&mut self, u16) -> Result<Ok, Error>;
    fn erased_serialize_u32(&mut self, u32) -> Result<Ok, Error>;
    fn erased_serialize_u64(&mut self, u64) -> Result<Ok, Error>;
    serde_if_integer128! {
        fn erased_serialize_i128(&mut self, i128) -> Result<Ok, Error>;
        fn erased_serialize_u128(&mut self, u128) -> Result<Ok, Error>;
    }
    fn erased_serialize_f32(&mut self, f32) -> Result<Ok, Error>;
    fn erased_serialize_f64(&mut self, f64) -> Result<Ok, Error>;
    fn erased_serialize_char(&mut self, char) -> Result<Ok, Error>;
    fn erased_serialize_str(&mut self, &str) -> Result<Ok, Error>;
    fn erased_serialize_bytes(&mut self, &[u8]) -> Result<Ok, Error>;
    fn erased_serialize_none(&mut self) -> Result<Ok, Error>;
    fn erased_serialize_some(&mut self, &dyn Serialize) -> Result<Ok, Error>;
    fn erased_serialize_unit(&mut self) -> Result<Ok, Error>;
    fn erased_serialize_unit_struct(&mut self, name: &'static str) -> Result<Ok, Error>;
    fn erased_serialize_unit_variant(&mut self, name: &'static str, variant_index: u32, variant: &'static str) -> Result<Ok, Error>;
    fn erased_serialize_newtype_struct(&mut self, name: &'static str, &dyn Serialize) -> Result<Ok, Error>;
    fn erased_serialize_newtype_variant(&mut self, name: &'static str, variant_index: u32, variant: &'static str, &dyn Serialize) -> Result<Ok, Error>;
    fn erased_serialize_seq(&mut self, len: Option<usize>) -> Result<Seq, Error>;
    fn erased_serialize_tuple(&mut self, len: usize) -> Result<Tuple, Error>;
    fn erased_serialize_tuple_struct(&mut self, name: &'static str, len: usize) -> Result<TupleStruct, Error>;
    fn erased_serialize_tuple_variant(&mut self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<TupleVariant, Error>;
    fn erased_serialize_map(&mut self, len: Option<usize>) -> Result<Map, Error>;
    fn erased_serialize_struct(&mut self, name: &'static str, len: usize) -> Result<Struct, Error>;
    fn erased_serialize_struct_variant(&mut self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<StructVariant, Error>;
    fn erased_is_human_readable(&self) -> bool;
}

impl dyn Serializer {
    /// Convert any Serde `Serializer` to a trait object.
    ///
    /// ```rust
    /// extern crate erased_serde;
    /// extern crate serde_json;
    /// extern crate serde_cbor;
    ///
    /// use std::collections::BTreeMap as Map;
    /// use std::io;
    ///
    /// use erased_serde::{Serialize, Serializer};
    ///
    /// fn main() {
    ///     // Construct some serializers.
    ///     let json = &mut serde_json::ser::Serializer::new(io::stdout());
    ///     let cbor = &mut serde_cbor::ser::Serializer::new(io::stdout());
    ///
    ///     // The values in this map are boxed trait objects. Ordinarily this would not
    ///     // be possible with serde::Serializer because of object safety, but type
    ///     // erasure makes it possible with erased_serde::Serializer.
    ///     let mut formats: Map<&str, Box<Serializer>> = Map::new();
    ///     formats.insert("json", Box::new(Serializer::erase(json)));
    ///     formats.insert("cbor", Box::new(Serializer::erase(cbor)));
    ///
    ///     // These are boxed trait objects as well. Same thing here - type erasure
    ///     // makes this possible.
    ///     let mut values: Map<&str, Box<Serialize>> = Map::new();
    ///     values.insert("vec", Box::new(vec!["a", "b"]));
    ///     values.insert("int", Box::new(65536));
    ///
    ///     // Pick a Serializer out of the formats map.
    ///     let format = formats.get_mut("json").unwrap();
    ///
    ///     // Pick a Serialize out of the values map.
    ///     let value = values.get("vec").unwrap();
    ///
    ///     // This line prints `["a","b"]` to stdout.
    ///     value.erased_serialize(format).unwrap();
    /// }
    /// ```
    pub fn erase<S>(serializer: S) -> erase::Serializer<S>
        where S: serde::Serializer,
              S::Ok: 'static,
    {
        erase::Serializer {
            state: Some(serializer),
        }
    }
}

// OK //////////////////////////////////////////////////////////////////////////

// Corresponds to the Serializer::Ok associated type.
//
// This struct is exposed to users by invoking methods on the Serialize or
// Serializer trait objects, so we need to make sure they do not hold on to the
// Ok beyond the lifetime of the data in the Any.
//
// We do this by enforcing S::Ok is 'static for every Serializer trait object
// created by the user.
pub struct Ok {
    data: Any,
}

impl Ok {
    fn new<T>(t: T) -> Self {
        Ok {
            data: Any::new(t),
        }
    }

    fn take<T>(self) -> T {
        self.data.take()
    }
}

// IMPL ERASED SERDE FOR SERDE /////////////////////////////////////////////////

impl<T: ?Sized> Serialize for T where T: serde::Serialize {
    fn erased_serialize(&self, serializer: &mut dyn Serializer) -> Result<Ok, Error> {
        self.serialize(serializer)
    }
}

mod erase {
    pub struct Serializer<S> {
        pub(crate) state: Option<S>,
    }
    impl<S> Serializer<S> {
        pub(crate) fn take(&mut self) -> S {
            self.state.take().unwrap()
        }
        pub(crate) fn as_ref(&self) -> &S {
            self.state.as_ref().unwrap()
        }
    }
}

impl<T> Serializer for erase::Serializer<T> where T: serde::Serializer {
    fn erased_serialize_bool(&mut self, v: bool) -> Result<Ok, Error> {
        self.take().serialize_bool(v).map(Ok::new).map_err(erase)
    }
    fn erased_serialize_i8(&mut self, v: i8) -> Result<Ok, Error> {
        self.take().serialize_i8(v).map(Ok::new).map_err(erase)
    }
    fn erased_serialize_i16(&mut self, v: i16) -> Result<Ok, Error> {
        self.take().serialize_i16(v).map(Ok::new).map_err(erase)
    }
    fn erased_serialize_i32(&mut self, v: i32) -> Result<Ok, Error> {
        self.take().serialize_i32(v).map(Ok::new).map_err(erase)
    }
    fn erased_serialize_i64(&mut self, v: i64) -> Result<Ok, Error> {
        self.take().serialize_i64(v).map(Ok::new).map_err(erase)
    }
    fn erased_serialize_u8(&mut self, v: u8) -> Result<Ok, Error> {
        self.take().serialize_u8(v).map(Ok::new).map_err(erase)
    }
    fn erased_serialize_u16(&mut self, v: u16) -> Result<Ok, Error> {
        self.take().serialize_u16(v).map(Ok::new).map_err(erase)
    }
    fn erased_serialize_u32(&mut self, v: u32) -> Result<Ok, Error> {
        self.take().serialize_u32(v).map(Ok::new).map_err(erase)
    }
    fn erased_serialize_u64(&mut self, v: u64) -> Result<Ok, Error> {
        self.take().serialize_u64(v).map(Ok::new).map_err(erase)
    }
    serde_if_integer128! {
        fn erased_serialize_i128(&mut self, v: i128) -> Result<Ok, Error> {
            self.take().serialize_i128(v).map(Ok::new).map_err(erase)
        }
        fn erased_serialize_u128(&mut self, v: u128) -> Result<Ok, Error> {
            self.take().serialize_u128(v).map(Ok::new).map_err(erase)
        }
    }
    fn erased_serialize_f32(&mut self, v: f32) -> Result<Ok, Error> {
        self.take().serialize_f32(v).map(Ok::new).map_err(erase)
    }
    fn erased_serialize_f64(&mut self, v: f64) -> Result<Ok, Error> {
        self.take().serialize_f64(v).map(Ok::new).map_err(erase)
    }
    fn erased_serialize_char(&mut self, v: char) -> Result<Ok, Error> {
        self.take().serialize_char(v).map(Ok::new).map_err(erase)
    }
    fn erased_serialize_str(&mut self, v: &str) -> Result<Ok, Error> {
        self.take().serialize_str(v).map(Ok::new).map_err(erase)
    }
    fn erased_serialize_bytes(&mut self, v: &[u8]) -> Result<Ok, Error> {
        self.take().serialize_bytes(v).map(Ok::new).map_err(erase)
    }
    fn erased_serialize_none(&mut self) -> Result<Ok, Error> {
        self.take().serialize_none().map(Ok::new).map_err(erase)
    }
    fn erased_serialize_some(&mut self, v: &dyn Serialize) -> Result<Ok, Error> {
        self.take().serialize_some(v).map(Ok::new).map_err(erase)
    }
    fn erased_serialize_unit(&mut self) -> Result<Ok, Error> {
        self.take().serialize_unit().map(Ok::new).map_err(erase)
    }
    fn erased_serialize_unit_struct(&mut self, name: &'static str) -> Result<Ok, Error> {
        self.take().serialize_unit_struct(name).map(Ok::new).map_err(erase)
    }
    fn erased_serialize_unit_variant(&mut self, name: &'static str, variant_index: u32, variant: &'static str) -> Result<Ok, Error> {
        self.take().serialize_unit_variant(name, variant_index, variant).map(Ok::new).map_err(erase)
    }
    fn erased_serialize_newtype_struct(&mut self, name: &'static str, v: &dyn Serialize) -> Result<Ok, Error> {
        self.take().serialize_newtype_struct(name, v).map(Ok::new).map_err(erase)
    }
    fn erased_serialize_newtype_variant(&mut self, name: &'static str, variant_index: u32, variant: &'static str, v: &dyn Serialize) -> Result<Ok, Error> {
        self.take().serialize_newtype_variant(name, variant_index, variant, v).map(Ok::new).map_err(erase)
    }
    fn erased_serialize_seq(&mut self, len: Option<usize>) -> Result<Seq, Error> {
        self.take().serialize_seq(len).map(Seq::new).map_err(erase)
    }
    fn erased_serialize_tuple(&mut self, len: usize) -> Result<Tuple, Error> {
        self.take().serialize_tuple(len).map(Tuple::new).map_err(erase)
    }
    fn erased_serialize_tuple_struct(&mut self, name: &'static str, len: usize) -> Result<TupleStruct, Error> {
        self.take().serialize_tuple_struct(name, len).map(TupleStruct::new).map_err(erase)
    }
    fn erased_serialize_tuple_variant(&mut self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<TupleVariant, Error> {
        self.take().serialize_tuple_variant(name, variant_index, variant, len).map(TupleVariant::new).map_err(erase)
    }
    fn erased_serialize_map(&mut self, len: Option<usize>) -> Result<Map, Error> {
        self.take().serialize_map(len).map(Map::new).map_err(erase)
    }
    fn erased_serialize_struct(&mut self, name: &'static str, len: usize) -> Result<Struct, Error> {
        self.take().serialize_struct(name, len).map(Struct::new).map_err(erase)
    }
    fn erased_serialize_struct_variant(&mut self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<StructVariant, Error> {
        self.take().serialize_struct_variant(name, variant_index, variant, len).map(StructVariant::new).map_err(erase)
    }
    fn erased_is_human_readable(&self) -> bool {
        self.as_ref().is_human_readable()
    }
}

// IMPL SERDE FOR ERASED SERDE /////////////////////////////////////////////////

/// Serialize the given type-erased serializable value.
///
/// This can be used to implement `serde::Serialize` for trait objects that have
/// `erased_serde::Serialize` as a supertrait.
///
/// ```
/// # extern crate serde;
/// # extern crate erased_serde;
/// #
/// trait Event: erased_serde::Serialize {
///     /* ... */
/// }
///
/// impl<'a> serde::Serialize for Event + 'a {
///     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
///         where S: serde::Serializer
///     {
///         erased_serde::serialize(self, serializer)
///     }
/// }
/// #
/// # fn main() {}
/// ```
///
/// Since this is reasonably common, the `serialize_trait_object!` macro
/// generates such a Serialize impl.
///
/// ```
/// #[macro_use]
/// extern crate erased_serde;
/// #
/// # trait Event: erased_serde::Serialize {}
///
/// serialize_trait_object!(Event);
/// #
/// # fn main() {}
/// ```
pub fn serialize<T: ?Sized, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where T: Serialize,
          S: serde::Serializer
{
    let mut erased = erase::Serializer { state: Some(serializer) };
    value.erased_serialize(&mut erased).map(Ok::take).map_err(unerase)
}

serialize_trait_object!(Serialize);

macro_rules! impl_serializer_for_trait_object {
    ($ty:ty) => {
        impl<'a> serde::Serializer for $ty {
            type Ok = Ok;
            type Error = Error;
            type SerializeSeq = Seq<'a>;
            type SerializeTuple = Tuple<'a>;
            type SerializeTupleStruct = TupleStruct<'a>;
            type SerializeTupleVariant = TupleVariant<'a>;
            type SerializeMap = Map<'a>;
            type SerializeStruct = Struct<'a>;
            type SerializeStructVariant = StructVariant<'a>;
            fn serialize_bool(self, v: bool) -> Result<Ok, Error> {
                self.erased_serialize_bool(v)
            }
            fn serialize_i8(self, v: i8) -> Result<Ok, Error> {
                self.erased_serialize_i8(v)
            }
            fn serialize_i16(self, v: i16) -> Result<Ok, Error> {
                self.erased_serialize_i16(v)
            }
            fn serialize_i32(self, v: i32) -> Result<Ok, Error> {
                self.erased_serialize_i32(v)
            }
            fn serialize_i64(self, v: i64) -> Result<Ok, Error> {
                self.erased_serialize_i64(v)
            }
            fn serialize_u8(self, v: u8) -> Result<Ok, Error> {
                self.erased_serialize_u8(v)
            }
            fn serialize_u16(self, v: u16) -> Result<Ok, Error> {
                self.erased_serialize_u16(v)
            }
            fn serialize_u32(self, v: u32) -> Result<Ok, Error> {
                self.erased_serialize_u32(v)
            }
            fn serialize_u64(self, v: u64) -> Result<Ok, Error> {
                self.erased_serialize_u64(v)
            }
            serde_if_integer128! {
                fn serialize_i128(self, v: i128) -> Result<Ok, Error> {
                    self.erased_serialize_i128(v)
                }
                fn serialize_u128(self, v: u128) -> Result<Ok, Error> {
                    self.erased_serialize_u128(v)
                }
            }
            fn serialize_f32(self, v: f32) -> Result<Ok, Error> {
                self.erased_serialize_f32(v)
            }
            fn serialize_f64(self, v: f64) -> Result<Ok, Error> {
                self.erased_serialize_f64(v)
            }
            fn serialize_char(self, v: char) -> Result<Ok, Error> {
                self.erased_serialize_char(v)
            }
            fn serialize_str(self, v: &str) -> Result<Ok, Error> {
                self.erased_serialize_str(v)
            }
            fn serialize_bytes(self, v: &[u8]) -> Result<Ok, Error> {
                self.erased_serialize_bytes(v)
            }
            fn serialize_none(self) -> Result<Ok, Error> {
                self.erased_serialize_none()
            }
            fn serialize_some<T: ?Sized + serde::Serialize>(self, v: &T) -> Result<Ok, Error> {
                self.erased_serialize_some(&v)
            }
            fn serialize_unit(self) -> Result<Ok, Error> {
                self.erased_serialize_unit()
            }
            fn serialize_unit_struct(self, name: &'static str) -> Result<Ok, Error> {
                self.erased_serialize_unit_struct(name)
            }
            fn serialize_unit_variant(self, name: &'static str, variant_index: u32, variant: &'static str) -> Result<Ok, Error> {
                self.erased_serialize_unit_variant(name, variant_index, variant)
            }
            fn serialize_newtype_struct<T: ?Sized + serde::Serialize>(self, name: &'static str, v: &T) -> Result<Ok, Error> {
                self.erased_serialize_newtype_struct(name, &v)
            }
            fn serialize_newtype_variant<T: ?Sized + serde::Serialize>(self, name: &'static str, variant_index: u32, variant: &'static str, v: &T) -> Result<Ok, Error> {
                self.erased_serialize_newtype_variant(name, variant_index, variant, &v)
            }
            fn serialize_seq(self, len: Option<usize>) -> Result<Seq<'a>, Error> {
                self.erased_serialize_seq(len)
            }
            fn serialize_tuple(self, len: usize) -> Result<Tuple<'a>, Error> {
                self.erased_serialize_tuple(len)
            }
            fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<TupleStruct<'a>, Error> {
                self.erased_serialize_tuple_struct(name, len)
            }
            fn serialize_tuple_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<TupleVariant<'a>, Error> {
                self.erased_serialize_tuple_variant(name, variant_index, variant, len)
            }
            fn serialize_map(self, len: Option<usize>) -> Result<Map<'a>, Error> {
                self.erased_serialize_map(len)
            }
            fn serialize_struct(self, name: &'static str, len: usize) -> Result<Struct<'a>, Error> {
                self.erased_serialize_struct(name, len)
            }
            fn serialize_struct_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<StructVariant<'a>, Error> {
                self.erased_serialize_struct_variant(name, variant_index, variant, len)
            }
            fn is_human_readable(&self) -> bool {
                self.erased_is_human_readable()
            }
        }
    };
}

impl_serializer_for_trait_object!(&'a mut dyn Serializer);
impl_serializer_for_trait_object!(&'a mut (dyn Serializer + Send));
impl_serializer_for_trait_object!(&'a mut (dyn Serializer + Sync));
impl_serializer_for_trait_object!(&'a mut (dyn Serializer + Send + Sync));

pub struct Seq<'a> {
    data: Any,
    serialize_element: fn(&mut Any, &dyn Serialize) -> Result<(), Error>,
    end: fn(Any) -> Result<Ok, Error>,
    lifetime: PhantomData<&'a dyn Serializer>,
}

impl<'a> Seq<'a> {
    fn new<T: serde::ser::SerializeSeq>(data: T) -> Self {
        Seq {
            data: Any::new(data),
            serialize_element: |data, v| {
                data.view::<T>().serialize_element(v).map_err(erase)
            },
            end: |data| {
                data.take::<T>().end().map(Ok::new).map_err(erase)
            },
            lifetime: PhantomData,
        }
    }
}

impl<'a> SerializeSeq for Seq<'a> {
    type Ok = Ok;
    type Error = Error;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error> where T: serde::Serialize {
        (self.serialize_element)(&mut self.data, &value)
    }
    fn end(self) -> Result<Ok, Error> {
        (self.end)(self.data)
    }
}

pub struct Tuple<'a> {
    data: Any,
    serialize_element: fn(&mut Any, &dyn Serialize) -> Result<(), Error>,
    end: fn(Any) -> Result<Ok, Error>,
    lifetime: PhantomData<&'a dyn Serializer>,
}

impl<'a> Tuple<'a> {
    fn new<T: serde::ser::SerializeTuple>(data: T) -> Self {
        Tuple {
            data: Any::new(data),
            serialize_element: |data, v| {
                data.view::<T>().serialize_element(v).map_err(erase)
            },
            end: |data| {
                data.take::<T>().end().map(Ok::new).map_err(erase)
            },
            lifetime: PhantomData,
        }
    }
}

impl<'a> SerializeTuple for Tuple<'a> {
    type Ok = Ok;
    type Error = Error;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error> where T: serde::Serialize {
        (self.serialize_element)(&mut self.data, &value)
    }
    fn end(self) -> Result<Ok, Error> {
        (self.end)(self.data)
    }
}

pub struct TupleStruct<'a> {
    data: Any,
    serialize_field: fn(&mut Any, &dyn Serialize) -> Result<(), Error>,
    end: fn(Any) -> Result<Ok, Error>,
    lifetime: PhantomData<&'a dyn Serializer>,
}

impl<'a> TupleStruct<'a> {
    fn new<T: serde::ser::SerializeTupleStruct>(data: T) -> Self {
        TupleStruct {
            data: Any::new(data),
            serialize_field: |data, v| {
                data.view::<T>().serialize_field(v).map_err(erase)
            },
            end: |data| {
                data.take::<T>().end().map(Ok::new).map_err(erase)
            },
            lifetime: PhantomData,
        }
    }
}

impl<'a> SerializeTupleStruct for TupleStruct<'a> {
    type Ok = Ok;
    type Error = Error;
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Error> where T: serde::Serialize {
        (self.serialize_field)(&mut self.data, &value)
    }
    fn end(self) -> Result<Ok, Error> {
        (self.end)(self.data)
    }
}

pub struct TupleVariant<'a> {
    data: Any,
    serialize_field: fn(&mut Any, &dyn Serialize) -> Result<(), Error>,
    end: fn(Any) -> Result<Ok, Error>,
    lifetime: PhantomData<&'a dyn Serializer>,
}

impl<'a> TupleVariant<'a> {
    fn new<T: serde::ser::SerializeTupleVariant>(data: T) -> Self {
        TupleVariant {
            data: Any::new(data),
            serialize_field: |data, v| {
                data.view::<T>().serialize_field(v).map_err(erase)
            },
            end: |data| {
                data.take::<T>().end().map(Ok::new).map_err(erase)
            },
            lifetime: PhantomData,
        }
    }
}

impl<'a> SerializeTupleVariant for TupleVariant<'a> {
    type Ok = Ok;
    type Error = Error;
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Error> where T: serde::Serialize {
        (self.serialize_field)(&mut self.data, &value)
    }
    fn end(self) -> Result<Ok, Error> {
        (self.end)(self.data)
    }
}

pub struct Map<'a> {
    data: Any,
    serialize_key: fn(&mut Any, &dyn Serialize) -> Result<(), Error>,
    serialize_value: fn(&mut Any, &dyn Serialize) -> Result<(), Error>,
    serialize_entry: fn(&mut Any, &dyn Serialize, &dyn Serialize) -> Result<(), Error>,
    end: fn(Any) -> Result<Ok, Error>,
    lifetime: PhantomData<&'a dyn Serializer>,
}

impl<'a> Map<'a> {
    fn new<T: serde::ser::SerializeMap>(data: T) -> Self {
        Map {
            data: Any::new(data),
            serialize_key: |data, v| {
                data.view::<T>().serialize_key(v).map_err(erase)
            },
            serialize_value: |data, v| {
                data.view::<T>().serialize_value(v).map_err(erase)
            },
            serialize_entry: |data, k, v| {
                data.view::<T>().serialize_entry(k, v).map_err(erase)
            },
            end: |data| {
                data.take::<T>().end().map(Ok::new).map_err(erase)
            },
            lifetime: PhantomData,
        }
    }
}

impl<'a> SerializeMap for Map<'a> {
    type Ok = Ok;
    type Error = Error;
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Error> where T: serde::Serialize {
        (self.serialize_key)(&mut self.data, &key)
    }
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Error> where T: serde::Serialize {
        (self.serialize_value)(&mut self.data, &value)
    }
    fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, key: &K, value: &V) -> Result<(), Error> where K: serde::Serialize, V: serde::Serialize {
        (self.serialize_entry)(&mut self.data, &key, &value)
    }
    fn end(self) -> Result<Ok, Error> {
        (self.end)(self.data)
    }
}

pub struct Struct<'a> {
    data: Any,
    serialize_field: fn(&mut Any, &'static str, &dyn Serialize) -> Result<(), Error>,
    end: fn(Any) -> Result<Ok, Error>,
    lifetime: PhantomData<&'a dyn Serializer>,
}

impl<'a> Struct<'a> {
    fn new<T: serde::ser::SerializeStruct>(data: T) -> Self {
        Struct {
            data: Any::new(data),
            serialize_field: |data, k, v| {
                data.view::<T>().serialize_field(k, v).map_err(erase)
            },
            end: |data| {
                data.take::<T>().end().map(Ok::new).map_err(erase)
            },
            lifetime: PhantomData,
        }
    }
}

impl<'a> SerializeStruct for Struct<'a> {
    type Ok = Ok;
    type Error = Error;
    fn serialize_field<T: ?Sized>(&mut self, name: &'static str, field: &T) -> Result<(), Error> where T: serde::Serialize {
        (self.serialize_field)(&mut self.data, name, &field)
    }
    fn end(self) -> Result<Ok, Error> {
        (self.end)(self.data)
    }
}

pub struct StructVariant<'a> {
    data: Any,
    serialize_field: fn(&mut Any, &'static str, &dyn Serialize) -> Result<(), Error>,
    end: fn(Any) -> Result<Ok, Error>,
    lifetime: PhantomData<&'a dyn Serializer>,
}

impl<'a> StructVariant<'a> {
    fn new<T: serde::ser::SerializeStructVariant>(data: T) -> Self {
        StructVariant {
            data: Any::new(data),
            serialize_field: |data, k, v| {
                data.view::<T>().serialize_field(k, v).map_err(erase)
            },
            end: |data| {
                data.take::<T>().end().map(Ok::new).map_err(erase)
            },
            lifetime: PhantomData,
        }
    }
}

impl<'a> SerializeStructVariant for StructVariant<'a> {
    type Ok = Ok;
    type Error = Error;
    fn serialize_field<T: ?Sized>(&mut self, name: &'static str, field: &T) -> Result<(), Error> where T: serde::Serialize {
        (self.serialize_field)(&mut self.data, name, &field)
    }
    fn end(self) -> Result<Ok, Error> {
        (self.end)(self.data)
    }
}

// IMPL ERASED SERDE FOR ERASED SERDE //////////////////////////////////////////

macro_rules! deref_erased_serializer {
    ($($imp:tt)+) => {
        impl $($imp)+ {
            fn erased_serialize_bool(&mut self, v: bool) -> Result<Ok, Error> {
                (**self).erased_serialize_bool(v)
            }
            fn erased_serialize_i8(&mut self, v: i8) -> Result<Ok, Error> {
                (**self).erased_serialize_i8(v)
            }
            fn erased_serialize_i16(&mut self, v: i16) -> Result<Ok, Error> {
                (**self).erased_serialize_i16(v)
            }
            fn erased_serialize_i32(&mut self, v: i32) -> Result<Ok, Error> {
                (**self).erased_serialize_i32(v)
            }
            fn erased_serialize_i64(&mut self, v: i64) -> Result<Ok, Error> {
                (**self).erased_serialize_i64(v)
            }
            fn erased_serialize_u8(&mut self, v: u8) -> Result<Ok, Error> {
                (**self).erased_serialize_u8(v)
            }
            fn erased_serialize_u16(&mut self, v: u16) -> Result<Ok, Error> {
                (**self).erased_serialize_u16(v)
            }
            fn erased_serialize_u32(&mut self, v: u32) -> Result<Ok, Error> {
                (**self).erased_serialize_u32(v)
            }
            fn erased_serialize_u64(&mut self, v: u64) -> Result<Ok, Error> {
                (**self).erased_serialize_u64(v)
            }
            serde_if_integer128! {
                fn erased_serialize_i128(&mut self, v: i128) -> Result<Ok, Error> {
                    (**self).erased_serialize_i128(v)
                }
                fn erased_serialize_u128(&mut self, v: u128) -> Result<Ok, Error> {
                    (**self).erased_serialize_u128(v)
                }
            }
            fn erased_serialize_f32(&mut self, v: f32) -> Result<Ok, Error> {
                (**self).erased_serialize_f32(v)
            }
            fn erased_serialize_f64(&mut self, v: f64) -> Result<Ok, Error> {
                (**self).erased_serialize_f64(v)
            }
            fn erased_serialize_char(&mut self, v: char) -> Result<Ok, Error> {
                (**self).erased_serialize_char(v)
            }
            fn erased_serialize_str(&mut self, v: &str) -> Result<Ok, Error> {
                (**self).erased_serialize_str(v)
            }
            fn erased_serialize_bytes(&mut self, v: &[u8]) -> Result<Ok, Error> {
                (**self).erased_serialize_bytes(v)
            }
            fn erased_serialize_none(&mut self) -> Result<Ok, Error> {
                (**self).erased_serialize_none()
            }
            fn erased_serialize_some(&mut self, v: &dyn Serialize) -> Result<Ok, Error> {
                (**self).erased_serialize_some(v)
            }
            fn erased_serialize_unit(&mut self) -> Result<Ok, Error> {
                (**self).erased_serialize_unit()
            }
            fn erased_serialize_unit_struct(&mut self, name: &'static str) -> Result<Ok, Error> {
                (**self).erased_serialize_unit_struct(name)
            }
            fn erased_serialize_unit_variant(&mut self, name: &'static str, variant_index: u32, variant: &'static str) -> Result<Ok, Error> {
                (**self).erased_serialize_unit_variant(name, variant_index, variant)
            }
            fn erased_serialize_newtype_struct(&mut self, name: &'static str, v: &dyn Serialize) -> Result<Ok, Error> {
                (**self).erased_serialize_newtype_struct(name, v)
            }
            fn erased_serialize_newtype_variant(&mut self, name: &'static str, variant_index: u32, variant: &'static str, v: &dyn Serialize) -> Result<Ok, Error> {
                (**self).erased_serialize_newtype_variant(name, variant_index, variant, v)
            }
            fn erased_serialize_seq(&mut self, len: Option<usize>) -> Result<Seq, Error> {
                (**self).erased_serialize_seq(len)
            }
            fn erased_serialize_tuple(&mut self, len: usize) -> Result<Tuple, Error> {
                (**self).erased_serialize_tuple(len)
            }
            fn erased_serialize_tuple_struct(&mut self, name: &'static str, len: usize) -> Result<TupleStruct, Error> {
                (**self).erased_serialize_tuple_struct(name, len)
            }
            fn erased_serialize_tuple_variant(&mut self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<TupleVariant, Error> {
                (**self).erased_serialize_tuple_variant(name, variant_index, variant, len)
            }
            fn erased_serialize_map(&mut self, len: Option<usize>) -> Result<Map, Error> {
                (**self).erased_serialize_map(len)
            }
            fn erased_serialize_struct(&mut self, name: &'static str, len: usize) -> Result<Struct, Error> {
                (**self).erased_serialize_struct(name, len)
            }
            fn erased_serialize_struct_variant(&mut self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<StructVariant, Error> {
                (**self).erased_serialize_struct_variant(name, variant_index, variant, len)
            }
            fn erased_is_human_readable(&self) -> bool {
                (**self).erased_is_human_readable()
            }
        }
    };
}

deref_erased_serializer!(<'a> Serializer for Box<dyn Serializer + 'a>);
deref_erased_serializer!(<'a> Serializer for Box<dyn Serializer + Send + 'a>);
deref_erased_serializer!(<'a> Serializer for Box<dyn Serializer + Sync + 'a>);
deref_erased_serializer!(<'a> Serializer for Box<dyn Serializer + Send + Sync + 'a>);
deref_erased_serializer!(<'a, T: ?Sized + Serializer> Serializer for &'a mut T);

// ERROR ///////////////////////////////////////////////////////////////////////

fn erase<E>(e: E) -> Error
    where E: Display
{
    serde::ser::Error::custom(e)
}

#[allow(deprecated)]
fn unerase<E>(e: Error) -> E
    where E: serde::ser::Error
{
    use std::error::Error;
    E::custom(e.description())
}

// TEST ////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    fn test_json<T>(t: T)
        where T: serde::Serialize
    {
        let expected = serde_json::to_vec(&t).unwrap();

        // test borrowed trait object
        {
            let obj: &dyn Serialize = &t;

            let mut buf = Vec::new();

            {
                let mut ser = serde_json::Serializer::new(&mut buf);
                let ser: &mut Serializer = &mut Serializer::erase(&mut ser);

                obj.erased_serialize(ser).unwrap();
            }

            assert_eq!(buf, expected);
        }

        // test boxed trait object
        {
            let obj: Box<Serialize> = Box::new(t);

            let mut buf = Vec::new();

            {
                let mut ser = serde_json::Serializer::new(&mut buf);
                let mut ser: Box<Serializer> = Box::new(Serializer::erase(&mut ser));

                obj.erased_serialize(&mut ser).unwrap();
            }

            assert_eq!(buf, expected);
        }
    }

    #[test]
    fn test_vec() {
        test_json(vec!["a", "b"]);
    }

    #[test]
    fn test_struct() {
        #[derive(Serialize)]
        struct S {
            f: usize,
        }

        test_json(S { f: 256 });
    }

    #[test]
    fn test_enum() {
        #[derive(Serialize)]
        enum E {
            Unit,
            Newtype(bool),
            Tuple(bool, bool),
            Struct { t: bool, f: bool },
        }

        test_json(E::Unit);
        test_json(E::Newtype(true));
        test_json(E::Tuple(true, false));
        test_json(E::Struct { t: true, f: false });
    }

    #[test]
    fn assert_serialize() {
        fn assert<T: serde::Serialize>() {}

        assert::<&dyn Serialize>();
        assert::<&(Serialize + Send)>();
        assert::<&(Serialize + Sync)>();
        assert::<&(Serialize + Send + Sync)>();
        assert::<&(Serialize + Sync + Send)>();
        assert::<Vec<&dyn Serialize>>();
        assert::<Vec<&(Serialize + Send)>>();

        assert::<Box<Serialize>>();
        assert::<Box<Serialize + Send>>();
        assert::<Box<Serialize + Sync>>();
        assert::<Box<Serialize + Send + Sync>>();
        assert::<Box<Serialize + Sync + Send>>();
        assert::<Vec<Box<Serialize>>>();
        assert::<Vec<Box<Serialize + Send>>>();
    }

    #[test]
    fn assert_serializer() {
        fn assert<T: serde::Serializer>() {}

        assert::<&mut Serializer>();
        assert::<&mut (Serializer + Send)>();
        assert::<&mut (Serializer + Sync)>();
        assert::<&mut (Serializer + Send + Sync)>();
        assert::<&mut (Serializer + Sync + Send)>();
    }
}
