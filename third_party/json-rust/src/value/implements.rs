// This is a private module that contains `PartialEq` and `From` trait
// implementations for `JsonValue`.

use std::collections::{ BTreeMap, HashMap };
use std::mem;
use std::string::String;
use std::vec::Vec;

use short::{ self, Short };
use number::Number;
use object::Object;

use { JsonValue, Null };

macro_rules! implement_eq {
    ($to:ident, $from:ty) => {
        impl PartialEq<$from> for JsonValue {
            fn eq(&self, other: &$from) -> bool {
                match *self {
                    JsonValue::$to(ref value) => value == other,
                    _                         => false
                }
            }
        }

        impl<'a> PartialEq<$from> for &'a JsonValue {
            fn eq(&self, other: &$from) -> bool {
                match **self {
                    JsonValue::$to(ref value) => value == other,
                    _                         => false
                }
            }
        }

        impl PartialEq<JsonValue> for $from {
            fn eq(&self, other: &JsonValue) -> bool {
                match *other {
                    JsonValue::$to(ref value) => value == self,
                    _ => false
                }
            }
        }
    }
}

macro_rules! implement {
    ($to:ident, $from:ty as num) => {
        impl From<$from> for JsonValue {
            fn from(val: $from) -> JsonValue {
                JsonValue::$to(val.into())
            }
        }

        implement_eq!($to, $from);
    };
    ($to:ident, $from:ty) => {
        impl From<$from> for JsonValue {
            fn from(val: $from) -> JsonValue {
                JsonValue::$to(val)
            }
        }

        implement_eq!($to, $from);
    }
}

impl<'a> From<&'a str> for JsonValue {
    fn from(val: &'a str) -> JsonValue {
        if val.len() <= short::MAX_LEN {
            JsonValue::Short(unsafe { Short::from_slice(val) })
        } else {
            JsonValue::String(val.into())
        }
    }
}

impl<T: Into<JsonValue>> From<Option<T>> for JsonValue {
    fn from(val: Option<T>) -> JsonValue {
        match val {
            Some(val) => val.into(),
            None      => JsonValue::Null,
        }
    }
}

impl<T: Into<JsonValue>> From<Vec<T>> for JsonValue {
    fn from(val: Vec<T>) -> JsonValue {
        let mut array = Vec::with_capacity(val.len());

        for val in val {
            array.push(val.into());
        }

        JsonValue::Array(array)
    }
}

impl From<HashMap<String, JsonValue>> for JsonValue {
    fn from(mut val: HashMap<String, JsonValue>) -> JsonValue {
        let mut object = Object::with_capacity(val.len());

        for (key, value) in val.drain() {
            object.insert(&key, value);
        }

        JsonValue::Object(object)
    }
}

impl From<BTreeMap<String, JsonValue>> for JsonValue {
    fn from(mut val: BTreeMap<String, JsonValue>) -> JsonValue {
        let mut object = Object::with_capacity(val.len());

        for (key, value) in val.iter_mut() {
            // Since BTreeMap has no `drain` available, we can use
            // the mutable iterator and replace all values by nulls,
            // taking ownership and transferring it to the new `Object`.
            let value = mem::replace(value, Null);
            object.insert(key, value);
        }

        JsonValue::Object(object)
    }
}

impl<'a> PartialEq<&'a str> for JsonValue {
    fn eq(&self, other: &&str) -> bool {
        match *self {
            JsonValue::Short(ref value)  => value == *other,
            JsonValue::String(ref value) => value == *other,
            _ => false
        }
    }
}

impl<'a> PartialEq<JsonValue> for &'a str {
    fn eq(&self, other: &JsonValue) -> bool {
        match *other {
            JsonValue::Short(ref value)  => value == *self,
            JsonValue::String(ref value) => value == *self,
            _ => false
        }
    }
}

impl PartialEq<str> for JsonValue {
    fn eq(&self, other: &str) -> bool {
        match *self {
            JsonValue::Short(ref value)  => value == other,
            JsonValue::String(ref value) => value == other,
            _ => false
        }
    }
}

impl<'a> PartialEq<JsonValue> for str {
    fn eq(&self, other: &JsonValue) -> bool {
        match *other {
            JsonValue::Short(ref value)  => value == self,
            JsonValue::String(ref value) => value == self,
            _ => false
        }
    }
}

implement!(String, String);
implement!(Number, isize as num);
implement!(Number, usize as num);
implement!(Number, i8 as num);
implement!(Number, i16 as num);
implement!(Number, i32 as num);
implement!(Number, i64 as num);
implement!(Number, u8 as num);
implement!(Number, u16 as num);
implement!(Number, u32 as num);
implement!(Number, u64 as num);
implement!(Number, f32 as num);
implement!(Number, f64 as num);
implement!(Number, Number);
implement!(Object, Object);
implement!(Boolean, bool);
