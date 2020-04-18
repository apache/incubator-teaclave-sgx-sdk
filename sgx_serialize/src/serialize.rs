// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

//! Support code for encoding and decoding types.

/*
Core encoding and decoding interfaces.
*/

use std::borrow::Cow;
use std::path;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::sync::Arc;
use std::boxed::Box;
use std::string::String;
use std::vec::Vec;
use std::borrow::{ToOwned};

pub trait Encoder {
    /// The error type for method results.
    type Error;

    // Primitive types:
    fn emit_nil(&mut self) -> Result<(), Self::Error>;
    fn emit_usize(&mut self, v: usize) -> Result<(), Self::Error>;
    fn emit_u128(&mut self, v: u128) -> Result<(), Self::Error>;
    fn emit_u64(&mut self, v: u64) -> Result<(), Self::Error>;
    fn emit_u32(&mut self, v: u32) -> Result<(), Self::Error>;
    fn emit_u16(&mut self, v: u16) -> Result<(), Self::Error>;
    fn emit_u8(&mut self, v: u8) -> Result<(), Self::Error>;
    fn emit_isize(&mut self, v: isize) -> Result<(), Self::Error>;
    fn emit_i64(&mut self, v: i64) -> Result<(), Self::Error>;
    fn emit_i32(&mut self, v: i32) -> Result<(), Self::Error>;
    fn emit_i16(&mut self, v: i16) -> Result<(), Self::Error>;
    fn emit_i8(&mut self, v: i8) -> Result<(), Self::Error>;
    fn emit_bool(&mut self, v: bool) -> Result<(), Self::Error>;
    fn emit_f64(&mut self, v: f64) -> Result<(), Self::Error>;
    fn emit_f32(&mut self, v: f32) -> Result<(), Self::Error>;
    fn emit_char(&mut self, v: char) -> Result<(), Self::Error>;
    fn emit_str(&mut self, v: &str) -> Result<(), Self::Error>;
    fn emit_i128(&mut self, v: i128) -> Result<(), Self::Error>;

    // Compound types:
    fn emit_enum<F>(&mut self, _name: &str, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        f(self)
    }

    fn emit_enum_variant<F>(
        &mut self, _v_name: &str,
        v_id: usize,
        _len: usize,
        f: F,
    ) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        self.emit_usize(v_id)?;
        f(self)
    }
    fn emit_enum_variant_arg<F>(
        &mut self,
        _a_idx: usize,
        f: F,
    ) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        f(self)
    }

    fn emit_enum_struct_variant<F>(
        &mut self, v_name: &str,
        v_id: usize,
        len: usize,
        f: F,
    ) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        self.emit_enum_variant(v_name, v_id, len, f)
    }

    fn emit_enum_struct_variant_field<F>(
        &mut self,
        _f_name: &str,
        f_idx: usize,
        f: F,
    ) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        self.emit_enum_variant_arg(f_idx, f)
    }

    fn emit_struct<F>(
        &mut self,
        _name: &str,
        _len: usize,
        f: F,
    ) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        f(self)
    }

    fn emit_struct_field<F>(
        &mut self,
        _f_name: &str,
        _f_idx: usize,
        f: F,
    ) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        f(self)
    }

    fn emit_tuple<F>(&mut self, _len: usize, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        f(self)
    }

    fn emit_tuple_arg<F>(&mut self, _idx: usize, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        f(self)
    }

    fn emit_tuple_struct<F>(
        &mut self,
        _name: &str,
        len: usize,
        f: F,
    ) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        self.emit_tuple(len, f)
    }

    fn emit_tuple_struct_arg<F>(
        &mut self,
        f_idx: usize,
        f: F,
    ) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        self.emit_tuple_arg(f_idx, f)
    }

    // Specialized types:
    fn emit_option<F>(&mut self, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        self.emit_enum("Option", f)
    }

    fn emit_option_none(&mut self) -> Result<(), Self::Error> {
        self.emit_enum_variant("None", 0, 0, |_| Ok(()))
    }

    fn emit_option_some<F>(&mut self, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        self.emit_enum_variant("Some", 1, 1, f)
    }

    fn emit_seq<F>(&mut self, len: usize, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        self.emit_usize(len)?;
        f(self)
    }

    fn emit_seq_elt<F>(&mut self, _idx: usize, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        f(self)
    }

    fn emit_map<F>(&mut self, len: usize, f: F) -> Result<(), Self::Error>
    where F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        self.emit_usize(len)?;
        f(self)
    }
    fn emit_map_elt_key<F>(&mut self, _idx: usize, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        f(self)
    }

    fn emit_map_elt_val<F>(&mut self, _idx: usize, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        f(self)
    }
}

