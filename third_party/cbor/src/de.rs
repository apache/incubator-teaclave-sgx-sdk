//! Deserialization.

use std::prelude::v1::*;
use byteorder::{ByteOrder, BigEndian};
use half::f16;
use serde::de;
use std::io;
use std::str;
use std::f32;
use std::result;
use std::marker::PhantomData;

use error::{Error, Result, ErrorCode};
use read::Reference;
pub use read::{Read, IoRead, SliceRead};

/// Decodes a value from CBOR data in a slice.
///
/// # Examples
///
/// Deserialize a `String`
///
/// ```
/// # use serde_cbor::de;
/// let v: Vec<u8> = vec![0x66, 0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72];
/// let value: String = de::from_slice(&v[..]).unwrap();
/// assert_eq!(value, "foobar");
/// ```
///
/// Deserialize a borrowed string with zero copies.
///
/// ```
/// # use serde_cbor::de;
/// let v: Vec<u8> = vec![0x66, 0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72];
/// let value: &str = de::from_slice(&v[..]).unwrap();
/// assert_eq!(value, "foobar");
/// ```
pub fn from_slice<'a, T>(slice: &'a [u8]) -> Result<T>
where
    T: de::Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_slice(slice);
    let value = de::Deserialize::deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(value)
}

/// Decodes a value from CBOR data in a reader.
///
/// # Examples
///
/// Deserialize a `String`
///
/// ```
/// # use serde_cbor::de;
/// let v: Vec<u8> = vec![0x66, 0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72];
/// let value: String = de::from_reader(&v[..]).unwrap();
/// assert_eq!(value, "foobar");
/// ```
///
/// Note that `from_reader` cannot borrow data:
///
/// ```compile_fail
/// # use serde_cbor::de;
/// let v: Vec<u8> = vec![0x66, 0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72];
/// let value: &str = de::from_reader(&v[..]).unwrap();
/// assert_eq!(value, "foobar");
/// ```
pub fn from_reader<T, R>(reader: R) -> Result<T>
where
    T: de::DeserializeOwned,
    R: io::Read,
{
    let mut deserializer = Deserializer::from_reader(reader);
    let value = de::Deserialize::deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(value)
}

/// A Serde `Deserialize`r of CBOR data.
pub struct Deserializer<R> {
    read: R,
    buf: Vec<u8>,
    remaining_depth: u8,
}

impl<R> Deserializer<IoRead<R>>
where
    R: io::Read,
{
    /// Constructs a `Deserializer` which reads from a `Read`er.
    pub fn from_reader(reader: R) -> Deserializer<IoRead<R>> {
        Deserializer::new(IoRead::new(reader))
    }
}

