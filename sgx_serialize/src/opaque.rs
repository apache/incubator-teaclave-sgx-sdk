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
// under the License.

//!
//! The mod opaque Encoder and Decoder container to save buffer of target types
//!

#![allow(clippy::diverging_sub_expression)]

use crate::leb128;
use crate::serialize;
use std::borrow::Cow;
use std::mem::MaybeUninit;
use std::string::{String, ToString};
use std::vec::Vec;

// -----------------------------------------------------------------------------
// Encoder
// -----------------------------------------------------------------------------

pub type EncodeResult = Result<(), !>;

pub struct Encoder {
    pub data: Vec<u8>,
}

impl Encoder {
    pub fn new(data: Vec<u8>) -> Encoder {
        Encoder { data }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    #[inline]
    pub fn position(&self) -> usize {
        self.data.len()
    }
}

macro_rules! write_leb128 {
    ($enc:expr, $value:expr, $int_ty:ty, $fun:ident) => {{
        const MAX_ENCODED_LEN: usize = max_leb128_len!($int_ty);
        let old_len = $enc.data.len();

        if MAX_ENCODED_LEN > $enc.data.capacity() - old_len {
            $enc.data.reserve(MAX_ENCODED_LEN);
        }

        // SAFETY: The above check and `reserve` ensures that there is enough
        // room to write the encoded value to the vector's internal buffer.
        unsafe {
            let buf = &mut *($enc.data.as_mut_ptr().add(old_len)
                as *mut [MaybeUninit<u8>; MAX_ENCODED_LEN]);
            let encoded = leb128::$fun(buf, $value);
            $enc.data.set_len(old_len + encoded.len());
        }

        Ok(())
    }};
}

impl serialize::Encoder for Encoder {
    type Error = !;

    #[inline]
    fn emit_unit(&mut self) -> EncodeResult {
        Ok(())
    }

    #[inline]
    fn emit_usize(&mut self, v: usize) -> EncodeResult {
        write_leb128!(self, v, usize, write_usize_leb128)
    }

    #[inline]
    fn emit_u128(&mut self, v: u128) -> EncodeResult {
        write_leb128!(self, v, u128, write_u128_leb128)
    }

    #[inline]
    fn emit_u64(&mut self, v: u64) -> EncodeResult {
        write_leb128!(self, v, u64, write_u64_leb128)
    }

    #[inline]
    fn emit_u32(&mut self, v: u32) -> EncodeResult {
        write_leb128!(self, v, u32, write_u32_leb128)
    }

    #[inline]
    fn emit_u16(&mut self, v: u16) -> EncodeResult {
        write_leb128!(self, v, u16, write_u16_leb128)
    }

    #[inline]
    fn emit_u8(&mut self, v: u8) -> EncodeResult {
        self.data.push(v);
        Ok(())
    }

    #[inline]
    fn emit_isize(&mut self, v: isize) -> EncodeResult {
        write_leb128!(self, v, isize, write_isize_leb128)
    }

    #[inline]
    fn emit_i128(&mut self, v: i128) -> EncodeResult {
        write_leb128!(self, v, i128, write_i128_leb128)
    }

    #[inline]
    fn emit_i64(&mut self, v: i64) -> EncodeResult {
        write_leb128!(self, v, i64, write_i64_leb128)
    }

    #[inline]
    fn emit_i32(&mut self, v: i32) -> EncodeResult {
        write_leb128!(self, v, i32, write_i32_leb128)
    }

    #[inline]
    fn emit_i16(&mut self, v: i16) -> EncodeResult {
        write_leb128!(self, v, i16, write_i16_leb128)
    }

    #[inline]
    fn emit_i8(&mut self, v: i8) -> EncodeResult {
        let as_u8: u8 = unsafe { std::mem::transmute(v) };
        self.emit_u8(as_u8)
    }

    #[inline]
    fn emit_bool(&mut self, v: bool) -> EncodeResult {
        self.emit_u8(u8::from(v))
    }

    #[inline]
    fn emit_f64(&mut self, v: f64) -> EncodeResult {
        let as_u64: u64 = v.to_bits();
        self.emit_u64(as_u64)
    }

    #[inline]
    fn emit_f32(&mut self, v: f32) -> EncodeResult {
        let as_u32: u32 = v.to_bits();
        self.emit_u32(as_u32)
    }

    #[inline]
    fn emit_char(&mut self, v: char) -> EncodeResult {
        self.emit_u32(v as u32)
    }

    #[inline]
    fn emit_str(&mut self, v: &str) -> EncodeResult {
        self.emit_usize(v.len())?;
        self.emit_raw_bytes(v.as_bytes())
    }

