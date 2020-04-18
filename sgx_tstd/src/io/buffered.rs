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

//! Buffering wrappers for I/O traits

use crate::io::prelude::*;
use crate::error;
use crate::io::{
    self, Error, ErrorKind, Initializer, IoSlice, IoSliceMut, SeekFrom, DEFAULT_BUF_SIZE,
};
use crate::memchr;
use core::cmp;
use core::fmt;

/// The `BufReader` struct adds buffering to any reader.
///
/// It can be excessively inefficient to work directly with a [`Read`] instance.
/// For example, every call to [`read`][`TcpStream::read`] on [`TcpStream`]
/// results in a system call. A `BufReader` performs large, infrequent reads on
/// the underlying [`Read`] and maintains an in-memory buffer of the results.
///
/// `BufReader` can improve the speed of programs that make *small* and
/// *repeated* read calls to the same file or network socket. It does not
/// help when reading very large amounts at once, or reading just one or a few
/// times. It also provides no advantage when reading from a source that is
/// already in memory, like a `Vec<u8>`.
///
/// When the `BufReader<R>` is dropped, the contents of its buffer will be
/// discarded. Creating multiple instances of a `BufReader<R>` on the same
/// stream can cause data loss. Reading from the underlying reader after
/// unwrapping the `BufReader<R>` with `BufReader::into_inner` can also cause
/// data loss.
///
/// [`Read`]: ../../std/io/trait.Read.html
/// [`TcpStream::read`]: ../../std/net/struct.TcpStream.html#method.read
/// [`TcpStream`]: ../../std/net/struct.TcpStream.html
///
pub struct BufReader<R> {
    inner: R,
    buf: Box<[u8]>,
    pos: usize,
    cap: usize,
}

impl<R: Read> BufReader<R> {
    /// Creates a new `BufReader<R>` with a default buffer capacity. The default is currently 8 KB,
    /// but may change in the future.
    ///
    pub fn new(inner: R) -> BufReader<R> {
        BufReader::with_capacity(DEFAULT_BUF_SIZE, inner)
    }

    /// Creates a new `BufReader<R>` with the specified buffer capacity.
    ///
    pub fn with_capacity(capacity: usize, inner: R) -> BufReader<R> {
        unsafe {
            let mut buffer = Vec::with_capacity(capacity);
            buffer.set_len(capacity);
            inner.initializer().initialize(&mut buffer);
            BufReader { inner, buf: buffer.into_boxed_slice(), pos: 0, cap: 0 }
        }
    }
}

impl<R> BufReader<R> {
    /// Gets a reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    ///
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    /// Gets a mutable reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    ///
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Returns a reference to the internally buffered data.
    ///
    /// Unlike `fill_buf`, this will not attempt to fill the buffer if it is empty.
    ///
    pub fn buffer(&self) -> &[u8] {
        &self.buf[self.pos..self.cap]
    }

    /// Returns the number of bytes the internal buffer can hold at once.
    ///
    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    /// Unwraps this `BufReader<R>`, returning the underlying reader.
    ///
    /// Note that any leftover data in the internal buffer is lost. Therefore,
    /// a following read from the underlying reader may lead to data loss.
    ///
    pub fn into_inner(self) -> R {
        self.inner
    }

    /// Invalidates all data in the internal buffer.
    #[inline]
    fn discard_buffer(&mut self) {
        self.pos = 0;
        self.cap = 0;
    }
}

impl<R: Seek> BufReader<R> {
    /// Seeks relative to the current position. If the new position lies within the buffer,
    /// the buffer will not be flushed, allowing for more efficient seeks.
    /// This method does not return the location of the underlying reader, so the caller
    /// must track this information themselves if it is required.
    pub fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
        let pos = self.pos as u64;
        if offset < 0 {
            if let Some(new_pos) = pos.checked_sub((-offset) as u64) {
                self.pos = new_pos as usize;
                return Ok(());
            }
        } else {
            if let Some(new_pos) = pos.checked_add(offset as u64) {
                if new_pos <= self.cap as u64 {
                    self.pos = new_pos as usize;
                    return Ok(());
                }
            }
        }
        self.seek(SeekFrom::Current(offset)).map(drop)
    }
}

