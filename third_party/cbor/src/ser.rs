//! Serialize a Rust data structure to CBOR data.
use std::prelude::v1::*;
use byteorder::{ByteOrder, BigEndian};
use half::f16;
use serde::ser::{self, Serialize};
use std::io;

use error::{Error, Result};

/// Serializes a value to a writer.
pub fn to_writer<W, T>(mut writer: &mut W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ser::Serialize,
{
    value.serialize(&mut Serializer::new(&mut writer))
}

/// Serializes a value to a writer and adds a CBOR self-describe tag.
pub fn to_writer_sd<W, T>(mut writer: &mut W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ser::Serialize,
{
    let mut ser = Serializer::new(&mut writer);
    ser.self_describe()?;
    value.serialize(&mut ser)
}

/// Serializes a value without names to a writer.
///
/// Struct fields and enum variants are identified by their numeric indices rather than names to
/// save space.
pub fn to_writer_packed<W, T>(mut writer: &mut W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ser::Serialize,
{
    value.serialize(&mut Serializer::packed(&mut writer))
}

/// Serializes a value without names to a writer and adds a CBOR self-describe tag.
///
/// Struct fields and enum variants are identified by their numeric indices rather than names to
/// save space.
pub fn to_writer_packed_sd<W, T>(mut writer: &mut W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ser::Serialize,
{
    let mut ser = Serializer::packed(&mut writer);
    ser.self_describe()?;
    value.serialize(&mut ser)
}

/// Serializes a value to a vector.
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: ser::Serialize,
{
    let mut vec = Vec::new();
    to_writer(&mut vec, value)?;
    Ok(vec)
}

/// Serializes a value to a vector and adds a CBOR self-describe tag.
pub fn to_vec_sd<T>(value: &T) -> Result<Vec<u8>>
where
    T: ser::Serialize,
{
    let mut vec = Vec::new();
    to_writer_sd(&mut vec, value)?;
    Ok(vec)
}

/// Serializes a value without names to a vector.
///
/// Struct fields and enum variants are identified by their numeric indices rather than names to
/// save space.
pub fn to_vec_packed<T>(value: &T) -> Result<Vec<u8>>
where
    T: ser::Serialize,
{
    let mut vec = Vec::new();
    to_writer_packed(&mut vec, value)?;
    Ok(vec)
}
/// Serializes a value without names to a vector and adds a CBOR self-describe tag.
///
/// Struct fields and enum variants are identified by their numeric indices rather than names to
/// save space.
pub fn to_vec_packed_sd<T>(value: &T) -> Result<Vec<u8>>
where
    T: ser::Serialize,
{
    let mut vec = Vec::new();
    to_writer_packed_sd(&mut vec, value)?;
    Ok(vec)
}

/// A structure for serializing Rust values to CBOR.
pub struct Serializer<W> {
    writer: W,
    packed: bool,
}

impl<W> Serializer<W>
where
    W: io::Write,
{
    /// Creates a new CBOR serializer.
    #[inline]
    pub fn new(writer: W) -> Serializer<W> {
        Serializer {
            writer,
            packed: false,
        }
    }

    /// Creates a new "packed" CBOR serializer.
    ///
    /// Struct fields and enum variants are identified by their numeric indices rather than names
    /// to save space.
    #[inline]
    pub fn packed(writer: W) -> Serializer<W> {
        Serializer {
            writer,
            packed: true,
        }
    }

    /// Writes a CBOR self-describe tag to the stream.
    ///
    /// Tagging allows a decoder to distinguish different file formats based on their content
    /// without further information.
    #[inline]
    pub fn self_describe(&mut self) -> Result<()> {
        let mut buf = [6 << 5 | 25, 0, 0];
        BigEndian::write_u16(&mut buf[1..], 55799);
        self.writer.write_all(&buf).map_err(Error::io)
    }

    /// Unwrap the `Writer` from the `Serializer`.
    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    }

    #[inline]
    fn write_u8(&mut self, major: u8, value: u8) -> Result<()> {
        if value <= 0x17 {
            self.writer.write_all(&[major << 5 | value])
        } else {
            let buf = [major << 5 | 24, value];
            self.writer.write_all(&buf)
        }.map_err(Error::io)
    }

    #[inline]
    fn write_u16(&mut self, major: u8, value: u16) -> Result<()> {
        if value <= u16::from(u8::max_value()) {
            self.write_u8(major, value as u8)
        } else {
            let mut buf = [major << 5 | 25, 0, 0];
            BigEndian::write_u16(&mut buf[1..], value);
            self.writer.write_all(&buf).map_err(Error::io)
        }
    }

    #[inline]
    fn write_u32(&mut self, major: u8, value: u32) -> Result<()> {
        if value <= u32::from(u16::max_value()) {
            self.write_u16(major, value as u16)
        } else {
            let mut buf = [major << 5 | 26, 0, 0, 0, 0];
            BigEndian::write_u32(&mut buf[1..], value);
            self.writer.write_all(&buf).map_err(Error::io)
        }
    }

    #[inline]
    fn write_u64(&mut self, major: u8, value: u64) -> Result<()> {
        if value <= u64::from(u32::max_value()) {
            self.write_u32(major, value as u32)
        } else {
            let mut buf = [major << 5 | 27, 0, 0, 0, 0, 0, 0, 0, 0];
            BigEndian::write_u64(&mut buf[1..], value);
            self.writer.write_all(&buf).map_err(Error::io)
        }
    }

    #[inline]
    fn serialize_collection<'a>(
        &'a mut self,
        major: u8,
        len: Option<usize>,
    ) -> Result<CollectionSerializer<'a, W>> {
        let needs_eof = match len {
            Some(len) => {
                self.write_u64(major, len as u64)?;
                false
            }
            None => {
                self.writer.write_all(&[major << 5 | 31]).map_err(Error::io)?;
                true
            }
        };

        Ok(CollectionSerializer {
            ser: self,
            needs_eof,
        })
    }
}

impl<'a, W> ser::Serializer for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = CollectionSerializer<'a, W>;
    type SerializeTuple = &'a mut Serializer<W>;
    type SerializeTupleStruct = &'a mut Serializer<W>;
    type SerializeTupleVariant = &'a mut Serializer<W>;
    type SerializeMap = CollectionSerializer<'a, W>;
    type SerializeStruct = StructSerializer<'a, W>;
    type SerializeStructVariant = StructSerializer<'a, W>;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<()> {
        let value = if value { 0xf5 } else { 0xf4 };
        self.writer.write_all(&[value]).map_err(Error::io)
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<()> {
        if value < 0 {
            self.write_u8(1, -(value + 1) as u8)
        } else {
            self.write_u8(0, value as u8)
        }
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<()> {
        if value < 0 {
            self.write_u16(1, -(value + 1) as u16)
        } else {
            self.write_u16(0, value as u16)
        }
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<()> {
        if value < 0 {
            self.write_u32(1, -(value + 1) as u32)
        } else {
            self.write_u32(0, value as u32)
        }
    }

    #[inline]
    fn serialize_i64(self, value: i64) -> Result<()> {
        if value < 0 {
            self.write_u64(1, -(value + 1) as u64)
        } else {
            self.write_u64(0, value as u64)
        }
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<()> {
        self.write_u8(0, value)
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<()> {
        self.write_u16(0, value)
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<()> {
        self.write_u32(0, value)
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<()> {
        self.write_u64(0, value)
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<()> {
        if value.is_infinite() {
            if value.is_sign_positive() {
                self.writer.write_all(&[0xf9, 0x7c, 0x00])
            } else {
                self.writer.write_all(&[0xf9, 0xfc, 0x00])
            }
        } else if value.is_nan() {
            self.writer.write_all(&[0xf9, 0x7e, 0x00])
        } else if f32::from(f16::from_f32(value)) == value {
            let mut buf = [0xf9, 0, 0];
            BigEndian::write_u16(&mut buf[1..], f16::from_f32(value).to_bits());
            self.writer.write_all(&buf)
        } else {
            let mut buf = [0xfa, 0, 0, 0, 0];
            BigEndian::write_f32(&mut buf[1..], value);
            self.writer.write_all(&buf)
        }.map_err(Error::io)
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<()> {
        if !value.is_finite() || value as f32 as f64 == value {
            self.serialize_f32(value as f32)
        } else {
            let mut buf = [0xfb, 0, 0, 0, 0, 0, 0, 0, 0];
            BigEndian::write_f64(&mut buf[1..], value);
            self.writer.write_all(&buf).map_err(Error::io)
        }
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<()> {
        // A char encoded as UTF-8 takes 4 bytes at most.
        let mut buf = [0; 4];
        self.serialize_str(value.encode_utf8(&mut buf))
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<()> {
        self.write_u64(3, value.len() as u64)?;
        self.writer.write_all(value.as_bytes()).map_err(Error::io)
    }

    #[inline]
    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        self.write_u64(2, value.len() as u64)?;
        self.writer.write_all(value).map_err(Error::io)
    }

    #[inline]
    fn serialize_unit(self) -> Result<()> {
        self.serialize_none()
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_none(self) -> Result<()> {
        self.writer.write_all(&[0xf6]).map_err(Error::io)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        if self.packed {
            self.serialize_u32(variant_index)
        } else {
            self.serialize_str(variant)
        }
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.writer.write_all(&[4 << 5 | 2]).map_err(Error::io)?;
        self.serialize_unit_variant(name, variant_index, variant)?;
        value.serialize(self)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<CollectionSerializer<'a, W>> {
        self.serialize_collection(4, len)
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<&'a mut Serializer<W>> {
        self.write_u64(4, len as u64)?;
        Ok(self)
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<&'a mut Serializer<W>> {
        self.serialize_tuple(len)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<&'a mut Serializer<W>> {
        self.write_u64(4, (len + 1) as u64)?;
        self.serialize_unit_variant(name, variant_index, variant)?;
        Ok(self)
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<CollectionSerializer<'a, W>> {
        self.serialize_collection(5, len)
    }

    #[inline]
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<StructSerializer<'a, W>> {
        self.write_u64(5, len as u64)?;
        Ok(StructSerializer { ser: self, idx: 0 })
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<StructSerializer<'a, W>> {
        self.writer.write_all(&[4 << 5 | 2]).map_err(Error::io)?;
        self.serialize_unit_variant(name, variant_index, variant)?;
        self.serialize_struct(name, len)
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'a, W> ser::SerializeTuple for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleStruct for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleVariant for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

#[doc(hidden)]
pub struct StructSerializer<'a, W: 'a> {
    ser: &'a mut Serializer<W>,
    idx: u32,
}

impl<'a, W> StructSerializer<'a, W>
where
    W: io::Write,
{
    #[inline]
    fn serialize_field_inner<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        if self.ser.packed {
            self.idx.serialize(&mut *self.ser)?;
        } else {
            key.serialize(&mut *self.ser)?;
        }
        self.idx += 1;
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn skip_field_inner(&mut self, _: &'static str) -> Result<()> {
        self.idx += 1;
        Ok(())
    }
}

impl<'a, W> ser::SerializeStruct for StructSerializer<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.serialize_field_inner(key, value)
    }

    #[inline]
    fn skip_field(&mut self, key: &'static str) -> Result<()> {
        self.skip_field_inner(key)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeStructVariant for StructSerializer<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.serialize_field_inner(key, value)
    }

    #[inline]
    fn skip_field(&mut self, key: &'static str) -> Result<()> {
        self.skip_field_inner(key)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

#[doc(hidden)]
pub struct CollectionSerializer<'a, W: 'a> {
    ser: &'a mut Serializer<W>,
    needs_eof: bool,
}

impl<'a, W> CollectionSerializer<'a, W>
where
    W: io::Write,
{
    #[inline]
    fn end_inner(self) -> Result<()> {
        if self.needs_eof {
            self.ser.writer.write_all(&[0xff]).map_err(Error::io)
        } else {
            Ok(())
        }
    }
}

impl<'a, W> ser::SerializeSeq for CollectionSerializer<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        self.end_inner()
    }
}

impl<'a, W> ser::SerializeMap for CollectionSerializer<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        key.serialize(&mut *self.ser)
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        self.end_inner()
    }
}