    #[inline]
    fn emit_raw_bytes(&mut self, s: &[u8]) -> EncodeResult {
        self.data.extend_from_slice(s);
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Decoder
// -----------------------------------------------------------------------------

pub struct Decoder<'a> {
    pub data: &'a [u8],
    position: usize,
}

impl<'a> Decoder<'a> {
    #[inline]
    pub fn new(data: &'a [u8], position: usize) -> Decoder<'a> {
        Decoder { data, position }
    }

    #[inline]
    pub fn position(&self) -> usize {
        self.position
    }

    #[inline]
    pub fn set_position(&mut self, pos: usize) {
        self.position = pos
    }

    #[inline]
    pub fn advance(&mut self, bytes: usize) {
        self.position += bytes;
    }

    #[inline]
    pub fn read_raw_bytes(&mut self, bytes: usize) -> &'a [u8] {
        let start = self.position;
        self.position += bytes;
        &self.data[start..self.position]
    }
}

macro_rules! read_leb128 {
    ($dec:expr, $fun:ident) => {{
        let (value, bytes_read) = leb128::$fun(&$dec.data[$dec.position..]);
        $dec.position += bytes_read;
        Ok(value)
    }};
}

impl<'a> serialize::Decoder for Decoder<'a> {
    type Error = String;

    #[inline]
    fn read_nil(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn read_u128(&mut self) -> Result<u128, Self::Error> {
        read_leb128!(self, read_u128_leb128)
    }

    #[inline]
    fn read_u64(&mut self) -> Result<u64, Self::Error> {
        read_leb128!(self, read_u64_leb128)
    }

    #[inline]
    fn read_u32(&mut self) -> Result<u32, Self::Error> {
        read_leb128!(self, read_u32_leb128)
    }

    #[inline]
    fn read_u16(&mut self) -> Result<u16, Self::Error> {
        read_leb128!(self, read_u16_leb128)
    }

    #[inline]
    fn read_u8(&mut self) -> Result<u8, Self::Error> {
        let value = self.data[self.position];
        self.position += 1;
        Ok(value)
    }

    #[inline]
    fn read_usize(&mut self) -> Result<usize, Self::Error> {
        read_leb128!(self, read_usize_leb128)
    }

    #[inline]
    fn read_i128(&mut self) -> Result<i128, Self::Error> {
        read_leb128!(self, read_i128_leb128)
    }

    #[inline]
    fn read_i64(&mut self) -> Result<i64, Self::Error> {
        read_leb128!(self, read_i64_leb128)
    }

    #[inline]
    fn read_i32(&mut self) -> Result<i32, Self::Error> {
        read_leb128!(self, read_i32_leb128)
    }

    #[inline]
    fn read_i16(&mut self) -> Result<i16, Self::Error> {
        read_leb128!(self, read_i16_leb128)
    }

    #[inline]
    fn read_i8(&mut self) -> Result<i8, Self::Error> {
        let as_u8 = self.data[self.position];
        self.position += 1;
        unsafe { Ok(::std::mem::transmute(as_u8)) }
    }

    #[inline]
    fn read_isize(&mut self) -> Result<isize, Self::Error> {
        read_leb128!(self, read_isize_leb128)
    }

    #[inline]
    fn read_bool(&mut self) -> Result<bool, Self::Error> {
        let value = self.read_u8()?;
        Ok(value != 0)
    }

    #[inline]
    fn read_f64(&mut self) -> Result<f64, Self::Error> {
        let bits = self.read_u64()?;
        Ok(f64::from_bits(bits))
    }

    #[inline]
    fn read_f32(&mut self) -> Result<f32, Self::Error> {
        let bits = self.read_u32()?;
        Ok(f32::from_bits(bits))
    }

    #[inline]
    fn read_char(&mut self) -> Result<char, Self::Error> {
        let bits = self.read_u32()?;
        Ok(std::char::from_u32(bits).unwrap())
    }

    #[inline]
    fn read_str(&mut self) -> Result<Cow<'_, str>, Self::Error> {
        let len = self.read_usize()?;
        let s = std::str::from_utf8(&self.data[self.position..self.position + len]).unwrap();
        self.position += len;
        Ok(Cow::Borrowed(s))
    }

    #[inline]
    fn error(&mut self, err: &str) -> Self::Error {
        err.to_string()
    }

    #[inline]
    fn read_raw_bytes_into(&mut self, s: &mut [u8]) -> Result<(), String> {
        let start = self.position;
        self.position += s.len();
        s.copy_from_slice(&self.data[start..self.position]);
        Ok(())
    }
}

pub fn encode<T: serialize::Encodable>(data: &T) -> Option<Vec<u8>> {
    let mut encoder = Encoder::new(Vec::new());
    data.encode(&mut encoder).ok()?;
    Some(encoder.into_inner())
}

pub fn decode<T: serialize::Decodable>(data: &[u8]) -> Option<T> {
    let mut decoder = Decoder::new(data, 0);
    serialize::Decodable::decode(&mut decoder).ok()
}
