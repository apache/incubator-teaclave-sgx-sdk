//! CBOR values and keys.

use std::prelude::v1::*;
use std::collections::BTreeMap;
use std::fmt;

use serde::de;
use serde::ser;

/// An enum over all possible CBOR types.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// Represents an unsigned integer.
    U64(u64),
    /// Represents a signed integer.
    I64(i64),
    /// Represents a byte string.
    Bytes(Vec<u8>),
    /// Represents an UTF-8 string.
    String(String),
    /// Represents a list.
    Array(Vec<Value>),
    /// Represents a map.
    Object(BTreeMap<ObjectKey, Value>),
    /// Represents a floating point value.
    F64(f64),
    /// Represents a boolean value.
    Bool(bool),
    /// Represents the absence of a value or the value undefined.
    Null,
}

impl Value {
    /// Returns true if the value is an object.
    pub fn is_object(&self) -> bool {
        self.as_object().is_some()
    }

    /// If the value is an object, returns the associated BTreeMap. Returns None otherwise.
    pub fn as_object(&self) -> Option<&BTreeMap<ObjectKey, Value>> {
        if let Value::Object(ref v) = *self {
            Some(v)
        } else {
            None
        }
    }


    /// If the value is an object, returns the associated mutable BTreeMap. Returns None otherwise.
    pub fn as_object_mut(&mut self) -> Option<&mut BTreeMap<ObjectKey, Value>> {
        if let Value::Object(ref mut v) = *self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns true if the value is an array.
    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    /// If the value is an array, returns the associated Vec. Returns None otherwise.
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        if let Value::Array(ref v) = *self {
            Some(v)
        } else {
            None
        }
    }

    /// If the value is an array, returns the associated mutable Vec. Returns None otherwise.
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        if let Value::Array(ref mut v) = *self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns true if the value is a byte string.
    pub fn is_bytes(&self) -> bool {
        self.as_bytes().is_some()
    }

    /// Returns the associated byte string or `None` if the value has a different type.
    pub fn as_bytes(&self) -> Option<&Vec<u8>> {
        if let Value::Bytes(ref v) = *self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns the associated mutable byte string or `None` if the value has a different type.
    pub fn as_bytes_mut(&mut self) -> Option<&mut Vec<u8>> {
        if let Value::Bytes(ref mut v) = *self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns true if the value is a string.
    pub fn is_string(&self) -> bool {
        self.as_string().is_some()
    }

    /// Returns the associated string or `None` if the value has a different type.
    pub fn as_string(&self) -> Option<&String> {
        if let Value::String(ref v) = *self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns the associated mutable string or `None` if the value has a different type.
    pub fn as_string_mut(&mut self) -> Option<&mut String> {
        if let Value::String(ref mut v) = *self {
            Some(v)
        } else {
            None
        }
    }

    /// Retrns true if the value is a number.
    pub fn is_number(&self) -> bool {
        match *self {
            Value::U64(_) | Value::I64(_) | Value::F64(_) => true,
            _ => false,
        }
    }

    /// Returns true if the `Value` is a i64. Returns false otherwise.
    pub fn is_i64(&self) -> bool {
        match *self {
            Value::I64(_) => true,
            _ => false,
        }
    }

    /// Returns true if the `Value` is a u64. Returns false otherwise.
    pub fn is_u64(&self) -> bool {
        match *self {
            Value::U64(_) => true,
            _ => false,
        }
    }

    /// Returns true if the `Value` is a f64. Returns false otherwise.
    pub fn is_f64(&self) -> bool {
        match *self {
            Value::F64(_) => true,
            _ => false,
        }
    }

    /// If the `Value` is a number, return or cast it to a i64. Returns None otherwise.
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Value::I64(n) => Some(n),
            Value::U64(n) => Some(n as i64),
            _ => None,
        }
    }

    /// If the `Value` is a number, return or cast it to a u64. Returns None otherwise.
    pub fn as_u64(&self) -> Option<u64> {
        match *self {
            Value::I64(n) => Some(n as u64),
            Value::U64(n) => Some(n),
            _ => None,
        }
    }

    /// If the `Value` is a number, return or cast it to a f64. Returns None otherwise.
    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Value::I64(n) => Some(n as f64),
            Value::U64(n) => Some(n as f64),
            Value::F64(n) => Some(n),
            _ => None,
        }
    }

    /// Returns true if the value is a boolean.
    pub fn is_boolean(&self) -> bool {
        self.as_boolean().is_some()
    }

    /// If the value is a Boolean, returns the associated bool. Returns None otherwise.
    pub fn as_boolean(&self) -> Option<bool> {
        if let Value::Bool(v) = *self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns true if the value is a Null. Returns false otherwise.
    pub fn is_null(&self) -> bool {
        self.as_null().is_some()
    }

    /// If the value is a Null, returns (). Returns None otherwise.
    pub fn as_null(&self) -> Option<()> {
        if let Value::Null = *self {
            Some(())
        } else {
            None
        }
    }
}