impl<R: Read> Read for BufReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // If we don't have any buffered data and we're doing a massive read
        // (larger than our internal buffer), bypass our internal buffer
        // entirely.
        if self.pos == self.cap && buf.len() >= self.buf.len() {
            self.discard_buffer();
            return self.inner.read(buf);
        }
        let nread = {
            let mut rem = self.fill_buf()?;
            rem.read(buf)?
        };
        self.consume(nread);
        Ok(nread)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let total_len = bufs.iter().map(|b| b.len()).sum::<usize>();
        if self.pos == self.cap && total_len >= self.buf.len() {
            self.discard_buffer();
            return self.inner.read_vectored(bufs);
        }
        let nread = {
            let mut rem = self.fill_buf()?;
            rem.read_vectored(bufs)?
        };
        self.consume(nread);
        Ok(nread)
    }

    // we can't skip unconditionally because of the large buffer case in read.
    unsafe fn initializer(&self) -> Initializer {
        self.inner.initializer()
    }
}

impl<R: Read> BufRead for BufReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        // If we've reached the end of our internal buffer then we need to fetch
        // some more data from the underlying reader.
        // Branch using `>=` instead of the more correct `==`
        // to tell the compiler that the pos..cap slice is always valid.
        if self.pos >= self.cap {
            debug_assert!(self.pos == self.cap);
            self.cap = self.inner.read(&mut self.buf)?;
            self.pos = 0;
        }
        Ok(&self.buf[self.pos..self.cap])
    }

    fn consume(&mut self, amt: usize) {
        self.pos = cmp::min(self.pos + amt, self.cap);
    }
}

impl<R> fmt::Debug for BufReader<R>
where
    R: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("BufReader")
            .field("reader", &self.inner)
            .field("buffer", &format_args!("{}/{}", self.cap - self.pos, self.buf.len()))
            .finish()
    }
}

impl<R: Seek> Seek for BufReader<R> {
    /// Seek to an offset, in bytes, in the underlying reader.
    ///
    /// The position used for seeking with `SeekFrom::Current(_)` is the
    /// position the underlying reader would be at if the `BufReader<R>` had no
    /// internal buffer.
    ///
    /// Seeking always discards the internal buffer, even if the seek position
    /// would otherwise fall within it. This guarantees that calling
    /// `.into_inner()` immediately after a seek yields the underlying reader
    /// at the same position.
    ///
    /// To seek without discarding the internal buffer, use [`BufReader::seek_relative`].
    ///
    /// See [`std::io::Seek`] for more details.
    ///
    /// Note: In the edge case where you're seeking with `SeekFrom::Current(n)`
    /// where `n` minus the internal buffer length overflows an `i64`, two
    /// seeks will be performed instead of one. If the second seek returns
    /// `Err`, the underlying reader will be left at the same position it would
    /// have if you called `seek` with `SeekFrom::Current(0)`.
    ///
    /// [`BufReader::seek_relative`]: struct.BufReader.html#method.seek_relative
    /// [`std::io::Seek`]: trait.Seek.html
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let result: u64;
        if let SeekFrom::Current(n) = pos {
            let remainder = (self.cap - self.pos) as i64;
            // it should be safe to assume that remainder fits within an i64 as the alternative
            // means we managed to allocate 8 exbibytes and that's absurd.
            // But it's not out of the realm of possibility for some weird underlying reader to
            // support seeking by i64::min_value() so we need to handle underflow when subtracting
            // remainder.
            if let Some(offset) = n.checked_sub(remainder) {
                result = self.inner.seek(SeekFrom::Current(offset))?;
            } else {
                // seek backwards by our remainder, and then by the offset
                self.inner.seek(SeekFrom::Current(-remainder))?;
                self.discard_buffer();
                result = self.inner.seek(SeekFrom::Current(n))?;
            }
        } else {
            // Seeking with Start/End doesn't care about our buffer length.
            result = self.inner.seek(pos)?;
        }
        self.discard_buffer();
        Ok(result)
    }
}