pub trait Decoder {
    type Error;

    // Primitive types:
    fn read_nil(&mut self) -> Result<(), Self::Error>;
    fn read_usize(&mut self) -> Result<usize, Self::Error>;
    fn read_u128(&mut self) -> Result<u128, Self::Error>;
    fn read_u64(&mut self) -> Result<u64, Self::Error>;
    fn read_u32(&mut self) -> Result<u32, Self::Error>;
    fn read_u16(&mut self) -> Result<u16, Self::Error>;
    fn read_u8(&mut self) -> Result<u8, Self::Error>;
    fn read_isize(&mut self) -> Result<isize, Self::Error>;
    fn read_i128(&mut self) -> Result<i128, Self::Error>;
    fn read_i64(&mut self) -> Result<i64, Self::Error>;
    fn read_i32(&mut self) -> Result<i32, Self::Error>;
    fn read_i16(&mut self) -> Result<i16, Self::Error>;
    fn read_i8(&mut self) -> Result<i8, Self::Error>;
    fn read_bool(&mut self) -> Result<bool, Self::Error>;
    fn read_f64(&mut self) -> Result<f64, Self::Error>;
    fn read_f32(&mut self) -> Result<f32, Self::Error>;
    fn read_char(&mut self) -> Result<char, Self::Error>;
    fn read_str(&mut self) -> Result<Cow<str>, Self::Error>;

    // Compound types:
    fn read_enum<T, F>(&mut self, _name: &str, f: F) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Self::Error>,
    {
        f(self)
    }

    fn read_enum_variant<T, F>(
        &mut self,
        _names: &[&str],
        mut f: F,
    ) -> Result<T, Self::Error>
    where
        F: FnMut(&mut Self, usize) -> Result<T, Self::Error>,
    {
        let disr = self.read_usize()?;
        f(self, disr)
    }

    fn read_enum_variant_arg<T, F>(
        &mut self,
        _a_idx: usize,
        f: F,
    ) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Self::Error>,
    {
        f(self)
    }

    fn read_enum_struct_variant<T, F>(
        &mut self,
        names: &[&str],
        f: F,
    ) -> Result<T, Self::Error>
    where
        F: FnMut(&mut Self, usize) -> Result<T, Self::Error>,
    {
        self.read_enum_variant(names, f)
    }

    fn read_enum_struct_variant_field<T, F>(
        &mut self,
        _f_name: &str,
        f_idx: usize,
        f: F,
    ) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Self::Error>,
    {
        self.read_enum_variant_arg(f_idx, f)
    }

    fn read_struct<T, F>(
        &mut self,
        _s_name: &str,
        _len: usize,
        f: F,
    ) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Self::Error>,
    {
        f(self)
    }

    fn read_struct_field<T, F>(
        &mut self,
        _f_name: &str,
        _f_idx: usize,
        f: F,
    ) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Self::Error>,
    {
        f(self)
    }

    fn read_tuple<T, F>(&mut self, _len: usize, f: F) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Self::Error>,
    {
        f(self)
    }

    fn read_tuple_arg<T, F>(
        &mut self,
        _a_idx: usize,
        f: F,
    ) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Self::Error>,
    {
        f(self)
    }

    fn read_tuple_struct<T, F>(
        &mut self,
        _s_name: &str,
        len: usize,
        f: F,
    ) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Self::Error>,
    {
        self.read_tuple(len, f)
    }

    fn read_tuple_struct_arg<T, F>(&mut self, a_idx: usize, f: F) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Self::Error>,
    {
        self.read_tuple_arg(a_idx, f)
    }

    // Specialized types:
    fn read_option<T, F>(&mut self, mut f: F) -> Result<T, Self::Error>
    where
        F: FnMut(&mut Self, bool) -> Result<T, Self::Error>,
    {
        self.read_enum("Option", move |this| {
            this.read_enum_variant(&["None", "Some"], move |this, idx| {
                match idx {
                    0 => f(this, false),
                    1 => f(this, true),
                    _ => Err(this.error("read_option: expected 0 for None or 1 for Some")),
                }
            })
        })
    }

