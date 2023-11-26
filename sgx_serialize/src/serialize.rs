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

// Support code for encoding and decoding types.
/*
Core encoding and decoding interfaces.
*/
use std::borrow::Cow;
use std::borrow::ToOwned;
use std::boxed::Box;
use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::path;
use std::rc::Rc;
use std::string::String;
use std::sync::Arc;
use std::vec::Vec;

pub trait Encoder {
    type Error;

    // Primitive types:
    fn emit_unit(&mut self) -> Result<(), Self::Error>;
    fn emit_usize(&mut self, v: usize) -> Result<(), Self::Error>;
    fn emit_u128(&mut self, v: u128) -> Result<(), Self::Error>;
    fn emit_u64(&mut self, v: u64) -> Result<(), Self::Error>;
    fn emit_u32(&mut self, v: u32) -> Result<(), Self::Error>;
    fn emit_u16(&mut self, v: u16) -> Result<(), Self::Error>;
    fn emit_u8(&mut self, v: u8) -> Result<(), Self::Error>;
    fn emit_isize(&mut self, v: isize) -> Result<(), Self::Error>;
    fn emit_i128(&mut self, v: i128) -> Result<(), Self::Error>;
    fn emit_i64(&mut self, v: i64) -> Result<(), Self::Error>;
    fn emit_i32(&mut self, v: i32) -> Result<(), Self::Error>;
    fn emit_i16(&mut self, v: i16) -> Result<(), Self::Error>;
    fn emit_i8(&mut self, v: i8) -> Result<(), Self::Error>;
    fn emit_bool(&mut self, v: bool) -> Result<(), Self::Error>;
    fn emit_f64(&mut self, v: f64) -> Result<(), Self::Error>;
    fn emit_f32(&mut self, v: f32) -> Result<(), Self::Error>;
    fn emit_char(&mut self, v: char) -> Result<(), Self::Error>;
    fn emit_str(&mut self, v: &str) -> Result<(), Self::Error>;
    fn emit_raw_bytes(&mut self, s: &[u8]) -> Result<(), Self::Error>;

    // Compound types:
    #[inline]
    fn emit_enum<F>(&mut self, _name: &str, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        f(self)
    }

