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

//!
//! The mod opaque Encoder and Decoder container to save buffer of target types
//!

use crate::leb128::{read_signed_leb128, read_unsigned_leb128, write_signed_leb128, write_unsigned_leb128};
use std::vec::Vec;
use std::string::String;
use std::string::ToString;
use std::borrow::Cow;
use std::io::{self, Write};
use crate::serialize;

pub struct Encoder<'a> {
    pub cursor: &'a mut io::Cursor<Vec<u8>>,
}

impl<'a> Encoder<'a> {
    pub fn new(cursor: &'a mut io::Cursor<Vec<u8>>) -> Encoder<'a> {
        Encoder { cursor: cursor }
    }
}

macro_rules! write_uleb128 {
    ($enc:expr, $value:expr) => {{
        let pos = $enc.cursor.position() as usize;
        let bytes_written = write_unsigned_leb128($enc.cursor.get_mut(), pos, $value as u128);
        $enc.cursor.set_position((pos + bytes_written) as u64);
        Ok(())
    }}
}

macro_rules! write_sleb128 {
    ($enc:expr, $value:expr) => {{
        let pos = $enc.cursor.position() as usize;
        let bytes_written = write_signed_leb128($enc.cursor.get_mut(), pos, $value as i128);
        $enc.cursor.set_position((pos + bytes_written) as u64);
        Ok(())
    }}
}

impl<'a> serialize::Encoder for Encoder<'a> {
    type Error = ();