    fn read_seq<T, F>(&mut self, f: F) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self, usize) -> Result<T, Self::Error>,
    {
        let len = self.read_usize()?;
        f(self, len)
    }

    fn read_seq_elt<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Self::Error>,
    {
        f(self)
    }

    fn read_map<T, F>(&mut self, f: F) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self, usize) -> Result<T, Self::Error>,
    {
        let len = self.read_usize()?;
        f(self, len)
    }

    fn read_map_elt_key<T, F>(
        &mut self,
        _idx: usize,
        f: F,
    ) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Self::Error>,
    {
        f(self)
    }

    fn read_map_elt_val<T, F>(
        &mut self,
        _idx: usize,
        f: F,
    ) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Self::Error>,
    {
        f(self)
    }

    // Failure
    fn error(&mut self, err: &str) -> Self::Error;
}

pub trait DeSerializable: Sized {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error>;
}

pub trait Serializable {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error>;
}

impl Serializable for usize {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_usize(*self)
    }
}

impl DeSerializable for usize {
    fn decode<D: Decoder>(d: &mut D) -> Result<usize, D::Error> {
        d.read_usize()
    }
}

impl Serializable for u8 {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_u8(*self)
    }
}

impl DeSerializable for u8 {
    fn decode<D: Decoder>(d: &mut D) -> Result<u8, D::Error> {
        d.read_u8()
    }
}

impl Serializable for u16 {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_u16(*self)
    }
}

impl DeSerializable for u16 {
    fn decode<D: Decoder>(d: &mut D) -> Result<u16, D::Error> {
        d.read_u16()
    }
}

impl Serializable for u32 {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_u32(*self)
    }
}

impl DeSerializable for u32 {
    fn decode<D: Decoder>(d: &mut D) -> Result<u32, D::Error> {
        d.read_u32()
    }
}

impl Serializable for u64 {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_u64(*self)
    }
}

impl DeSerializable for u64 {
    fn decode<D: Decoder>(d: &mut D) -> Result<u64, D::Error> {
        d.read_u64()
    }
}

impl Serializable for u128 {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_u128(*self)
    }
}

impl DeSerializable for u128 {
    fn decode<D: Decoder>(d: &mut D) -> Result<u128, D::Error> {
        d.read_u128()
    }
}

impl Serializable for isize {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_isize(*self)
    }
}

impl DeSerializable for isize {
    fn decode<D: Decoder>(d: &mut D) -> Result<isize, D::Error> {
        d.read_isize()
    }
}

impl Serializable for i8 {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_i8(*self)
    }
}

impl DeSerializable for i8 {
    fn decode<D: Decoder>(d: &mut D) -> Result<i8, D::Error> {
        d.read_i8()
    }
}

impl Serializable for i16 {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_i16(*self)
    }
}

impl DeSerializable for i16 {
    fn decode<D: Decoder>(d: &mut D) -> Result<i16, D::Error> {
        d.read_i16()
    }
}

impl Serializable for i32 {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_i32(*self)
    }
}

impl DeSerializable for i32 {
    fn decode<D: Decoder>(d: &mut D) -> Result<i32, D::Error> {
        d.read_i32()
    }
}

impl Serializable for i64 {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_i64(*self)
    }
}

impl DeSerializable for i64 {
    fn decode<D: Decoder>(d: &mut D) -> Result<i64, D::Error> {
        d.read_i64()
    }
}

impl Serializable for i128 {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_i128(*self)
    }
}

impl DeSerializable for i128 {
    fn decode<D: Decoder>(d: &mut D) -> Result<i128, D::Error> {
        d.read_i128()
    }
}

impl Serializable for str {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_str(self)
    }
}

impl Serializable for String {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_str(&self[..])
    }
}

impl DeSerializable for String {
    fn decode<D: Decoder>(d: &mut D) -> Result<String, D::Error> {
        Ok(d.read_str()?.into_owned())
    }
}

impl Serializable for f32 {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_f32(*self)
    }
}

impl DeSerializable for f32 {
    fn decode<D: Decoder>(d: &mut D) -> Result<f32, D::Error> {
        d.read_f32()
    }
}