impl<'a> Deserializer<SliceRead<'a>> {
    /// Constructs a `Deserializer` which reads from a slice.
    ///
    /// Borrowed strings and byte slices will be provided when possible.
    pub fn from_slice(bytes: &'a [u8]) -> Deserializer<SliceRead<'a>> {
        Deserializer::new(SliceRead::new(bytes))
    }
}

impl<'de, R> Deserializer<R>
where
    R: Read<'de>,
{
    /// Constructs a `Deserializer` from one of the possible serde_cbor input sources.
    ///
    /// `from_slice` and `from_reader` should normally be used instead of this method.
    pub fn new(read: R) -> Self {
        Deserializer {
            read,
            buf: Vec::new(),
            remaining_depth: 128,
        }
    }

    /// This method should be called after a value has been deserialized to ensure there is no
    /// trailing data in the input source.
    pub fn end(&mut self) -> Result<()> {
        match self.next()? {
            Some(_) => Err(self.error(ErrorCode::TrailingData)),
            None => Ok(()),
        }
    }

    /// Turn a CBOR deserializer into an iterator over values of type T.
    pub fn into_iter<T>(self) -> StreamDeserializer<'de, R, T>
    where
        T: de::Deserialize<'de>,
    {
        StreamDeserializer {
            de: self,
            output: PhantomData,
            lifetime: PhantomData,
        }
    }

    fn next(&mut self) -> Result<Option<u8>> {
        self.read.next().map_err(Error::io)
    }

    fn peek(&mut self) -> Result<Option<u8>> {
        self.read.peek().map_err(Error::io)
    }

    fn consume(&mut self) {
        self.read.discard();
    }

    fn error(&self, reason: ErrorCode) -> Error {
        let offset = self.read.offset();
        Error::syntax(reason, offset)
    }

    fn parse_u8(&mut self) -> Result<u8> {
        match self.next()? {
            Some(byte) => Ok(byte),
            None => Err(self.error(ErrorCode::EofWhileParsingValue)),
        }
    }

    fn parse_u16(&mut self) -> Result<u16> {
        let mut buf = [0; 2];
        self.read.read_into(&mut buf)?;
        Ok(BigEndian::read_u16(&buf))
    }

    fn parse_u32(&mut self) -> Result<u32> {
        let mut buf = [0; 4];
        self.read.read_into(&mut buf)?;
        Ok(BigEndian::read_u32(&buf))
    }

    fn parse_u64(&mut self) -> Result<u64> {
        let mut buf = [0; 8];
        self.read.read_into(&mut buf)?;
        Ok(BigEndian::read_u64(&buf))
    }

    fn parse_bytes<V>(&mut self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.buf.clear();
        match self.read.read(len, &mut self.buf, 0)? {
            Reference::Borrowed(buf) => visitor.visit_borrowed_bytes(buf),
            Reference::Copied => visitor.visit_bytes(&self.buf),
        }
    }

    fn parse_indefinite_bytes(&mut self) -> Result<&[u8]> {
        let mut offset = 0;
        self.buf.clear();
        loop {
            let byte = self.parse_u8()?;
            let len = match byte {
                0x40...0x57 => byte as usize - 0x40,
                0x58 => self.parse_u8()? as usize,
                0x59 => self.parse_u16()? as usize,
                0x5a => self.parse_u32()? as usize,
                0x5b => {
                    let len = self.parse_u64()?;
                    if len > usize::max_value() as u64 {
                        return Err(self.error(ErrorCode::LengthOutOfRange));
                    }
                    len as usize
                }
                0xff => break,
                _ => return Err(self.error(ErrorCode::UnexpectedCode)),
            };

            match self.read.read(len, &mut self.buf, offset)? {
                Reference::Borrowed(buf) => {
                    let new_len = offset + len;
                    if new_len > self.buf.len() {
                        self.buf.resize(new_len, 0);
                    }
                    self.buf[offset..].copy_from_slice(buf);
                }
                Reference::Copied => {}
            }

            offset += len;
        }

        Ok(&self.buf[..offset])
    }

    fn convert_str<'a>(&self, buf: &'a [u8]) -> Result<&'a str> {
        match str::from_utf8(buf) {
            Ok(s) => Ok(s),
            Err(e) => {
                let shift = buf.len() - e.valid_up_to();
                let offset = self.read.offset() - shift as u64;
                Err(Error::syntax(ErrorCode::InvalidUtf8, offset))
            }
        }
    }

    fn parse_str<V>(&mut self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.buf.clear();
        match self.read.read(len, &mut self.buf, 0)? {
            Reference::Borrowed(buf) => {
                let s = self.convert_str(buf)?;
                visitor.visit_borrowed_str(s)
            }
            Reference::Copied => {
                let s = self.convert_str(&self.buf)?;
                visitor.visit_str(s)
            }
        }
    }

    fn parse_indefinite_str(&mut self) -> Result<&str> {
        let mut offset = 0;
        self.buf.clear();
        loop {
            let byte = self.parse_u8()?;
            let len = match byte {
                0x60...0x77 => byte as usize - 0x60,
                0x78 => self.parse_u8()? as usize,
                0x79 => self.parse_u16()? as usize,
                0x7a => self.parse_u32()? as usize,
                0x7b => {
                    let len = self.parse_u64()?;
                    if len > usize::max_value() as u64 {
                        return Err(self.error(ErrorCode::LengthOutOfRange));
                    }
                    len as usize
                }
                0xff => break,
                _ => return Err(self.error(ErrorCode::UnexpectedCode)),
            };

            match self.read.read(len, &mut self.buf, offset)? {
                Reference::Borrowed(buf) => {
                    let new_len = offset + len;
                    if new_len > self.buf.len() {
                        self.buf.resize(new_len, 0);
                    }
                    self.buf[offset..].copy_from_slice(buf);
                }
                Reference::Copied => {}
            }

            offset += len;
        }

        self.convert_str(&self.buf[..offset])
    }

    fn recursion_checked<F, T>(&mut self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Deserializer<R>) -> Result<T>,
    {
        self.remaining_depth -= 1;
        if self.remaining_depth == 0 {
            return Err(self.error(ErrorCode::RecursionLimitExceeded));
        }
        let r = f(self);
        self.remaining_depth += 1;
        r
    }

    fn parse_array<V>(&mut self, mut len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.recursion_checked(|de| {
            let value = visitor.visit_seq(SeqAccess { de, len: &mut len })?;

            if len != 0 {
                Err(de.error(ErrorCode::TrailingData))
            } else {
                Ok(value)
            }
        })
    }

    fn parse_indefinite_array<V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.recursion_checked(|de| {
            let value = visitor.visit_seq(IndefiniteSeqAccess { de })?;
            match de.next()? {
                Some(0xff) => Ok(value),
                Some(_) => Err(de.error(ErrorCode::TrailingData)),
                None => Err(de.error(ErrorCode::EofWhileParsingArray)),
            }
        })
    }

    fn parse_map<V>(&mut self, mut len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.recursion_checked(|de| {
            let value = visitor.visit_map(MapAccess { de, len: &mut len })?;

            if len != 0 {
                Err(de.error(ErrorCode::TrailingData))
            } else {
                Ok(value)
            }
        })
    }

    fn parse_indefinite_map<V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.recursion_checked(|de| {
            let value = visitor.visit_map(IndefiniteMapAccess { de })?;
            match de.next()? {
                Some(0xff) => Ok(value),
                Some(_) => Err(de.error(ErrorCode::TrailingData)),
                None => Err(de.error(ErrorCode::EofWhileParsingMap)),
            }
        })
    }

    fn parse_enum<V>(&mut self, mut len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.recursion_checked(|de| {
            let value = visitor.visit_enum(VariantAccess {
                seq: SeqAccess { de, len: &mut len },
            })?;

            if len != 0 {
                Err(de.error(ErrorCode::TrailingData))
            } else {
                Ok(value)
            }
        })
    }

    fn parse_indefinite_enum<V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.recursion_checked(|de| {
            let value = visitor.visit_enum(
                VariantAccess { seq: IndefiniteSeqAccess { de } },
            )?;
            match de.next()? {
                Some(0xff) => Ok(value),
                Some(_) => Err(de.error(ErrorCode::TrailingData)),
                None => Err(de.error(ErrorCode::EofWhileParsingArray)),
            }
        })
    }

    fn parse_f16(&mut self) -> Result<f32> {
        Ok(f32::from(f16::from_bits(self.parse_u16()?)))
    }

    fn parse_f32(&mut self) -> Result<f32> {
        let mut buf = [0; 4];
        self.read.read_into(&mut buf)?;
        Ok(BigEndian::read_f32(&buf))
    }

    fn parse_f64(&mut self) -> Result<f64> {
        let mut buf = [0; 8];
        self.read.read_into(&mut buf)?;
        Ok(BigEndian::read_f64(&buf))
    }

    fn parse_value<V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let byte = self.parse_u8()?;
        match byte {
            // Major type 0: an unsigned integer
            0x00...0x17 => visitor.visit_u8(byte),
            0x18 => {
                let value = self.parse_u8()?;
                visitor.visit_u8(value)
            }
            0x19 => {
                let value = self.parse_u16()?;
                visitor.visit_u16(value)
            }
            0x1a => {
                let value = self.parse_u32()?;
                visitor.visit_u32(value)
            }
            0x1b => {
                let value = self.parse_u64()?;
                visitor.visit_u64(value)
            }
            0x1c...0x1f => Err(self.error(ErrorCode::UnassignedCode)),

            // Major type 1: a negative integer
            0x20...0x37 => visitor.visit_i8(-1 - (byte - 0x20) as i8),
            0x38 => {
                let value = self.parse_u8()?;
                visitor.visit_i16(-1 - i16::from(value))
            }
            0x39 => {
                let value = self.parse_u16()?;
                visitor.visit_i32(-1 - i32::from(value))
            }
            0x3a => {
                let value = self.parse_u32()?;
                visitor.visit_i64(-1 - i64::from(value))
            }
            0x3b => {
                let value = self.parse_u64()?;
                if value > i64::max_value() as u64 {
                    return Err(self.error(ErrorCode::NumberOutOfRange));
                }
                visitor.visit_i64(-1 - value as i64)
            }
            0x3c...0x3f => Err(self.error(ErrorCode::UnassignedCode)),

            // Major type 2: a byte string
            0x40...0x57 => self.parse_bytes(byte as usize - 0x40, visitor),
            0x58 => {
                let len = self.parse_u8()?;
                self.parse_bytes(len as usize, visitor)
            }
            0x59 => {
                let len = self.parse_u16()?;
                self.parse_bytes(len as usize, visitor)
            }
            0x5a => {
                let len = self.parse_u32()?;
                self.parse_bytes(len as usize, visitor)
            }
            0x5b => {
                let len = self.parse_u64()?;
                if len > usize::max_value() as u64 {
                    return Err(self.error(ErrorCode::LengthOutOfRange));
                }
                self.parse_bytes(len as usize, visitor)
            }
            0x5c...0x5e => Err(self.error(ErrorCode::UnassignedCode)),
            0x5f => {
                let bytes = self.parse_indefinite_bytes()?;
                visitor.visit_bytes(bytes)
            }

            // Major type 3: a text string
            0x60...0x77 => self.parse_str(byte as usize - 0x60, visitor),
            0x78 => {
                let len = self.parse_u8()?;
                self.parse_str(len as usize, visitor)
            }
            0x79 => {
                let len = self.parse_u16()?;
                self.parse_str(len as usize, visitor)
            }
            0x7a => {
                let len = self.parse_u32()?;
                self.parse_str(len as usize, visitor)
            }
            0x7b => {
                let len = self.parse_u64()?;
                if len > usize::max_value() as u64 {
                    return Err(self.error(ErrorCode::LengthOutOfRange));
                }
                self.parse_str(len as usize, visitor)
            }
            0x7c...0x7e => Err(self.error(ErrorCode::UnassignedCode)),
            0x7f => {
                let s = self.parse_indefinite_str()?;
                visitor.visit_str(s)
            }

            // Major type 4: an array of data items
            0x80...0x97 => self.parse_array(byte as usize - 0x80, visitor),
            0x98 => {
                let len = self.parse_u8()?;
                self.parse_array(len as usize, visitor)
            }
            0x99 => {
                let len = self.parse_u16()?;
                self.parse_array(len as usize, visitor)
            }
            0x9a => {
                let len = self.parse_u32()?;
                self.parse_array(len as usize, visitor)
            }
            0x9b => {
                let len = self.parse_u64()?;
                if len > usize::max_value() as u64 {
                    return Err(self.error(ErrorCode::LengthOutOfRange));
                }
                self.parse_array(len as usize, visitor)
            }
            0x9c...0x9e => Err(self.error(ErrorCode::UnassignedCode)),
            0x9f => self.parse_indefinite_array(visitor),

            // Major type 5: a map of pairs of data items
            0xa0...0xb7 => self.parse_map(byte as usize - 0xa0, visitor),
            0xb8 => {
                let len = self.parse_u8()?;
                self.parse_map(len as usize, visitor)
            }
            0xb9 => {
                let len = self.parse_u16()?;
                self.parse_map(len as usize, visitor)
            }
            0xba => {
                let len = self.parse_u32()?;
                self.parse_map(len as usize, visitor)
            }
            0xbb => {
                let len = self.parse_u64()?;
                if len > usize::max_value() as u64 {
                    return Err(self.error(ErrorCode::LengthOutOfRange));
                }
                self.parse_map(len as usize, visitor)
            }
            0xbc...0xbe => Err(self.error(ErrorCode::UnassignedCode)),
            0xbf => self.parse_indefinite_map(visitor),

            // Major type 6: optional semantic tagging of other major types
            0xc0...0xd7 => self.parse_value(visitor),
            0xd8 => {
                self.parse_u8()?;
                self.parse_value(visitor)
            }
            0xd9 => {
                self.parse_u16()?;
                self.parse_value(visitor)
            }
            0xda => {
                self.parse_u32()?;
                self.parse_value(visitor)
            }
            0xdb => {
                self.parse_u64()?;
                self.parse_value(visitor)
            }
            0xdc...0xdf => Err(self.error(ErrorCode::UnassignedCode)),

            // Major type 7: floating-point numbers and other simple data types that need no content
            0xe0...0xf3 => Err(self.error(ErrorCode::UnassignedCode)),
            0xf4 => visitor.visit_bool(false),
            0xf5 => visitor.visit_bool(true),
            0xf6 => visitor.visit_unit(),
            0xf7 => visitor.visit_unit(),
            0xf8 => Err(self.error(ErrorCode::UnassignedCode)),
            0xf9 => {
                let value = self.parse_f16()?;
                visitor.visit_f32(value)
            }
            0xfa => {
                let value = self.parse_f32()?;
                visitor.visit_f32(value)
            }
            0xfb => {
                let value = self.parse_f64()?;
                visitor.visit_f64(value)
            }
            0xfc...0xfe => Err(self.error(ErrorCode::UnassignedCode)),
            0xff => Err(self.error(ErrorCode::UnexpectedCode)),

            // https://github.com/rust-lang/rust/issues/12483
            //_ => unreachable!(),
        }
    }
}