    fn emit_enum_variant<F>(
        &mut self,
        _v_name: &str,
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

    #[inline]
    fn emit_enum_variant_arg<F>(&mut self, _idx: usize, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        f(self)
    }

    #[inline]
    fn emit_struct<F>(&mut self, _name: &str, _len: usize, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        f(self)
    }

    #[inline]
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

    #[inline]
    fn emit_tuple<F>(&mut self, _len: usize, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        f(self)
    }

    #[inline]
    fn emit_tuple_arg<F>(&mut self, _idx: usize, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        f(self)
    }

    // Specialized types:
    fn emit_option<F>(&mut self, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        self.emit_enum("Option", f)
    }

    #[inline]
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

    #[inline]
    fn emit_seq_elt<F>(&mut self, _idx: usize, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        f(self)
    }

    fn emit_map<F>(&mut self, len: usize, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        self.emit_usize(len)?;
        f(self)
    }

    #[inline]
    fn emit_map_elt_key<F>(&mut self, _idx: usize, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Self::Error>,
    {
        f(self)
    }

    #[inline]
    fn emit_map_elt_val<F>(&mut self, f: F) -> Result<(), Self::Error>
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
    fn read_str(&mut self) -> Result<Cow<'_, str>, Self::Error>;
    fn read_raw_bytes_into(&mut self, s: &mut [u8]) -> Result<(), Self::Error>;

    // Compound types:
    #[inline]
    fn read_enum<T, F>(&mut self, _name: &str, f: F) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Self::Error>,
    {
        f(self)
    }

    #[inline]
    fn read_enum_variant<T, F>(&mut self, _names: &[&str], mut f: F) -> Result<T, Self::Error>
    where
        F: FnMut(&mut Self, usize) -> Result<T, Self::Error>,
    {
        let disr = self.read_usize()?;
        f(self, disr)
    }

    #[inline]
    fn read_enum_variant_arg<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Self::Error>,
    {
        f(self)
    }

    #[inline]
    fn read_struct<T, F>(&mut self, _s_name: &str, _len: usize, f: F) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Self::Error>,
    {
        f(self)
    }

    #[inline]
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

    #[inline]
    fn read_tuple<T, F>(&mut self, _len: usize, f: F) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Self::Error>,
    {
        f(self)
    }

    #[inline]
    fn read_tuple_arg<T, F>(&mut self, f: F) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Self::Error>,
    {
        f(self)
    }

    // Specialized types:
    fn read_option<T, F>(&mut self, mut f: F) -> Result<T, Self::Error>
    where
        F: FnMut(&mut Self, bool) -> Result<T, Self::Error>,
    {
        self.read_enum("Option", move |this| {
            this.read_enum_variant(&["None", "Some"], move |this, idx| match idx {
                0 => f(this, false),
                1 => f(this, true),
                _ => Err(this.error("read_option: expected 0 for None or 1 for Some")),
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

    #[inline]
    fn read_seq_elt<T, F>(&mut self, f: F) -> Result<T, Self::Error>
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

    #[inline]
    fn read_map_elt_key<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Self::Error>,
    {
        f(self)
    }

    #[inline]
    fn read_map_elt_val<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Self::Error>,
    {
        f(self)
    }

    // Failure
    fn error(&mut self, err: &str) -> Self::Error;
}

/// Trait for types that can be serialized
///
/// This can be implemented using the `Encodable`, `TyEncodable` and
/// `MetadataEncodable` macros.
///
/// * `Encodable` should be used in crates that don't depend on
///   `rustc_middle`.
/// * `MetadataEncodable` is used in `rustc_metadata` for types that contain
///   `rustc_metadata::rmeta::Lazy`.
/// * `TyEncodable` should be used for types that are only serialized in crate
///   metadata or the incremental cache. This is most types in `rustc_middle`.
pub trait Encodable {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error>;
}

/// Trait for types that can be deserialized
///
/// This can be implemented using the `Decodable`, `TyDecodable` and
/// `MetadataDecodable` macros.
///
/// * `Decodable` should be used in crates that don't depend on
///   `rustc_middle`.
/// * `MetadataDecodable` is used in `rustc_metadata` for types that contain
///   `rustc_metadata::rmeta::Lazy`.
/// * `TyDecodable` should be used for types that are only serialized in crate
///   metadata or the incremental cache. This is most types in `rustc_middle`.
pub trait Decodable: Sized {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error>;
}

macro_rules! direct_serialize_impls {
    ($($ty:ident $emit_method:ident $read_method:ident),*) => {
        $(
            impl Encodable for $ty {
                fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
                    s.$emit_method(*self)
                }
            }

            impl Decodable for $ty {
                fn decode<D: Decoder>(d: &mut D) -> Result<$ty, D::Error> {
                    d.$read_method()
                }
            }
        )*
    }
}

direct_serialize_impls! {
    usize emit_usize read_usize,
    u8 emit_u8 read_u8,
    u16 emit_u16 read_u16,
    u32 emit_u32 read_u32,
    u64 emit_u64 read_u64,
    u128 emit_u128 read_u128,
    isize emit_isize read_isize,
    i8 emit_i8 read_i8,
    i16 emit_i16 read_i16,
    i32 emit_i32 read_i32,
    i64 emit_i64 read_i64,
    i128 emit_i128 read_i128,
    f32 emit_f32 read_f32,
    f64 emit_f64 read_f64,
    bool emit_bool read_bool,
    char emit_char read_char
}

impl Encodable for ! {
    fn encode<S: Encoder>(&self, _s: &mut S) -> Result<(), S::Error> {
        unreachable!()
    }
}

impl Decodable for ! {
    fn decode<D: Decoder>(_d: &mut D) -> Result<!, D::Error> {
        unreachable!()
    }
}

impl Encodable for ::std::num::NonZeroU32 {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_u32(self.get())
    }
}

impl Decodable for ::std::num::NonZeroU32 {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_u32()
            .map(|d| ::std::num::NonZeroU32::new(d).unwrap())
    }
}

impl Encodable for str {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_str(self)
    }
}

impl Encodable for &str {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_str(self)
    }
}

impl Encodable for String {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_str(&self[..])
    }
}

impl Decodable for String {
    fn decode<D: Decoder>(d: &mut D) -> Result<String, D::Error> {
        Ok(d.read_str()?.into_owned())
    }
}

impl Encodable for () {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_unit()
    }
}