impl Serializable for f64 {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_f64(*self)
    }
}

impl DeSerializable for f64 {
    fn decode<D: Decoder>(d: &mut D) -> Result<f64, D::Error> {
        d.read_f64()
    }
}

impl Serializable for bool {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_bool(*self)
    }
}

impl DeSerializable for bool {
    fn decode<D: Decoder>(d: &mut D) -> Result<bool, D::Error> {
        d.read_bool()
    }
}

impl Serializable for char {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_char(*self)
    }
}

impl DeSerializable for char {
    fn decode<D: Decoder>(d: &mut D) -> Result<char, D::Error> {
        d.read_char()
    }
}

impl Serializable for () {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_nil()
    }
}

impl DeSerializable for () {
    fn decode<D: Decoder>(d: &mut D) -> Result<(), D::Error> {
        d.read_nil()
    }
}

impl<'a, T: ?Sized + Serializable> Serializable for &'a T {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        (**self).encode(s)
    }
}

impl<T: ?Sized + Serializable> Serializable for Box<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        (**self).encode(s)
    }
}

impl< T: DeSerializable> DeSerializable for Box<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<Box<T>, D::Error> {
        Ok(Box::new(DeSerializable::decode(d)?))
    }
}

impl< T: DeSerializable> DeSerializable for Box<[T]> {
    fn decode<D: Decoder>(d: &mut D) -> Result<Box<[T]>, D::Error> {
        let v: Vec<T> = DeSerializable::decode(d)?;
        Ok(v.into_boxed_slice())
    }
}

impl<T:Serializable> Serializable for Rc<T> {
    #[inline]
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        (**self).encode(s)
    }
}

impl<T:DeSerializable> DeSerializable for Rc<T> {
    #[inline]
    fn decode<D: Decoder>(d: &mut D) -> Result<Rc<T>, D::Error> {
        Ok(Rc::new(DeSerializable::decode(d)?))
    }
}

impl<T:Serializable> Serializable for [T] {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_seq(self.len(), |s| {
            for (i, e) in self.iter().enumerate() {
                s.emit_seq_elt(i, |s| e.encode(s))?
            }
            Ok(())
        })
    }
}

impl<T:Serializable> Serializable for Vec<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_seq(self.len(), |s| {
            for (i, e) in self.iter().enumerate() {
                s.emit_seq_elt(i, |s| e.encode(s))?
            }
            Ok(())
        })
    }
}

impl<T:DeSerializable> DeSerializable for Vec<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<Vec<T>, D::Error> {
        d.read_seq(|d, len| {
            let mut v = Vec::with_capacity(len);
            for i in 0..len {
                v.push(d.read_seq_elt(i, |d| DeSerializable::decode(d))?);
            }
            Ok(v)
        })
    }
}

impl<'a, T:Serializable> Serializable for Cow<'a, [T]>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_seq(self.len(), |s| {
            for (i, e) in self.iter().enumerate() {
                s.emit_seq_elt(i, |s| e.encode(s))?
            }
            Ok(())
        })
    }
}

impl<T:DeSerializable+ToOwned> DeSerializable for Cow<'static, [T]>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn decode<D: Decoder>(d: &mut D) -> Result<Cow<'static, [T]>, D::Error> {
        d.read_seq(|d, len| {
            let mut v = Vec::with_capacity(len);
            for i in 0..len {
                v.push(d.read_seq_elt(i, |d| DeSerializable::decode(d))?);
            }
            Ok(Cow::Owned(v))
        })
    }
}

impl<T:Serializable> Serializable for Option<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_option(|s| {
            match *self {
                None => s.emit_option_none(),
                Some(ref v) => s.emit_option_some(|s| v.encode(s)),
            }
        })
    }
}

impl<T:DeSerializable> DeSerializable for Option<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<Option<T>, D::Error> {
        d.read_option(|d, b| {
            if b {
                Ok(Some(DeSerializable::decode(d)?))
            } else {
                Ok(None)
            }
        })
    }
}

macro_rules! peel {
    ($name:ident, $($other:ident,)*) => (tuple! { $($other,)* })
}

/// Evaluates to the number of identifiers passed to it, for example: `count_idents!(a, b, c) == 3
macro_rules! count_idents {
    () => { 0 };
    ($_i:ident, $($rest:ident,)*) => { 1 + count_idents!($($rest,)*) }
}