    fn emit_nil(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn emit_usize(&mut self, v: usize) -> Result<(), Self::Error> {
        write_uleb128!(self, v)
    }

    fn emit_u128(&mut self, v: u128) -> Result<(), Self::Error> {
        write_uleb128!(self, v)
    }

    fn emit_u64(&mut self, v: u64) -> Result<(), Self::Error> {
        write_uleb128!(self, v)
    }

    fn emit_u32(&mut self, v: u32) -> Result<(), Self::Error> {
        write_uleb128!(self, v)
    }

    fn emit_u16(&mut self, v: u16) -> Result<(), Self::Error> {
        write_uleb128!(self, v)
    }

    fn emit_u8(&mut self, v: u8) -> Result<(), Self::Error> {
        let _ = self.cursor.write_all(&[v]);
        Ok(())
    }

    fn emit_isize(&mut self, v: isize) -> Result<(), Self::Error> {
        write_sleb128!(self, v)
    }

    fn emit_i128(&mut self, v: i128) -> Result<(), Self::Error> {
        write_sleb128!(self, v)
    }

    fn emit_i64(&mut self, v: i64) -> Result<(), Self::Error> {
        write_sleb128!(self, v)
    }

    fn emit_i32(&mut self, v: i32) -> Result<(), Self::Error> {
        write_sleb128!(self, v)
    }

    fn emit_i16(&mut self, v: i16) -> Result<(), Self::Error> {
        write_sleb128!(self, v)
    }

    fn emit_i8(&mut self, v: i8) -> Result<(), Self::Error> {
        let as_u8: u8 = unsafe { ::std::mem::transmute(v) };
        let _ = self.cursor.write_all(&[as_u8]);
        Ok(())
    }

    fn emit_bool(&mut self, v: bool) -> Result<(), Self::Error> {
        self.emit_u8(if v {
            1
        } else {
            0
        })
    }

    fn emit_f64(&mut self, v: f64) -> Result<(), Self::Error> {
        let as_u64: u64 = unsafe { ::std::mem::transmute(v) };
        self.emit_u64(as_u64)
    }

    fn emit_f32(&mut self, v: f32) -> Result<(), Self::Error> {
        let as_u32: u32 = unsafe { ::std::mem::transmute(v) };
        self.emit_u32(as_u32)
    }

    fn emit_char(&mut self, v: char) -> Result<(), Self::Error> {
        self.emit_u32(v as u32)
    }

    fn emit_str(&mut self, v: &str) -> Result<(), Self::Error> {
        self.emit_usize(v.len())?;
        let _ = self.cursor.write_all(v.as_bytes());
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
    pub fn new(data: &'a [u8], position: usize) -> Decoder<'a> {
        Decoder {
            data: data,
            position: position,
        }
    }

    // pub fn position(&self) -> usize {
    //     self.position
    // }

    // pub fn advance(&mut self, bytes: usize) {
    //     self.position += bytes;
    // }
}

macro_rules! read_uleb128 {
    ($dec:expr, $t:ty) => ({
        let (value, bytes_read) = read_unsigned_leb128($dec.data, $dec.position);
        $dec.position += bytes_read;
        Ok(value as $t)
    })
}

macro_rules! read_sleb128 {
    ($dec:expr, $t:ty) => ({
        let (value, bytes_read) = read_signed_leb128($dec.data, $dec.position);
        $dec.position += bytes_read;
        Ok(value as $t)
    })
}

impl<'a> serialize::Decoder for Decoder<'a> {
    type Error = String;

    #[inline]
    fn read_nil(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn read_u128(&mut self) -> Result<u128, Self::Error> {
        read_uleb128!(self, u128)
    }

    #[inline]
    fn read_u64(&mut self) -> Result<u64, Self::Error> {
        read_uleb128!(self, u64)
    }

    #[inline]
    fn read_u32(&mut self) -> Result<u32, Self::Error> {
        read_uleb128!(self, u32)
    }

    #[inline]
    fn read_u16(&mut self) -> Result<u16, Self::Error> {
        read_uleb128!(self, u16)
    }

    #[inline]
    fn read_u8(&mut self) -> Result<u8, Self::Error> {
        let value = self.data[self.position];
        self.position += 1;
        Ok(value)
    }

    #[inline]
    fn read_usize(&mut self) -> Result<usize, Self::Error> {
        read_uleb128!(self, usize)
    }

    #[inline]
    fn read_i128(&mut self) -> Result<i128, Self::Error> {
        read_sleb128!(self, i128)
    }

    #[inline]
    fn read_i64(&mut self) -> Result<i64, Self::Error> {
        read_sleb128!(self, i64)
    }

    #[inline]
    fn read_i32(&mut self) -> Result<i32, Self::Error> {
        read_sleb128!(self, i32)
    }

    #[inline]
    fn read_i16(&mut self) -> Result<i16, Self::Error> {
        read_sleb128!(self, i16)
    }

    #[inline]
    fn read_i8(&mut self) -> Result<i8, Self::Error> {
        let as_u8 = self.data[self.position];
        self.position += 1;
        unsafe { Ok(::std::mem::transmute(as_u8)) }
    }

    #[inline]
    fn read_isize(&mut self) -> Result<isize, Self::Error> {
        read_sleb128!(self, isize)
    }

    #[inline]
    fn read_bool(&mut self) -> Result<bool, Self::Error> {
        let value = self.read_u8()?;
        Ok(value != 0)
    }

    #[inline]
    fn read_f64(&mut self) -> Result<f64, Self::Error> {
        let bits = self.read_u64()?;
        Ok(unsafe { ::std::mem::transmute(bits) })
    }

    #[inline]
    fn read_f32(&mut self) -> Result<f32, Self::Error> {
        let bits = self.read_u32()?;
        Ok(unsafe { ::std::mem::transmute(bits) })
    }

    #[inline]
    fn read_char(&mut self) -> Result<char, Self::Error> {
        let bits = self.read_u32()?;
        Ok(::std::char::from_u32(bits).unwrap())
    }

    #[inline]
    fn read_str(&mut self) -> Result<Cow<str>, Self::Error> {
        let len = self.read_usize()?;
        let s = ::std::str::from_utf8(&self.data[self.position..self.position + len]).unwrap();
        self.position += len;
        Ok(Cow::Borrowed(s))
    }

    fn error(&mut self, err: &str) -> Self::Error {
        err.to_string()
    }
}