impl<'de> de::Deserialize<'de> for Value {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> de::Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str("any valid CBOR value")
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Value, E>
            where
                E: de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Value, E>
            where
                E: de::Error,
            {
                Ok(Value::String(value))
            }
            #[inline]
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_byte_buf(v.to_owned())
            }

            #[inline]
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Bytes(v))
            }

            #[inline]
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::U64(v))
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::I64(v))
            }

            #[inline]
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Bool(v))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_unit()
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Null)
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }

                Ok(Value::Array(vec))
            }

            #[inline]
            fn visit_map<V>(self, mut visitor: V) -> Result<Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut values = BTreeMap::new();

                while let Some((key, value)) = visitor.next_entry()? {
                    values.insert(key, value);
                }

                Ok(Value::Object(values))
            }

            #[inline]
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::F64(v))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl ser::Serialize for Value {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match *self {
            Value::U64(v) => serializer.serialize_u64(v),
            Value::I64(v) => serializer.serialize_i64(v),
            Value::Bytes(ref v) => serializer.serialize_bytes(&v),
            Value::String(ref v) => serializer.serialize_str(&v),
            Value::Array(ref v) => v.serialize(serializer),
            Value::Object(ref v) => v.serialize(serializer),
            Value::F64(v) => serializer.serialize_f64(v),
            Value::Bool(v) => serializer.serialize_bool(v),
            Value::Null => serializer.serialize_unit(),
        }
    }
}

/// A simplified CBOR value containing only types useful for keys.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObjectKey {
    /// An integer.
    Integer(i64),
    /// A byte string.
    Bytes(Vec<u8>),
    /// An UTF-8 string.
    String(String),
    /// A boolean value.
    Bool(bool),
    /// No value.
    Null,
}

impl ObjectKey {
    /// Returns true if the ObjectKey is a byte string.
    pub fn is_bytes(&self) -> bool {
        self.as_bytes().is_some()
    }

    /// Returns the associated byte string or `None` if the ObjectKey has a different type.
    pub fn as_bytes(&self) -> Option<&Vec<u8>> {
        if let ObjectKey::Bytes(ref v) = *self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns the associated mutable byte string or `None` if the ObjectKey has a different type.
    pub fn as_bytes_mut(&mut self) -> Option<&mut Vec<u8>> {
        if let ObjectKey::Bytes(ref mut v) = *self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns true if the ObjectKey is a string.
    pub fn is_string(&self) -> bool {
        self.as_string().is_some()
    }

    /// Returns the associated string or `None` if the *ObjectKey` has a different type.
    pub fn as_string(&self) -> Option<&String> {
        if let ObjectKey::String(ref v) = *self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns the associated mutable string or `None` if the `ObjectKey` has a different type.
    pub fn as_string_mut(&mut self) -> Option<&mut String> {
        if let ObjectKey::String(ref mut v) = *self {
            Some(v)
        } else {
            None
        }
    }

    /// Retrns true if the `ObjectKey` is a number.
    pub fn is_number(&self) -> bool {
        match *self {
            ObjectKey::Integer(_) => true,
            _ => false,
        }
    }

    /// If the `ObjectKey` is a number, return or cast it to a i64. Returns None otherwise.
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            ObjectKey::Integer(n) => Some(n),
            _ => None,
        }
    }

    /// If the `ObjectKey` is a number, return or cast it to a u64. Returns None otherwise.
    pub fn as_u64(&self) -> Option<u64> {
        match *self {
            ObjectKey::Integer(n) => Some(n as u64),
            _ => None,
        }
    }

    /// Returns true if the ObjectKey is a boolean.
    pub fn is_boolean(&self) -> bool {
        self.as_boolean().is_some()
    }

    /// If the ObjectKey is a Boolean, returns the associated bool. Returns None otherwise.
    pub fn as_boolean(&self) -> Option<bool> {
        if let ObjectKey::Bool(v) = *self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns true if the ObjectKey is a Null. Returns false otherwise.
    pub fn is_null(&self) -> bool {
        self.as_null().is_some()
    }

    /// If the ObjectKey is a Null, returns (). Returns None otherwise.
    pub fn as_null(&self) -> Option<()> {
        if let ObjectKey::Null = *self {
            Some(())
        } else {
            None
        }
    }
}

impl<'de> de::Deserialize<'de> for ObjectKey {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<ObjectKey, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ObjectKeyVisitor;

        impl<'de> de::Visitor<'de> for ObjectKeyVisitor {
            type Value = ObjectKey;

            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str("any valid CBOR key")
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ObjectKey::String(value))
            }
            #[inline]
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_byte_buf(v.to_owned())
            }