impl<'de, 'a, R> de::Deserializer<'de> for &'a mut Deserializer<R>
where
    R: Read<'de>,
{
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.parse_value(visitor)
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.peek()? {
            Some(0xf6) => {
                self.consume();
                visitor.visit_none()
            }
            _ => visitor.visit_some(self),
        }
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _name: &str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    // Unit variants are encoded as just the variant identifier.
    // Tuple variants are encoded as an array of the variant identifier followed by the fields.
    // Struct variants are encoded as an array of the variant identifier followed by the struct.
    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.peek()? {
            Some(byte @ 0x80...0x9f) => {
                self.consume();
                match byte {
                    0x80...0x97 => self.parse_enum(byte as usize - 0x80, visitor),
                    0x98 => {
                        let len = self.parse_u8()?;
                        self.parse_enum(len as usize, visitor)
                    }
                    0x99 => {
                        let len = self.parse_u16()?;
                        self.parse_enum(len as usize, visitor)
                    }
                    0x9a => {
                        let len = self.parse_u32()?;
                        self.parse_enum(len as usize, visitor)
                    }
                    0x9b => {
                        let len = self.parse_u64()?;
                        if len > usize::max_value() as u64 {
                            return Err(self.error(ErrorCode::LengthOutOfRange));
                        }
                        self.parse_enum(len as usize, visitor)
                    }
                    0x9c...0x9e => Err(self.error(ErrorCode::UnassignedCode)),
                    0x9f => self.parse_indefinite_enum(visitor),

                    _ => unreachable!(),
                }
            }
            None => Err(self.error(ErrorCode::EofWhileParsingValue)),
            _ => visitor.visit_enum(UnitVariantAccess { de: self }),
        }
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string unit
        unit_struct seq tuple tuple_struct map struct identifier ignored_any
        bytes byte_buf
    }
}