impl Decodable for () {
    fn decode<D: Decoder>(d: &mut D) -> Result<(), D::Error> {
        d.read_nil()
    }
}

impl<T: ?Sized> Encodable for PhantomData<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_unit()
    }
}

impl<T: ?Sized> Decodable for PhantomData<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<PhantomData<T>, D::Error> {
        d.read_nil()?;
        Ok(PhantomData)
    }
}

impl<T: Decodable> Decodable for Box<[T]> {
    fn decode<D: Decoder>(d: &mut D) -> Result<Box<[T]>, D::Error> {
        let v: Vec<T> = Decodable::decode(d)?;
        Ok(v.into_boxed_slice())
    }
}

impl<T: Encodable> Encodable for Rc<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        (**self).encode(s)
    }
}

impl<T: Decodable> Decodable for Rc<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<Rc<T>, D::Error> {
        Ok(Rc::new(Decodable::decode(d)?))
    }
}

impl<T: Encodable> Encodable for [T] {
    default fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_seq(self.len(), |s| {
            for (i, e) in self.iter().enumerate() {
                s.emit_seq_elt(i, |s| e.encode(s))?
            }
            Ok(())
        })
    }
}

impl<T: Encodable> Encodable for Vec<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        let slice: &[T] = self;
        slice.encode(s)
    }
}

impl<T: Decodable> Decodable for Vec<T> {
    default fn decode<D: Decoder>(d: &mut D) -> Result<Vec<T>, D::Error> {
        d.read_seq(|d, len| {
            let mut v = Vec::with_capacity(len);
            for _ in 0..len {
                v.push(d.read_seq_elt(|d| Decodable::decode(d))?);
            }
            Ok(v)
        })
    }
}

impl<T: Encodable, const N: usize> Encodable for [T; N] {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        let slice: &[T] = self;
        slice.encode(s)
    }
}

impl<T: Decodable, const N: usize> Decodable for [T; N] {
    fn decode<D: Decoder>(d: &mut D) -> Result<[T; N], D::Error> {
        d.read_seq(|d, len| {
            assert!(len == N);
            let mut v: [T; N] = unsafe { std::mem::zeroed() };
            for item in &mut v.iter_mut().take(len) {
                *item = d.read_seq_elt(|d| Decodable::decode(d))?;
            }
            Ok(v)
        })
    }
}

impl<'a, T: Encodable> Encodable for Cow<'a, [T]>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        let slice: &[T] = self;
        slice.encode(s)
    }
}

impl<T: Decodable + ToOwned> Decodable for Cow<'static, [T]>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn decode<D: Decoder>(d: &mut D) -> Result<Cow<'static, [T]>, D::Error> {
        let v: Vec<T> = Decodable::decode(d)?;
        Ok(Cow::Owned(v))
    }
}

impl<T: Encodable> Encodable for Option<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_option(|s| match *self {
            None => s.emit_option_none(),
            Some(ref v) => s.emit_option_some(|s| v.encode(s)),
        })
    }
}

impl<T: Decodable> Decodable for Option<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<Option<T>, D::Error> {
        d.read_option(|d, b| {
            if b {
                Ok(Some(Decodable::decode(d)?))
            } else {
                Ok(None)
            }
        })
    }
}

impl<T1: Encodable, T2: Encodable> Encodable for Result<T1, T2> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_enum("Result", |s| match *self {
            Ok(ref v) => {
                s.emit_enum_variant("Ok", 0, 1, |s| s.emit_enum_variant_arg(0, |s| v.encode(s)))
            }
            Err(ref v) => {
                s.emit_enum_variant("Err", 1, 1, |s| s.emit_enum_variant_arg(0, |s| v.encode(s)))
            }
        })
    }
}