/// Wraps a writer and buffers its output.
///
/// It can be excessively inefficient to work directly with something that
/// implements [`Write`]. For example, every call to
/// [`write`][`TcpStream::write`] on [`TcpStream`] results in a system call. A
/// `BufWriter<W>` keeps an in-memory buffer of data and writes it to an underlying
/// writer in large, infrequent batches.
///
/// `BufWriter<W>` can improve the speed of programs that make *small* and
/// *repeated* write calls to the same file or network socket. It does not
/// help when writing very large amounts at once, or writing just one or a few
/// times. It also provides no advantage when writing to a destination that is
/// in memory, like a `Vec<u8>`.
///
/// It is critical to call [`flush`] before `BufWriter<W>` is dropped. Though
/// dropping will attempt to flush the the contents of the buffer, any errors
/// that happen in the process of dropping will be ignored. Calling [`flush`]
/// ensures that the buffer is empty and thus dropping will not even attempt
/// file operations.
///
/// By wrapping the stream with a `BufWriter<W>`, these ten writes are all grouped
/// together by the buffer and will all be written out in one system call when
/// the `stream` is flushed.
///
/// [`Write`]: ../../std/io/trait.Write.html
/// [`TcpStream::write`]: ../../std/net/struct.TcpStream.html#method.write
/// [`TcpStream`]: ../../std/net/struct.TcpStream.html
/// [`flush`]: #method.flush
pub struct BufWriter<W: Write> {
    inner: Option<W>,
    buf: Vec<u8>,
    // #30888: If the inner writer panics in a call to write, we don't want to
    // write the buffered data a second time in BufWriter's destructor. This
    // flag tells the Drop impl if it should skip the flush.
    panicked: bool,
}

/// An error returned by `into_inner` which combines an error that
/// happened while writing out the buffer, and the buffered writer object
/// which may be used to recover from the condition.
///
#[derive(Debug)]
pub struct IntoInnerError<W>(W, Error);

impl<W: Write> BufWriter<W> {
    /// Creates a new `BufWriter<W>` with a default buffer capacity. The default is currently 8 KB,
    /// but may change in the future.
    ///
    pub fn new(inner: W) -> BufWriter<W> {
        BufWriter::with_capacity(DEFAULT_BUF_SIZE, inner)
    }

    /// Creates a new `BufWriter<W>` with the specified buffer capacity.
    ///
    pub fn with_capacity(capacity: usize, inner: W) -> BufWriter<W> {
        BufWriter { inner: Some(inner), buf: Vec::with_capacity(capacity), panicked: false }
    }

    fn flush_buf(&mut self) -> io::Result<()> {
        let mut written = 0;
        let len = self.buf.len();
        let mut ret = Ok(());
        while written < len {
            self.panicked = true;
            let r = self.inner.as_mut().unwrap().write(&self.buf[written..]);
            self.panicked = false;

            match r {
                Ok(0) => {
                    ret =
                        Err(Error::new(ErrorKind::WriteZero, "failed to write the buffered data"));
                    break;
                }
                Ok(n) => written += n,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => {
                    ret = Err(e);
                    break;
                }
            }
        }
        if written > 0 {
            self.buf.drain(..written);
        }
        ret
    }

    /// Gets a reference to the underlying writer.
    ///
    pub fn get_ref(&self) -> &W {
        self.inner.as_ref().unwrap()
    }

    /// Gets a mutable reference to the underlying writer.
    ///
    /// It is inadvisable to directly write to the underlying writer.
    ///
    pub fn get_mut(&mut self) -> &mut W {
        self.inner.as_mut().unwrap()
    }

    /// Returns a reference to the internally buffered data.
    ///
    pub fn buffer(&self) -> &[u8] {
        &self.buf
    }

    /// Returns the number of bytes the internal buffer can hold without flushing.
    ///
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    /// Unwraps this `BufWriter<W>`, returning the underlying writer.
    ///
    /// The buffer is written out before returning the writer.
    ///
    /// # Errors
    ///
    /// An `Err` will be returned if an error occurs while flushing the buffer.
    ///
    pub fn into_inner(mut self) -> Result<W, IntoInnerError<BufWriter<W>>> {
        match self.flush_buf() {
            Err(e) => Err(IntoInnerError(self, e)),
            Ok(()) => Ok(self.inner.take().unwrap()),
        }
    }
}