macro_rules! tuple {
    () => ();
    ( $($name:ident,)+ ) => (
        impl<$($name:DeSerializable),*> DeSerializable for ($($name,)*) {
            #[allow(non_snake_case)]
            fn decode<D: Decoder>(d: &mut D) -> Result<($($name,)*), D::Error> {
                let len: usize = count_idents!($($name,)*);
                d.read_tuple(len, |d| {
                    let mut i = 0;
                    let ret = ($(d.read_tuple_arg({ i+=1; i-1 },
                                                  |d| -> Result<$name,D::Error> {
                        DeSerializable::decode(d)
                    })?,)*);
                    Ok(ret)
                })
            }
        }
        impl<$($name:Serializable),*> Serializable for ($($name,)*) {
            #[allow(non_snake_case)]
            fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
                let ($(ref $name,)*) = *self;
                let mut n = 0;
                $(let $name = $name; n += 1;)*
                s.emit_tuple(n, |s| {
                    let mut i = 0;
                    $(s.emit_tuple_arg({ i+=1; i-1 }, |s| $name.encode(s))?;)*
                    Ok(())
                })
            }
        }
        peel! { $($name,)* }
    )
}

tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, }


macro_rules! array {
    () => ();
    ($len:expr, $($idx:expr,)*) => {
        impl<T:DeSerializable> DeSerializable for [T; $len] {
            fn decode<D: Decoder>(d: &mut D) -> Result<[T; $len], D::Error> {
                d.read_seq(|d, len| {
                    if len != $len {
                        return Err(d.error("wrong array length"));
                    }
                    Ok([$(
                        d.read_seq_elt($len - $idx - 1,
                                            |d| DeSerializable::decode(d))?
                    ),*])
                })
            }
        }

        impl<T:Serializable> Serializable for [T; $len] {
            fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
                s.emit_seq($len, |s| {
                    for i in 0..$len {
                        s.emit_seq_elt(i, |s| self[i].encode(s))?;
                    }
                    Ok(())
                })
            }
        }
        array! { $($idx,)* }
    }
}

array! {
    32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16,
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
}

impl Serializable for path::PathBuf {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        self.to_str().unwrap().encode(e)
    }
}

impl DeSerializable for path::PathBuf {
    fn decode<D: Decoder>(d: &mut D) -> Result<path::PathBuf, D::Error> {
        let bytes: String = DeSerializable::decode(d)?;
        Ok(path::PathBuf::from(bytes))
    }
}

impl<T: Serializable + Copy> Serializable for Cell<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        self.get().encode(s)
    }
}

impl<T: DeSerializable + Copy> DeSerializable for Cell<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<Cell<T>, D::Error> {
        Ok(Cell::new(DeSerializable::decode(d)?))
    }
}

// FIXME: #15036
// Should use `try_borrow`, returning a
// `encoder.error("attempting to Encode borrowed RefCell")`
// from `encode` when `try_borrow` returns `None`.

impl<T: Serializable> Serializable for RefCell<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        self.borrow().encode(s)
    }
}

impl<T: DeSerializable> DeSerializable for RefCell<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<RefCell<T>, D::Error> {
        Ok(RefCell::new(DeSerializable::decode(d)?))
    }
}

impl<T:Serializable> Serializable for Arc<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        (**self).encode(s)
    }
}

impl<T:DeSerializable+Send+Sync> DeSerializable for Arc<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<Arc<T>, D::Error> {
        Ok(Arc::new(DeSerializable::decode(d)?))
    }
}

use std::hash::Hash;
use std::collections::{LinkedList, VecDeque, BTreeMap, BTreeSet, HashMap, HashSet};

// Limit collections from allocating more than
// 1 MB for calls to `with_capacity`.
fn cap_capacity<T>(given_len: usize) -> usize {
    use std::cmp::min;
    use std::mem::size_of;
    const PRE_ALLOCATE_CAP: usize = 0x100000;

    match size_of::<T>() {
        0 => min(given_len, PRE_ALLOCATE_CAP),
        n => min(given_len, PRE_ALLOCATE_CAP / n)
    }
}

impl<T: Serializable> Serializable for LinkedList<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_seq(self.len(), |s| {
            for (i, e) in self.iter().enumerate() {
                s.emit_seq_elt(i, |s| e.encode(s))?;
            }
            Ok(())
        })
    }
}