impl<T1: Decodable, T2: Decodable> Decodable for Result<T1, T2> {
    fn decode<D: Decoder>(d: &mut D) -> Result<Result<T1, T2>, D::Error> {
        d.read_enum("Result", |d| {
            d.read_enum_variant(&["Ok", "Err"], |d, disr| match disr {
                0 => Ok(Ok(d.read_enum_variant_arg(0, |d| T1::decode(d))?)),
                1 => Ok(Err(d.read_enum_variant_arg(1, |d| T2::decode(d))?)),
                _ => {
                    panic!(
                        "Encountered invalid discriminant while \
                                decoding `Result`."
                    );
                }
            })
        })
    }
}

macro_rules! peel {
    ($name:ident, $($other:ident,)*) => (tuple! { $($other,)* })
}

/// Evaluates to the number of tokens passed to it.
///
/// Logarithmic counting: every one or two recursive expansions, the number of
/// tokens to count is divided by two, instead of being reduced by one.
/// Therefore, the recursion depth is the binary logarithm of the number of
/// tokens to count, and the expanded tree is likewise very small.
macro_rules! count {
    ()                     => (0usize);
    ($one:tt)              => (1usize);
    ($($pairs:tt $_p:tt)*) => (count!($($pairs)*) << 1usize);
    ($odd:tt $($rest:tt)*) => (count!($($rest)*) | 1usize);
}

macro_rules! tuple {
    () => ();
    ( $($name:ident,)+ ) => (
        impl<$($name: Decodable),+> Decodable for ($($name,)+) {
            #[allow(non_snake_case)]
            fn decode<D: Decoder>(d: &mut D) -> Result<($($name,)+), D::Error> {
                let len: usize = count!($($name)+);
                d.read_tuple(len, |d| {
                    let ret = ($(d.read_tuple_arg(|d| -> Result<$name, D::Error> {
                        Decodable::decode(d)
                    })?,)+);
                    Ok(ret)
                })
            }
        }
        impl<$($name: Encodable),+> Encodable for ($($name,)+) {
            #[allow(non_snake_case)]
            fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
                let ($(ref $name,)+) = *self;
                let mut n = 0;
                $(let $name = $name; n += 1;)+
                s.emit_tuple(n, |s| {
                    let mut i = 0;
                    $(s.emit_tuple_arg({ i += 1; i - 1 }, |s| $name.encode(s))?;)+
                    Ok(())
                })
            }
        }
        peel! { $($name,)+ }
    )
}

tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, }

impl Encodable for path::Path {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        self.to_str().unwrap().encode(e)
    }
}

impl Encodable for path::PathBuf {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        path::Path::encode(self, e)
    }
}

impl Decodable for path::PathBuf {
    fn decode<D: Decoder>(d: &mut D) -> Result<path::PathBuf, D::Error> {
        let bytes: String = Decodable::decode(d)?;
        Ok(path::PathBuf::from(bytes))
    }
}

impl<T: Encodable + Copy> Encodable for Cell<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        self.get().encode(s)
    }
}

impl<T: Decodable + Copy> Decodable for Cell<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<Cell<T>, D::Error> {
        Ok(Cell::new(Decodable::decode(d)?))
    }
}

// FIXME: #15036
// Should use `try_borrow`, returning a
// `encoder.error("attempting to Encode borrowed RefCell")`
// from `encode` when `try_borrow` returns `None`.

impl<T: Encodable> Encodable for RefCell<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        self.borrow().encode(s)
    }
}

impl<T: Decodable> Decodable for RefCell<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<RefCell<T>, D::Error> {
        Ok(RefCell::new(Decodable::decode(d)?))
    }
}

impl<T: Encodable> Encodable for Arc<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        (**self).encode(s)
    }
}

impl<T: Decodable> Decodable for Arc<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<Arc<T>, D::Error> {
        Ok(Arc::new(Decodable::decode(d)?))
    }
}

impl<T: ?Sized + Encodable> Encodable for Box<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        (**self).encode(s)
    }
}
impl<T: Decodable> Decodable for Box<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<Box<T>, D::Error> {
        Ok(Box::new(Decodable::decode(d)?))
    }
}
