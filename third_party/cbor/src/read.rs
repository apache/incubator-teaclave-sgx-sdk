use std::prelude::v1::*;
use std::cmp;
use std::io::{self, Read as StdRead};

use error::{Result, Error, ErrorCode};

/// Trait used by the deserializer for iterating over input.
///
/// This trait is sealed and cannot be implemented for types outside of `serde_cbor`.
pub trait Read<'de>: private::Sealed {
    #[doc(hidden)]
    fn next(&mut self) -> io::Result<Option<u8>>;
    #[doc(hidden)]
    fn peek(&mut self) -> io::Result<Option<u8>>;

    #[doc(hidden)]
    fn read(
        &mut self,
        n: usize,
        scratch: &mut Vec<u8>,
        scratch_offset: usize,
    ) -> Result<Reference<'de>>;

    #[doc(hidden)]
    fn read_into(&mut self, buf: &mut [u8]) -> Result<()>;

    #[doc(hidden)]
    fn discard(&mut self);

    #[doc(hidden)]
    fn offset(&self) -> u64;
}

pub enum Reference<'b> {
    Borrowed(&'b [u8]),
    Copied,
}

mod private {
    pub trait Sealed {}
}

/// CBOR input source that reads from a std::io input stream.
pub struct IoRead<R>
where
    R: io::Read,
{
    reader: OffsetReader<R>,
    ch: Option<u8>,
}

impl<R> IoRead<R>
where
    R: io::Read,
{
    /// Creates a new CBOR input source to read from a std::io input stream.
    pub fn new(reader: R) -> IoRead<R> {
        IoRead {
            reader: OffsetReader {
                reader,
                offset: 0,
            },
            ch: None,
        }
    }

    #[inline]
    fn next_inner(&mut self) -> io::Result<Option<u8>> {
        let mut buf = [0; 1];
        loop {
            match self.reader.read(&mut buf) {
                Ok(0) => return Ok(None),
                Ok(_) => return Ok(Some(buf[0])),
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
    }
}

impl<R> private::Sealed for IoRead<R>
where
    R: io::Read,
{
}

impl<'de, R> Read<'de> for IoRead<R>
where
    R: io::Read,
{
    #[inline]
    fn next(&mut self) -> io::Result<Option<u8>> {
        match self.ch.take() {
            Some(ch) => Ok(Some(ch)),
            None => self.next_inner(),
        }
    }

    #[inline]
    fn peek(&mut self) -> io::Result<Option<u8>> {
        match self.ch {
            Some(ch) => Ok(Some(ch)),
            None => {
                self.ch = self.next_inner()?;
                Ok(self.ch)
            }
        }
    }

    fn read(
        &mut self,
        mut n: usize,
        scratch: &mut Vec<u8>,
        mut scratch_offset: usize,
    ) -> Result<Reference<'de>> {
        while n > 0 {
            // defend against malicious input pretending to be huge strings by limiting growth
            let to_read = cmp::min(n, 16 * 1024);
            n -= to_read;

            if to_read > scratch.len() - scratch_offset {
                scratch.resize(scratch_offset + to_read, 0);
            }

            if let Some(ch) = self.ch.take() {
                scratch[scratch_offset] = ch;
                scratch_offset += 1;
            }

            self.read_into(&mut scratch[scratch_offset..])?;
            scratch_offset = scratch.len();
        }

        Ok(Reference::Copied)
    }

    fn read_into(&mut self, mut buf: &mut [u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.reader.read(buf) {
                Ok(0) => {
                    return Err(Error::syntax(
                        ErrorCode::EofWhileParsingValue,
                        self.offset(),
                    ))
                }
                Ok(count) => {
                    buf = &mut {
                        buf
                    }[count..]
                }
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(Error::io(e)),
            }
        }

        Ok(())
    }

    #[inline]
    fn discard(&mut self) {
        self.ch = None;
    }

    fn offset(&self) -> u64 {
        self.reader.offset
    }
}

struct OffsetReader<R> {
    reader: R,
    offset: u64,
}

impl<R> io::Read for OffsetReader<R>
where
    R: io::Read,
{
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let r = self.reader.read(buf);
        if let Ok(count) = r {
            self.offset += count as u64;
        }
        r
    }
}

/// A CBOR input source that reads from a slice of bytes.
pub struct SliceRead<'a> {
    slice: &'a [u8],
    index: usize,
}

impl<'a> SliceRead<'a> {
    /// Creates a CBOR input source to read from a slice of bytes.
    pub fn new(slice: &'a [u8]) -> SliceRead<'a> {
        SliceRead {
            slice,
            index: 0,
        }
    }

    fn end(&self, n: usize) -> Result<usize> {
        match self.index.checked_add(n) {
            Some(end) if end <= self.slice.len() => Ok(end),
            _ => {
                Err(Error::syntax(
                    ErrorCode::EofWhileParsingValue,
                    self.slice.len() as u64,
                ))
            }
        }
    }
}

impl<'a> private::Sealed for SliceRead<'a> {}

impl<'a> Read<'a> for SliceRead<'a> {
    #[inline]
    fn next(&mut self) -> io::Result<Option<u8>> {
        Ok(if self.index < self.slice.len() {
            let ch = self.slice[self.index];
            self.index += 1;
            Some(ch)
        } else {
            None
        })
    }

    #[inline]
    fn peek(&mut self) -> io::Result<Option<u8>> {
        Ok(if self.index < self.slice.len() {
            Some(self.slice[self.index])
        } else {
            None
        })
    }

    #[inline]
    fn read(&mut self, n: usize, _: &mut Vec<u8>, _: usize) -> Result<Reference<'a>> {
        let end = self.end(n)?;
        let slice = &self.slice[self.index..end];
        self.index = end;
        Ok(Reference::Borrowed(slice))
    }

    #[inline]
    fn read_into(&mut self, buf: &mut [u8]) -> Result<()> {
        let end = self.end(buf.len())?;
        buf.copy_from_slice(&self.slice[self.index..end]);
        self.index = end;
        Ok(())
    }

    #[inline]
    fn discard(&mut self) {
        self.index += 1;
    }

    fn offset(&self) -> u64 {
        self.index as u64
    }
}
