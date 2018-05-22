use { Result, Error };

use std::ops::{ Index, IndexMut, Deref };
use std::{ fmt, mem, usize, u8, u16, u32, u64, isize, i8, i16, i32, i64, f32 };
use std::string::String;
use std::vec::Vec;

use std::io::{ self, Write };

use short::Short;
use number::Number;
use object::Object;
use iterators::{ Members, MembersMut, Entries, EntriesMut };
use codegen::{ Generator, PrettyGenerator, DumpGenerator, WriterGenerator, PrettyWriterGenerator };

mod implements;

// These are convenience macros for converting `f64` to the `$unsigned` type.
// The macros check that the numbers are representable the target type.
macro_rules! number_to_unsigned {
    ($unsigned:ident, $value:expr, $high:ty) => {
        if $value > $unsigned::MAX as $high {
            None
        } else {
            Some($value as $unsigned)
        }
    }
}

macro_rules! number_to_signed {
    ($signed:ident, $value:expr, $high:ty) => {
        if $value < $signed::MIN as $high || $value > $signed::MAX as $high {
            None
        } else {
            Some($value as $signed)
        }
    }
}

#[derive(Debug, Clone)]
pub enum JsonValue {
    Null,
    Short(Short),
    String(String),
    Number(Number),
    Boolean(bool),
    Object(Object),
    Array(Vec<JsonValue>),
}