            #[inline]
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ObjectKey::Bytes(v))
            }

            #[inline]
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ObjectKey::Integer(v as i64))
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ObjectKey::Integer(v))
            }

            #[inline]
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ObjectKey::Bool(v))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_unit()
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ObjectKey::Null)
            }
        }

        deserializer.deserialize_any(ObjectKeyVisitor)
    }
}

impl ser::Serialize for ObjectKey {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match *self {
            ObjectKey::Integer(v) => serializer.serialize_i64(v),
            ObjectKey::Bytes(ref v) => serializer.serialize_bytes(&v),
            ObjectKey::String(ref v) => serializer.serialize_str(&v),
            ObjectKey::Bool(v) => serializer.serialize_bool(v),
            ObjectKey::Null => serializer.serialize_unit(),
        }
    }
}

impl From<ObjectKey> for Value {
    fn from(key: ObjectKey) -> Value {
        match key {
            ObjectKey::Integer(v) => Value::I64(v),
            ObjectKey::Bytes(v) => Value::Bytes(v),
            ObjectKey::String(v) => Value::String(v),
            ObjectKey::Bool(v) => Value::Bool(v),
            ObjectKey::Null => Value::Null,
        }
    }
}

impl From<Value> for ObjectKey {
    fn from(value: Value) -> ObjectKey {
        match value {
            Value::U64(v) => ObjectKey::Integer(v as i64),
            Value::I64(v) => ObjectKey::Integer(v),
            Value::Bytes(v) => ObjectKey::Bytes(v),
            Value::String(v) => ObjectKey::String(v),
            Value::Bool(v) => ObjectKey::Bool(v),
            Value::Null => ObjectKey::Null,
            _ => panic!("invalid value type for key"),
        }
    }
}

macro_rules! impl_from {
    ($for_enum:ident, $variant:ident, $for_type:ty) => (
        impl From<$for_type> for $for_enum {
            fn from (v: $for_type) -> $for_enum {
                $for_enum::$variant(v)
            }
        }
    )
}

// All except &'a str and Cow<'a, str>
impl_from!(ObjectKey, Integer, i64);
impl_from!(ObjectKey, Bytes, Vec<u8>);
impl_from!(ObjectKey, String, String);
impl_from!(ObjectKey, Bool, bool);

// All except &'a str and Cow<'a, str>
impl_from!(Value, U64, u64);
impl_from!(Value, I64, i64);
impl_from!(Value, Bytes, Vec<u8>);
impl_from!(Value, String, String);
impl_from!(Value, Array, Vec<Value>);
impl_from!(Value, Object, BTreeMap<ObjectKey, Value>);
impl_from!(Value, F64, f64);
impl_from!(Value, Bool, bool);

/// Convert a `serde_cbor::Value` into a type `T`
pub fn from_value<T>(value: Value) -> Result<T, ::error::Error>
where
    T: de::DeserializeOwned,
{
    // TODO implement in a way that doesn't require
    // roundtrip through buffer (i.e. by implementing
    // `serde::de::Deserializer` for `Value` and then doing
    // `T::deserialize(value)`).
    let buf = ::to_vec(&value)?;
    ::from_slice(buf.as_slice())
}