trait MakeError {
    fn error(&self, code: ErrorCode) -> Error;
}

struct SeqAccess<'a, R: 'a> {
    de: &'a mut Deserializer<R>,
    len: &'a mut usize,
}

impl<'de, 'a, R> de::SeqAccess<'de> for SeqAccess<'a, R>
where
    R: Read<'de>,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if *self.len == 0 {
            return Ok(None);
        }
        *self.len -= 1;

        let value = seed.deserialize(&mut *self.de)?;
        Ok(Some(value))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(*self.len)
    }
}

impl<'de, 'a, R> MakeError for SeqAccess<'a, R>
where
    R: Read<'de>,
{
    fn error(&self, code: ErrorCode) -> Error {
        self.de.error(code)
    }
}

struct IndefiniteSeqAccess<'a, R: 'a> {
    de: &'a mut Deserializer<R>,
}

impl<'de, 'a, R> de::SeqAccess<'de> for IndefiniteSeqAccess<'a, R>
where
    R: Read<'de>,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.de.peek()? {
            Some(0xff) => return Ok(None),
            Some(_) => {}
            None => return Err(self.de.error(ErrorCode::EofWhileParsingArray)),
        }

        let value = seed.deserialize(&mut *self.de)?;
        Ok(Some(value))
    }
}