impl<T:DeSerializable> DeSerializable for LinkedList<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<LinkedList<T>, D::Error> {
        d.read_seq(|d, len| {
            let mut list = LinkedList::new();
            for i in 0..len {
                list.push_back(d.read_seq_elt(i, |d| DeSerializable::decode(d))?);
            }
            Ok(list)
        })
    }
}

impl<T: Serializable> Serializable for VecDeque<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_seq(self.len(), |s| {
            for (i, e) in self.iter().enumerate() {
                s.emit_seq_elt(i, |s| e.encode(s))?;
            }
            Ok(())
        })
    }
}

impl<T:DeSerializable> DeSerializable for VecDeque<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<VecDeque<T>, D::Error> {
        d.read_seq(|d, len| {
            let mut deque: VecDeque<T> = VecDeque::new();
            for i in 0..len {
                deque.push_back(d.read_seq_elt(i, |d| DeSerializable::decode(d))?);
            }
            Ok(deque)
        })
    }
}

impl<K: Serializable + Ord, V: Serializable> Serializable for BTreeMap<K, V> {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        e.emit_map(self.len(), |e| {
            let mut i = 0;
            for (key, val) in self.iter() {
                e.emit_map_elt_key(i, |e| key.encode(e))?;
                e.emit_map_elt_val(i, |e| val.encode(e))?;
                i += 1;
            }
            Ok(())
        })
    }
}

impl<K: DeSerializable + Ord, V: DeSerializable> DeSerializable for BTreeMap<K, V> {
    fn decode<D: Decoder>(d: &mut D) -> Result<BTreeMap<K, V>, D::Error> {
        d.read_map(|d, len| {
            let mut map = BTreeMap::new();
            for i in 0..len {
                let key = d.read_map_elt_key(i, |d| DeSerializable::decode(d))?;
                let val = d.read_map_elt_val(i, |d| DeSerializable::decode(d))?;
                map.insert(key, val);
            }
            Ok(map)
        })
    }
}

impl<T: Serializable + Ord> Serializable for BTreeSet<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_seq(self.len(), |s| {
            let mut i = 0;
            for e in self.iter() {
                s.emit_seq_elt(i, |s| e.encode(s))?;
                i += 1;
            }
            Ok(())
        })
    }
}

impl<T: DeSerializable + Ord> DeSerializable for BTreeSet<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<BTreeSet<T>, D::Error> {
        d.read_seq(|d, len| {
            let mut set = BTreeSet::new();
            for i in 0..len {
                set.insert(d.read_seq_elt(i, |d| DeSerializable::decode(d))?);
            }
            Ok(set)
        })
    }
}

impl<K, V> Serializable for HashMap<K, V>
where
    K: Serializable + Hash + Eq,
    V: Serializable,
{
    fn encode<E: Encoder>(&self, e: &mut E) -> Result<(), E::Error> {
        e.emit_map(self.len(), |e| {
            let mut i = 0;
            for (key, val) in self.iter() {
                e.emit_map_elt_key(i, |e| key.encode(e))?;
                e.emit_map_elt_val(i, |e| val.encode(e))?;
                i += 1;
            }
            Ok(())
        })
    }
}

impl<K, V> DeSerializable for HashMap<K, V>
where
    K: DeSerializable + Hash + Eq,
    V: DeSerializable,
{
    fn decode<D: Decoder>(d: &mut D) -> Result<HashMap<K, V>, D::Error> {
        d.read_map(|d, len| {
            let mut map = HashMap::with_capacity(cap_capacity::<(K, V)>(len));
            for i in 0..len {
                let key = d.read_map_elt_key(i, |d| DeSerializable::decode(d))?;
                let val = d.read_map_elt_val(i, |d| DeSerializable::decode(d))?;
                map.insert(key, val);
            }
            Ok(map)
        })
    }
}

impl<T> Serializable for HashSet<T>
where
    T: Serializable + Hash + Eq,
{
    fn encode<E: Encoder>(&self, s: &mut E) -> Result<(), E::Error> {
        s.emit_seq(self.len(), |s| {
            let mut i = 0;
            for e in self.iter() {
                s.emit_seq_elt(i, |s| e.encode(s))?;
                i += 1;
            }
            Ok(())
        })
    }
}