impl PartialEq for JsonValue {
    fn eq(&self, other: &Self) -> bool {
        use self::JsonValue::*;
        match (self, other) {
            (&Null, &Null) => true,
            (&Short(ref a), &Short(ref b)) => a == b,
            (&String(ref a), &String(ref b)) => a == b,
            (&Short(ref a), &String(ref b))
            | (&String(ref b), &Short(ref a)) => a.as_str() == b.as_str(),
            (&Number(ref a), &Number(ref b)) => a == b,
            (&Boolean(ref a), &Boolean(ref b)) => a == b,
            (&Object(ref a), &Object(ref b)) => a == b,
            (&Array(ref a), &Array(ref b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for JsonValue {}

/// Implements formatting
///
/// ```
/// # use json;
/// let data = json::parse(r#"{"url":"https://github.com/"}"#).unwrap();
/// println!("{}", data);
/// println!("{:#}", data);
/// ```
impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            f.write_str(&self.pretty(4))
        } else {
            match *self {
                JsonValue::Short(ref value)   => value.fmt(f),
                JsonValue::String(ref value)  => value.fmt(f),
                JsonValue::Number(ref value)  => value.fmt(f),
                JsonValue::Boolean(ref value) => value.fmt(f),
                JsonValue::Null               => f.write_str("null"),
                _                             => f.write_str(&self.dump())
            }
        }
    }
}


static NULL: JsonValue = JsonValue::Null;

impl JsonValue {
    /// Create an empty `JsonValue::Object` instance.
    /// When creating an object with data, consider using the `object!` macro.
    pub fn new_object() -> JsonValue {
        JsonValue::Object(Object::new())
    }

    /// Create an empty `JsonValue::Array` instance.
    /// When creating array with data, consider using the `array!` macro.
    pub fn new_array() -> JsonValue {
        JsonValue::Array(Vec::new())
    }

    /// Prints out the value as JSON string.
    pub fn dump(&self) -> String {
        let mut gen = DumpGenerator::new();
        gen.write_json(self).expect("Can't fail");
        gen.consume()
    }

    /// Pretty prints out the value as JSON string. Takes an argument that's
    /// number of spaces to indent new blocks with.
    pub fn pretty(&self, spaces: u16) -> String {
        let mut gen = PrettyGenerator::new(spaces);
        gen.write_json(self).expect("Can't fail");
        gen.consume()
    }

    /// Writes the JSON as byte stream into an implementor of `std::io::Write`.
    ///
    /// This method is deprecated as it will panic on io errors, use `write` instead.
    #[deprecated(since="0.10.2", note="use `JsonValue::write` instead")]
    pub fn to_writer<W: Write>(&self, writer: &mut W) {
        let mut gen = WriterGenerator::new(writer);
        gen.write_json(self).expect("Deprecated");
    }

    /// Writes the JSON as byte stream into an implementor of `std::io::Write`.
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let mut gen = WriterGenerator::new(writer);
        gen.write_json(self)
    }

    /// Writes the JSON as byte stream into an implementor of `std::io::Write`.
    pub fn write_pretty<W: Write>(&self, writer: &mut W, spaces: u16) -> io::Result<()> {
        let mut gen = PrettyWriterGenerator::new(writer, spaces);
        gen.write_json(self)
    }

    pub fn is_string(&self) -> bool {
        match *self {
            JsonValue::Short(_)  => true,
            JsonValue::String(_) => true,
            _                    => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match *self {
            JsonValue::Number(_) => true,
            _                    => false,
        }
    }

    pub fn is_boolean(&self) -> bool {
        match *self {
            JsonValue::Boolean(_) => true,
            _                     => false
        }
    }

    pub fn is_null(&self) -> bool {
        match *self {
            JsonValue::Null => true,
            _               => false,
        }
    }

    pub fn is_object(&self) -> bool {
        match *self {
            JsonValue::Object(_) => true,
            _                    => false,
        }
    }

    pub fn is_array(&self) -> bool {
        match *self {
            JsonValue::Array(_) => true,
            _                   => false,
        }
    }

    /// Checks whether the value is empty. Returns true for:
    ///
    /// - empty string (`""`)
    /// - number `0`
    /// - boolean `false`
    /// - null
    /// - empty array (`array![]`)
    /// - empty object (`object!{}`)
    pub fn is_empty(&self) -> bool {
        match *self {
            JsonValue::Null               => true,
            JsonValue::Short(ref value)   => value.is_empty(),
            JsonValue::String(ref value)  => value.is_empty(),
            JsonValue::Number(ref value)  => value.is_empty(),
            JsonValue::Boolean(ref value) => !value,
            JsonValue::Array(ref value)   => value.is_empty(),
            JsonValue::Object(ref value)  => value.is_empty(),
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match *self {
            JsonValue::Short(ref value)  => Some(value),
            JsonValue::String(ref value) => Some(value),
            _                            => None
        }
    }

    pub fn as_number(&self) -> Option<Number> {
        match *self {
            JsonValue::Number(value) => Some(value),
            _                        => None
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        self.as_number().map(|value| value.into())
    }

    pub fn as_f32(&self) -> Option<f32> {
        self.as_number().map(|value| value.into())
    }

    pub fn as_u64(&self) -> Option<u64> {
        self.as_number().and_then(|value| {
            if value.is_sign_positive() {
                Some(value.into())
            } else {
                None
            }
        })
    }

    pub fn as_u32(&self) -> Option<u32> {
        self.as_u64().and_then(|value| number_to_unsigned!(u32, value, u64))
    }

    pub fn as_u16(&self) -> Option<u16> {
        self.as_u64().and_then(|value| number_to_unsigned!(u16, value, u64))
    }

    pub fn as_u8(&self) -> Option<u8> {
        self.as_u64().and_then(|value| number_to_unsigned!(u8, value, u64))
    }

    pub fn as_usize(&self) -> Option<usize> {
        self.as_u64().and_then(|value| number_to_unsigned!(usize, value, u64))
    }

    pub fn as_i64(&self) -> Option<i64> {
        self.as_number().map(|value| value.into())
    }

    pub fn as_i32(&self) -> Option<i32> {
        self.as_i64().and_then(|value| number_to_signed!(i32, value, i64))
    }

    pub fn as_i16(&self) -> Option<i16> {
        self.as_i64().and_then(|value| number_to_signed!(i16, value, i64))
    }

    pub fn as_i8(&self) -> Option<i8> {
        self.as_i64().and_then(|value| number_to_signed!(i8, value, i64))
    }

    pub fn as_isize(&self) -> Option<isize> {
        self.as_i64().and_then(|value| number_to_signed!(isize, value, i64))
    }

    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            JsonValue::Boolean(ref value) => Some(*value),
            _                             => None
        }
    }

    /// Obtain an integer at a fixed decimal point. This is useful for
    /// converting monetary values and doing arithmetic on them without
    /// rounding errors introduced by floating point operations.
    ///
    /// Will return `None` if `Number` called on a value that's not a number,
    /// or if the number is negative or a NaN.
    ///
    /// ```
    /// # use json::JsonValue;
    /// let price_a = JsonValue::from(5.99);
    /// let price_b = JsonValue::from(7);
    /// let price_c = JsonValue::from(10.2);
    ///
    /// assert_eq!(price_a.as_fixed_point_u64(2), Some(599));
    /// assert_eq!(price_b.as_fixed_point_u64(2), Some(700));
    /// assert_eq!(price_c.as_fixed_point_u64(2), Some(1020));
    /// ```
    pub fn as_fixed_point_u64(&self, point: u16) -> Option<u64> {
        match *self {
            JsonValue::Number(ref value) => value.as_fixed_point_u64(point),
            _                            => None
        }
    }

    /// Analog to `as_fixed_point_u64`, except returning a signed
    /// `i64`, properly handling negative numbers.
    ///
    /// ```
    /// # use json::JsonValue;
    /// let balance_a = JsonValue::from(-1.49);
    /// let balance_b = JsonValue::from(42);
    ///
    /// assert_eq!(balance_a.as_fixed_point_i64(2), Some(-149));
    /// assert_eq!(balance_b.as_fixed_point_i64(2), Some(4200));
    /// ```
    pub fn as_fixed_point_i64(&self, point: u16) -> Option<i64> {
        match *self {
            JsonValue::Number(ref value) => value.as_fixed_point_i64(point),
            _                            => None
        }
    }

    /// Take over the ownership of the value, leaving `Null` in it's place.
    ///
    /// ## Example
    ///
    /// ```
    /// # #[macro_use] extern crate json;
    /// # fn main() {
    /// let mut data = array!["Foo", 42];
    ///
    /// let first = data[0].take();
    /// let second = data[1].take();
    ///
    /// assert!(first == "Foo");
    /// assert!(second == 42);
    ///
    /// assert!(data[0].is_null());
    /// assert!(data[1].is_null());
    /// # }
    /// ```
    pub fn take(&mut self) -> JsonValue {
        mem::replace(self, JsonValue::Null)
    }

    /// Checks that self is a string, returns an owned Rust `String`, leaving
    /// `Null` in it's place.
    ///
    /// - If the contained string is already a heap allocated `String`, then
    /// the ownership is moved without any heap allocation.
    ///
    /// - If the contained string is a `Short`, this will perform a heap
    /// allocation to convert the types for you.
    ///
    /// ## Example
    ///
    /// ```
    /// # #[macro_use] extern crate json;
    /// # fn main() {
    /// let mut data = array!["Hello", "World"];
    ///
    /// let owned = data[0].take_string().expect("Should be a string");
    ///
    /// assert_eq!(owned, "Hello");
    /// assert!(data[0].is_null());
    /// # }
    /// ```
    pub fn take_string(&mut self) -> Option<String> {
        let mut placeholder = JsonValue::Null;

        mem::swap(self, &mut placeholder);

        match placeholder {
            JsonValue::Short(short)   => return Some(short.into()),
            JsonValue::String(string) => return Some(string),

            // Not a string? Swap the original value back in place!
            _ => mem::swap(self, &mut placeholder)
        }

        None
    }

    /// Works on `JsonValue::Array` - pushes a new value to the array.
    pub fn push<T>(&mut self, value: T) -> Result<()>
    where T: Into<JsonValue> {
        match *self {
            JsonValue::Array(ref mut vec) => {
                vec.push(value.into());
                Ok(())
            },
            _ => Err(Error::wrong_type("Array"))
        }
    }

    /// Works on `JsonValue::Array` - remove and return last element from
    /// an array. On failure returns a null.
    pub fn pop(&mut self) -> JsonValue {
        match *self {
            JsonValue::Array(ref mut vec) => {
                vec.pop().unwrap_or(JsonValue::Null)
            },
            _ => JsonValue::Null
        }
    }

    /// Works on `JsonValue::Array` - checks if the array contains a value
    pub fn contains<T>(&self, item: T) -> bool where T: PartialEq<JsonValue> {
        match *self {
            JsonValue::Array(ref vec) => vec.iter().any(|member| item == *member),
            _                         => false
        }
    }

    /// Works on `JsonValue::Object` - checks if the object has a key
    pub fn has_key(&self, key: &str) -> bool {
        match *self {
            JsonValue::Object(ref object) => object.get(key).is_some(),
            _                             => false
        }
    }

    /// Returns length of array or object (number of keys), defaults to `0` for
    /// other types.
    pub fn len(&self) -> usize {
        match *self {
            JsonValue::Array(ref vec) => {
                vec.len()
            },
            JsonValue::Object(ref object) => {
                object.len()
            },
            _ => 0
        }
    }

    /// Works on `JsonValue::Array` - returns an iterator over members.
    /// Will return an empty iterator if called on non-array types.
    pub fn members(&self) -> Members {
        match *self {
            JsonValue::Array(ref vec) => {
                vec.iter()
            },
            _ => [].iter()
        }
    }

    /// Works on `JsonValue::Array` - returns a mutable iterator over members.
    /// Will return an empty iterator if called on non-array types.
    pub fn members_mut(&mut self) -> MembersMut {
        match *self {
            JsonValue::Array(ref mut vec) => {
                vec.iter_mut()
            },
            _ => [].iter_mut()
        }
    }

    /// Works on `JsonValue::Object` - returns an iterator over key value pairs.
    /// Will return an empty iterator if called on non-object types.
    pub fn entries(&self) -> Entries {
        match *self {
            JsonValue::Object(ref object) => {
                object.iter()
            },
            _ => Entries::empty()
        }
    }

    /// Works on `JsonValue::Object` - returns a mutable iterator over
    /// key value pairs.
    /// Will return an empty iterator if called on non-object types.
    pub fn entries_mut(&mut self) -> EntriesMut {
        match *self {
            JsonValue::Object(ref mut object) => {
                object.iter_mut()
            },
            _ => EntriesMut::empty()
        }
    }

    /// Works on `JsonValue::Object` - remove a key and return the value it held.
    /// If the key was not present, the method is called on anything but an
    /// object, it will return a null.
    pub fn remove(&mut self, key: &str) -> JsonValue {
        match *self {
            JsonValue::Object(ref mut object) => {
                object.remove(key).unwrap_or(JsonValue::Null)
            },
            _ => JsonValue::Null
        }
    }

    /// Works on `JsonValue::Array` - remove an entry and return the value it held.
    /// If the method is called on anything but an object or if the index is out of bounds, it
    /// will return `JsonValue::Null`.
    pub fn array_remove(&mut self, index: usize) -> JsonValue {
        match *self {
            JsonValue::Array(ref mut vec) => {
                if index < vec.len() {
                    vec.remove(index)
                } else {
                    JsonValue::Null
                }
            },
            _ => JsonValue::Null
        }
    }

    /// When called on an array or an object, will wipe them clean. When called
    /// on a string will clear the string. Numbers and booleans become null.
    pub fn clear(&mut self) {
        match *self {
            JsonValue::String(ref mut string) => string.clear(),
            JsonValue::Object(ref mut object) => object.clear(),
            JsonValue::Array(ref mut vec)     => vec.clear(),
            _                                 => *self = JsonValue::Null,
        }
    }
}

/// Implements indexing by `usize` to easily access array members:
///
/// ## Example
///
/// ```
/// # use json::JsonValue;
/// let mut array = JsonValue::new_array();
///
/// array.push("foo");
///
/// assert!(array[0] == "foo");
/// ```
impl Index<usize> for JsonValue {
    type Output = JsonValue;

    fn index(&self, index: usize) -> &JsonValue {
        match *self {
            JsonValue::Array(ref vec) => vec.get(index).unwrap_or(&NULL),
            _ => &NULL
        }
    }
}

/// Implements mutable indexing by `usize` to easily modify array members:
///
/// ## Example
///
/// ```
/// # #[macro_use]
/// # extern crate json;
/// #
/// # fn main() {
/// let mut array = array!["foo", 3.14];
///
/// array[1] = "bar".into();
///
/// assert!(array[1] == "bar");
/// # }
/// ```
impl IndexMut<usize> for JsonValue {
    fn index_mut(&mut self, index: usize) -> &mut JsonValue {
        match *self {
            JsonValue::Array(ref mut vec) => {
                let in_bounds = index < vec.len();

                if in_bounds {
                    &mut vec[index]
                } else {
                    vec.push(JsonValue::Null);
                    vec.last_mut().unwrap()
                }
            }
            _ => {
                *self = JsonValue::new_array();
                self.push(JsonValue::Null).unwrap();
                self.index_mut(index)
            }
        }
    }
}

/// Implements indexing by `&str` to easily access object members:
///
/// ## Example
///
/// ```
/// # #[macro_use]
/// # extern crate json;
/// #
/// # fn main() {
/// let object = object!{
///     "foo" => "bar"
/// };
///
/// assert!(object["foo"] == "bar");
/// # }
/// ```
impl<'a> Index<&'a str> for JsonValue {
    type Output = JsonValue;

    fn index(&self, index: &str) -> &JsonValue {
        match *self {
            JsonValue::Object(ref object) => &object[index],
            _ => &NULL
        }
    }
}

impl Index<String> for JsonValue {
    type Output = JsonValue;

    fn index(&self, index: String) -> &JsonValue {
        self.index(index.deref())
    }
}

impl<'a> Index<&'a String> for JsonValue {
    type Output = JsonValue;

    fn index(&self, index: &String) -> &JsonValue {
        self.index(index.deref())
    }
}

/// Implements mutable indexing by `&str` to easily modify object members:
///
/// ## Example
///
/// ```
/// # #[macro_use]
/// # extern crate json;
/// #
/// # fn main() {
/// let mut object = object!{};
///
/// object["foo"] = 42.into();
///
/// assert!(object["foo"] == 42);
/// # }
/// ```
impl<'a> IndexMut<&'a str> for JsonValue {
    fn index_mut(&mut self, index: &str) -> &mut JsonValue {
        match *self {
            JsonValue::Object(ref mut object) => {
                &mut object[index]
            },
            _ => {
                *self = JsonValue::new_object();
                self.index_mut(index)
            }
        }
    }
}

impl IndexMut<String> for JsonValue {
    fn index_mut(&mut self, index: String) -> &mut JsonValue {
        self.index_mut(index.deref())
    }
}

impl<'a> IndexMut<&'a String> for JsonValue {
    fn index_mut(&mut self, index: &String) -> &mut JsonValue {
        self.index_mut(index.deref())
    }
}