impl<'de, 'a, R> MakeError for IndefiniteSeqAccess<'a, R>
where
    R: Read<'de>,
{
    fn error(&self, code: ErrorCode) -> Error {
        self.de.error(code)
    }
}

struct MapAccess<'a, R: 'a> {
    de: &'a mut Deserializer<R>,
    len: &'a mut usize,
}

impl<'de, 'a, R> de::MapAccess<'de> for MapAccess<'a, R>
where
    R: Read<'de>,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if *self.len == 0 {
            return Ok(None);
        }
        *self.len -= 1;

        let value = seed.deserialize(&mut *self.de)?;
        Ok(Some(value))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(*self.len)
    }
}

struct IndefiniteMapAccess<'a, R: 'a> {
    de: &'a mut Deserializer<R>,
}

impl<'de, 'a, R> de::MapAccess<'de> for IndefiniteMapAccess<'a, R>
where
    R: Read<'de>,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.de.peek()? {
            Some(0xff) => return Ok(None),
            Some(_) => {}
            None => return Err(self.de.error(ErrorCode::EofWhileParsingMap)),
        }

        let value = seed.deserialize(&mut *self.de)?;
        Ok(Some(value))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct UnitVariantAccess<'a, R: 'a> {
    de: &'a mut Deserializer<R>,
}