impl<W: Write> Write for BufWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.buf.len() + buf.len() > self.buf.capacity() {
            self.flush_buf()?;
        }
        if buf.len() >= self.buf.capacity() {
            self.panicked = true;
            let r = self.get_mut().write(buf);
            self.panicked = false;
            r
        } else {
            self.buf.write(buf)
        }
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let total_len = bufs.iter().map(|b| b.len()).sum::<usize>();
        if self.buf.len() + total_len > self.buf.capacity() {
            self.flush_buf()?;
        }
        if total_len >= self.buf.capacity() {
            self.panicked = true;
            let r = self.get_mut().write_vectored(bufs);
            self.panicked = false;
            r
        } else {
            self.buf.write_vectored(bufs)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_buf().and_then(|()| self.get_mut().flush())
    }
}

impl<W: Write> fmt::Debug for BufWriter<W>
where
    W: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("BufWriter")
            .field("writer", &self.inner.as_ref().unwrap())
            .field("buffer", &format_args!("{}/{}", self.buf.len(), self.buf.capacity()))
            .finish()
    }
}

impl<W: Write + Seek> Seek for BufWriter<W> {
    /// Seek to the offset, in bytes, in the underlying writer.
    ///
    /// Seeking always writes out the internal buffer before seeking.
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.flush_buf().and_then(|_| self.get_mut().seek(pos))
    }
}

impl<W: Write> Drop for BufWriter<W> {
    fn drop(&mut self) {
        if self.inner.is_some() && !self.panicked {
            // dtors should not panic, so we ignore a failed flush
            let _r = self.flush_buf();
        }
    }
}

impl<W> IntoInnerError<W> {
    /// Returns the error which caused the call to `into_inner()` to fail.
    ///
    /// This error was returned when attempting to write the internal buffer.
    ///
    pub fn error(&self) -> &Error {
        &self.1
    }

    /// Returns the buffered writer instance which generated the error.
    ///
    /// The returned object can be used for error recovery, such as
    /// re-inspecting the buffer.
    ///
    pub fn into_inner(self) -> W {
        self.0
    }
}

impl<W> From<IntoInnerError<W>> for Error {
    fn from(iie: IntoInnerError<W>) -> Error {
        iie.1
    }
}

impl<W: Send + fmt::Debug> error::Error for IntoInnerError<W> {
    fn description(&self) -> &str {
        error::Error::description(self.error())
    }
}

impl<W> fmt::Display for IntoInnerError<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error().fmt(f)
    }
}

/// Wraps a writer and buffers output to it, flushing whenever a newline
/// (`0x0a`, `'\n'`) is detected.
///
/// The [`BufWriter`][bufwriter] struct wraps a writer and buffers its output.
/// But it only does this batched write when it goes out of scope, or when the
/// internal buffer is full. Sometimes, you'd prefer to write each line as it's
/// completed, rather than the entire buffer at once. Enter `LineWriter`. It
/// does exactly that.
///
/// Like [`BufWriter`][bufwriter], a `LineWriter`’s buffer will also be flushed when the
/// `LineWriter` goes out of scope or when its internal buffer is full.
///
/// [bufwriter]: struct.BufWriter.html
///
/// If there's still a partial line in the buffer when the `LineWriter` is
/// dropped, it will flush those contents.
///
pub struct LineWriter<W: Write> {
    inner: BufWriter<W>,
    need_flush: bool,
}

impl<W: Write> LineWriter<W> {
    /// Creates a new `LineWriter`.
    ///
    pub fn new(inner: W) -> LineWriter<W> {
        // Lines typically aren't that long, don't use a giant buffer
        LineWriter::with_capacity(1024, inner)
    }

    /// Creates a new `LineWriter` with a specified capacity for the internal
    /// buffer.
    ///
    pub fn with_capacity(capacity: usize, inner: W) -> LineWriter<W> {
        LineWriter { inner: BufWriter::with_capacity(capacity, inner), need_flush: false }
    }

    /// Gets a reference to the underlying writer.
    ///
    pub fn get_ref(&self) -> &W {
        self.inner.get_ref()
    }

