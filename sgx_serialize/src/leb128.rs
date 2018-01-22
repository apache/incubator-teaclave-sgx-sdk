// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

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
    where W: FnMut(usize, u8)
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
    where W: FnMut(usize, u8)
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