impl<'de, 'a, R> de::EnumAccess<'de> for UnitVariantAccess<'a, R>
where
    R: Read<'de>,
{
    type Error = Error;
    type Variant = UnitVariantAccess<'a, R>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, UnitVariantAccess<'a, R>)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(&mut *self.de)?;
        Ok((variant, self))
    }
}

impl<'de, 'a, R> de::VariantAccess<'de> for UnitVariantAccess<'a, R>
where
    R: Read<'de>,
{
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        Err(de::Error::invalid_type(
            de::Unexpected::UnitVariant,
            &"newtype variant",
        ))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            de::Unexpected::UnitVariant,
            &"tuple variant",
        ))
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            de::Unexpected::UnitVariant,
            &"struct variant",
        ))
    }
}

struct VariantAccess<T> {
    seq: T,
}

impl<'de, T> de::EnumAccess<'de> for VariantAccess<T>
where
    T: de::SeqAccess<'de, Error = Error> + MakeError,
{
    type Error = Error;
    type Variant = VariantAccess<T>;

    fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, VariantAccess<T>)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = match self.seq.next_element_seed(seed) {
            Ok(Some(variant)) => variant,
            Ok(None) => return Err(self.seq.error(ErrorCode::ArrayTooShort)),
            Err(e) => return Err(e),
        };
        Ok((variant, self))
    }
}

impl<'de, T> de::VariantAccess<'de> for VariantAccess<T>
where
    T: de::SeqAccess<'de, Error = Error>
        + MakeError,
{
    type Error = Error;

    fn unit_variant(mut self) -> Result<()> {
        match self.seq.next_element() {
            Ok(Some(())) => Ok(()),
            Ok(None) => Err(self.seq.error(ErrorCode::ArrayTooLong)),
            Err(e) => Err(e),
        }
    }

    fn newtype_variant_seed<S>(mut self, seed: S) -> Result<S::Value>
    where
        S: de::DeserializeSeed<'de>,
    {
        match self.seq.next_element_seed(seed) {
            Ok(Some(variant)) => Ok(variant),
            Ok(None) => Err(self.seq.error(ErrorCode::ArrayTooShort)),
            Err(e) => Err(e),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(self.seq)
    }

    fn struct_variant<V>(mut self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let seed = StructVariantSeed { visitor };
        match self.seq.next_element_seed(seed) {
            Ok(Some(variant)) => Ok(variant),
            Ok(None) => Err(self.seq.error(ErrorCode::ArrayTooShort)),
            Err(e) => Err(e),
        }
    }
}

struct StructVariantSeed<V> {
    visitor: V,
}

impl<'de, V> de::DeserializeSeed<'de> for StructVariantSeed<V>
where
    V: de::Visitor<'de>,
{
    type Value = V::Value;

    fn deserialize<D>(self, de: D) -> result::Result<V::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        de.deserialize_any(self.visitor)
    }
}

/// Iterator that deserializes a stream into multiple CBOR values.
///
/// A stream deserializer can be created from any CBOR deserializer using the
/// `Deserializer::into_iter` method.
pub struct StreamDeserializer<'de, R, T> {
    de: Deserializer<R>,
    output: PhantomData<T>,
    lifetime: PhantomData<&'de ()>,
}

impl<'de, R, T> StreamDeserializer<'de, R, T>
where
    R: Read<'de>,
    T: de::Deserialize<'de>,
{
    /// Create a new CBOR stream deserializer from one of the possible
    /// serde_cbor input sources.
    ///
    /// Typically it is more convenient to use one of these methods instead:
    ///
    /// * `Deserializer::from_bytes(...).into_iter()`
    /// * `Deserializer::from_reader(...).into_iter()`
    pub fn new(read: R) -> StreamDeserializer<'de, R, T> {
        StreamDeserializer {
            de: Deserializer::new(read),
            output: PhantomData,
            lifetime: PhantomData,
        }
    }
}

impl<'de, R, T> Iterator for StreamDeserializer<'de, R, T>
where
    R: Read<'de>,
    T: de::Deserialize<'de>,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Result<T>> {
        match self.de.peek() {
            Ok(Some(_)) => Some(T::deserialize(&mut self.de)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}
