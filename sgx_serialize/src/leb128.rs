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

use std::vec::Vec;

#[inline]
fn write_to_vec(vec: &mut Vec<u8>, position: usize, byte: u8) {
    if position == vec.len() {
        vec.push(byte);
    } else {
        vec[position] = byte;
    }
}

/// Encodes an integer using unsigned leb128 encoding and stores
/// the result using a callback function.
///
/// The callback function `write` is called once on writing one byte out.
#[inline]
pub fn write_unsigned_leb128_to<W>(mut value: u128, mut write: W) -> usize
where
    W: FnMut(usize, u8),
{
    let mut position = 0;
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }

        write(position, byte);
        position += 1;

        if value == 0 {
            break;
        }
    }

    position
}

/// A wrapper function of `write_unsigned_leb128_to`.
/// `write_unsigned_leb128` uses a private function `write_to_vec` for emitting
/// bytes to given `Vec<u8>`.
pub fn write_unsigned_leb128(out: &mut Vec<u8>, start_position: usize, value: u128) -> usize {
    write_unsigned_leb128_to(value, |i, v| write_to_vec(out, start_position+i, v))
}

/// `read_unsigned_leb128` reads data from arg `data` at offset `start_position`
/// Returns the decoded `u128` value along with read size
#[inline]
pub fn read_unsigned_leb128(data: &[u8], start_position: usize) -> (u128, usize) {
    let mut result = 0;
    let mut shift = 0;
    let mut position = start_position;
    loop {
        let byte = data[position];
        position += 1;
        result |= ((byte & 0x7F) as u128) << shift;
        if (byte & 0x80) == 0 {
            break;
        }
        shift += 7;
    }

    (result, position - start_position)
}

/// Encodes an integer using signed leb128 encoding and stores
/// the result using a callback function.
///
/// The callback function `write` is called once on writing one byte out.
#[inline]
pub fn write_signed_leb128_to<W>(mut value: i128, mut write: W) -> usize
where
    W: FnMut(usize, u8),
{
    let mut position = 0;

    loop {
        let mut byte = (value as u8) & 0x7f;
        value >>= 7;
        let more = !((((value == 0) && ((byte & 0x40) == 0)) ||
                      ((value == -1) && ((byte & 0x40) != 0))));

        if more {
            byte |= 0x80; // Mark this byte to show that more bytes will follow.
        }

        write(position, byte);
        position += 1;

        if !more {
            break;
        }
    }
    position
}

/// A wrapper function of `write_signed_leb128_to`.
/// `write_unsigned_leb128` uses a private function `write_to_vec` for emitting
/// bytes to given `Vec<u8>`.
pub fn write_signed_leb128(out: &mut Vec<u8>, start_position: usize, value: i128) -> usize {
    write_signed_leb128_to(value, |i, v| write_to_vec(out, start_position+i, v))
}

/// `read_signed_leb128` reads data from arg `data` at offset `start_position`
/// Returns the decoded `u128` value along with read size
#[inline]
pub fn read_signed_leb128(data: &[u8], start_position: usize) -> (i128, usize) {
    let mut result = 0;
    let mut shift = 0;
    let mut position = start_position;
    let mut byte;

    loop {
        byte = data[position];
        position += 1;
        result |= ((byte & 0x7F) as i128) << shift;
        shift += 7;

        if (byte & 0x80) == 0 {
            break;
        }
    }

    if (shift < 64) && ((byte & 0x40) != 0) {
        // sign extend
        result |= -(1 << shift);
    }

    (result, position - start_position)
}
