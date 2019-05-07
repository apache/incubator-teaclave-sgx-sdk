// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
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

#![allow(missing_copy_implementations)]

use io::{self, Read, Initializer, Write, ErrorKind, BufRead};
use core::fmt;
use core::mem;

/// Copies the entire contents of a reader into a writer.
///
/// This function will continuously read data from `reader` and then
/// write it into `writer` in a streaming fashion until `reader`
/// returns EOF.
///
/// On success, the total number of bytes that were copied from
/// `reader` to `writer` is returned.
///
/// # Errors
///
/// This function will return an error immediately if any call to `read` or
/// `write` returns an error. All instances of `ErrorKind::Interrupted` are
/// handled by this function and the underlying operation is retried.
pub fn copy<R: ?Sized, W: ?Sized>(reader: &mut R, writer: &mut W) -> io::Result<u64>
    where R: Read, W: Write
{
    let mut buf = unsafe {
        let mut buf: [u8; super::DEFAULT_BUF_SIZE] = mem::uninitialized();
        reader.initializer().initialize(&mut buf);
        buf
    };

    let mut written = 0;
    loop {
        let len = match reader.read(&mut buf) {
            Ok(0) => return Ok(written),
            Ok(len) => len,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        writer.write_all(&buf[..len])?;
        written += len as u64;
    }
}

/// A reader which is always at EOF.
///
/// This struct is generally created by calling [`empty`]. Please see
/// the documentation of [`empty()`][`empty`] for more details.
///
/// [`empty`]: fn.empty.html
pub struct Empty { _priv: () }

/// Constructs a new handle to an empty reader.
///
/// All reads from the returned reader will return `Ok(0)`.
pub fn empty() -> Empty { Empty { _priv: () } }

impl Read for Empty {
    #[inline]
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> { Ok(0) }

    #[inline]
    unsafe fn initializer(&self) -> Initializer {
        Initializer::nop()
    }
}

impl BufRead for Empty {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> { Ok(&[]) }
    #[inline]
    fn consume(&mut self, _n: usize) {}
}

impl fmt::Debug for Empty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("Empty { .. }")
    }
}

/// A reader which yields one byte over and over and over and over and over and...
///
/// This struct is generally created by calling [`repeat`][repeat]. Please
/// see the documentation of `repeat()` for more details.
///
/// [repeat]: fn.repeat.html
pub struct Repeat { byte: u8 }

/// Creates an instance of a reader that infinitely repeats one byte.
///
/// All reads from this reader will succeed by filling the specified buffer with
/// the given byte.
pub fn repeat(byte: u8) -> Repeat { Repeat { byte: byte } }

impl Read for Repeat {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        for slot in &mut *buf {
            *slot = self.byte;
        }
        Ok(buf.len())
    }

    #[inline]
    unsafe fn initializer(&self) -> Initializer {
        Initializer::nop()
    }
}

impl fmt::Debug for Repeat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("Repeat { .. }")
    }
}

/// A writer which will move data into the void.
///
/// This struct is generally created by calling [`sink`][sink]. Please
/// see the documentation of `sink()` for more details.
///
/// [sink]: fn.sink.html
pub struct Sink { _priv: () }

/// Creates an instance of a writer which will successfully consume all data.
///
/// All calls to `write` on the returned instance will return `Ok(buf.len())`
/// and the contents of the buffer will not be inspected.
pub fn sink() -> Sink { Sink { _priv: () } }

impl Write for Sink {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> { Ok(buf.len()) }
    #[inline]
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

impl fmt::Debug for Sink {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("Sink { .. }")
    }
}