    /// Gets a mutable reference to the underlying writer.
    ///
    /// Caution must be taken when calling methods on the mutable reference
    /// returned as extra writes could corrupt the output stream.
    ///
    pub fn get_mut(&mut self) -> &mut W {
        self.inner.get_mut()
    }

    /// Unwraps this `LineWriter`, returning the underlying writer.
    ///
    /// The internal buffer is written out before returning the writer.
    ///
    /// # Errors
    ///
    /// An `Err` will be returned if an error occurs while flushing the buffer.
    ///
    pub fn into_inner(self) -> Result<W, IntoInnerError<LineWriter<W>>> {
        self.inner.into_inner().map_err(|IntoInnerError(buf, e)| {
            IntoInnerError(LineWriter { inner: buf, need_flush: false }, e)
        })
    }
}

impl<W: Write> Write for LineWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.need_flush {
            self.flush()?;
        }

        // Find the last newline character in the buffer provided. If found then
        // we're going to write all the data up to that point and then flush,
        // otherwise we just write the whole block to the underlying writer.
        let i = match memchr::memrchr(b'\n', buf) {
            Some(i) => i,
            None => return self.inner.write(buf),
        };

        // Ok, we're going to write a partial amount of the data given first
        // followed by flushing the newline. After we've successfully written
        // some data then we *must* report that we wrote that data, so future
        // errors are ignored. We set our internal `need_flush` flag, though, in
        // case flushing fails and we need to try it first next time.
        let n = self.inner.write(&buf[..=i])?;
        self.need_flush = true;
        if self.flush().is_err() || n != i + 1 {
            return Ok(n);
        }

        // At this point we successfully wrote `i + 1` bytes and flushed it out,
        // meaning that the entire line is now flushed out on the screen. While
        // we can attempt to finish writing the rest of the data provided.
        // Remember though that we ignore errors here as we've successfully
        // written data, so we need to report that.
        match self.inner.write(&buf[i + 1..]) {
            Ok(i) => Ok(n + i),
            Err(_) => Ok(n),
        }
    }

    // Vectored writes are very similar to the writes above, but adjusted for
    // the list of buffers that we have to write.
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        if self.need_flush {
            self.flush()?;
        }

        // Find the last newline, and failing that write the whole buffer
        let last_newline = bufs
            .iter()
            .enumerate()
            .rev()
            .filter_map(|(i, buf)| {
                let pos = memchr::memrchr(b'\n', buf)?;
                Some((i, pos))
            })
            .next();
        let (i, j) = match last_newline {
            Some(pair) => pair,
            None => return self.inner.write_vectored(bufs),
        };
        let (prefix, suffix) = bufs.split_at(i);
        let (buf, suffix) = suffix.split_at(1);
        let buf = &buf[0];

        // Write everything up to the last newline, flushing afterwards. Note
        // that only if we finished our entire `write_vectored` do we try the
        // subsequent
        // `write`
        let mut n = 0;
        let prefix_amt = prefix.iter().map(|i| i.len()).sum();
        if prefix_amt > 0 {
            n += self.inner.write_vectored(prefix)?;
            self.need_flush = true;
        }
        if n == prefix_amt {
            match self.inner.write(&buf[..=j]) {
                Ok(m) => n += m,
                Err(e) if n == 0 => return Err(e),
                Err(_) => return Ok(n),
            }
            self.need_flush = true;
        }
        if self.flush().is_err() || n != j + 1 + prefix_amt {
            return Ok(n);
        }

        // ... and now write out everything remaining
        match self.inner.write(&buf[j + 1..]) {
            Ok(i) => n += i,
            Err(_) => return Ok(n),
        }

        if suffix.iter().map(|s| s.len()).sum::<usize>() == 0 {
            return Ok(n);
        }
        match self.inner.write_vectored(suffix) {
            Ok(i) => Ok(n + i),
            Err(_) => Ok(n),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()?;
        self.need_flush = false;
        Ok(())
    }
}

impl<W: Write> fmt::Debug for LineWriter<W>
where
    W: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("LineWriter")
            .field("writer", &self.inner.inner)
            .field(
                "buffer",
                &format_args!("{}/{}", self.inner.buf.len(), self.inner.buf.capacity()),
            )
            .finish()
    }
}