impl<T> DeSerializable for HashSet<T>
where
    T: DeSerializable + Hash + Eq,
{
    fn decode<D: Decoder>(d: &mut D) -> Result<HashSet<T>, D::Error> {
        d.read_seq(|d, len| {
            let mut set = HashSet::with_capacity(cap_capacity::<T>(len));
            for i in 0..len {
                set.insert(d.read_seq_elt(i, |d| DeSerializable::decode(d))?);
            }
            Ok(set)
        })
    }
}

use std::io::Cursor;
use std::marker::PhantomData;
use crate::opaque::Encoder as DataEncoder;
use crate::opaque::Decoder as DataDecoder;

///  SerializeHelper make it easy to obtain serialize function.
pub struct SerializeHelper {
    cursor: RefCell<Cursor<Vec<u8>>>,
}

impl SerializeHelper {
    /// Create a new instance of SerializeHelper
    ///
    /// ```
    /// let helper = SerializeHelper::new();
    /// ```
    ///
    pub fn new() -> SerializeHelper {
        SerializeHelper {
            cursor: RefCell::new(Cursor::new(Vec::new())),
        }
    }

    /// Get the size of the serialized buffer of the target.
    pub fn get_size(&self) -> usize {
        self.cursor.borrow().get_ref().len()
    }
}

impl SerializeHelper {
    /// Use encode to serialize a target type. The target must impl the tarit Serializable.
    /// The function return a Option::Some of `Vec<u8>`, if something error, return Option::None.
    ///
    /// ```
    /// #[derive(Serializable, DeSerializable)]
    /// struct TestSturct {
    ///     a1: u32,
    ///     a2: u32,
    /// }
    /// let a = TestEnum::EnumStruct {a1: 2017, a2:829};
    /// let helper = SerializeHelper::new();
    /// let data = helper.encode(a).unwrap();
    /// ```
    ///
    pub fn encode< E: Serializable >(&self, target: E) -> Option<Vec<u8>> {
        {
            let mut cursor = self.cursor.borrow_mut();
            let mut encoder = DataEncoder::new(&mut cursor);
            match target.encode(&mut encoder) {
                Result::Err(_) => return Option::None,
                _ => {},
            }
        }
        let data = self.cursor.borrow().clone().into_inner();
        Option::Some(data)
    }
}

///  DeSerializeHelper make it easy to obtain deserialize function.
pub struct DeSerializeHelper<'a, T:'a + ?Sized> {
    data: Vec<u8>,
    marker: PhantomData<&'a T>,
}

impl<'a, T: 'a + ?Sized + DeSerializable> DeSerializeHelper<'a, T> {
    /// Create a new instance of DeSerializeHelper, parameter data must be a variable of
    /// `Vec<u8>` which return by SerializeHelper::encode.
    ///
    /// ```
    /// #[derive(Serializable, DeSerializable)]
    /// struct TestSturct {
    ///     a1: u32,
    ///     a2: u32,
    /// }
    /// let a = TestEnum::EnumStruct {a1: 2017, a2:829};
    /// let helper = SerializeHelper::new();
    /// let data = helper.encode(a).unwrap();
    /// let helper = DeSerializeHelper::<TestEnum>::new(data);
    /// ```
    ///
    pub fn new(data: Vec<u8>) -> DeSerializeHelper<'a, T> {
        DeSerializeHelper {
            data: data,
            marker: PhantomData,
        }
    }

    /// Use decode to deserialize self data, and return the type T of SerializeHelper::encode.
    ///
    /// ```
    /// #[derive(Serializable, DeSerializable)]
    /// struct TestSturct {
    ///     a1: u32,
    ///     a2: u32,
    /// }
    /// let a = TestEnum::EnumStruct {a1: 2017, a2:829};
    /// let helper = SerializeHelper::new();
    /// let data = helper.encode(a).unwrap();
    /// let helper = DeSerializeHelper::<TestEnum>::new(data);
    /// let c = helper.decode().unwrap();
    /// ```
    ///
    pub fn decode(&self) -> Option<T> {
        let mut decoder = DataDecoder::new(&self.data[..], 0);
        match DeSerializable::decode(&mut decoder) {
            Result::Err(_) => Option::None,
            Result::Ok(d) => Option::Some(d),
        }
    }
}
