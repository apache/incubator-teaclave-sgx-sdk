use std::io;
use std::io::Read;
use std::cmp;
use std::ptr;
use std::vec::Vec;
use byteorder::ReadBytesExt;
use byteorder::LittleEndian;

use bit;
use lz77;
use util;
use super::symbol;

/// DEFLATE decoder.
#[derive(Debug)]
pub struct Decoder<R> {
    bit_reader: bit::BitReader<R>,
    buffer: Vec<u8>,
    offset: usize,
    eos: bool,
}
impl<R> Decoder<R>
where
    R: Read,
{
    /// Makes a new decoder instance.
    ///
    /// `inner` is to be decoded DEFLATE stream.
    ///
    /// # Examples
    /// ```
    /// use std::io::{Cursor, Read};
    /// use libflate::deflate::Decoder;
    ///
    /// let encoded_data = [243, 72, 205, 201, 201, 87, 8, 207, 47, 202, 73, 81, 4, 0];
    /// let mut decoder = Decoder::new(&encoded_data[..]);
    /// let mut buf = Vec::new();
    /// decoder.read_to_end(&mut buf).unwrap();
    ///
    /// assert_eq!(buf, b"Hello World!");
    /// ```
    pub fn new(inner: R) -> Self {
        Decoder {
            bit_reader: bit::BitReader::new(inner),
            buffer: Vec::new(),
            offset: 0,
            eos: false,
        }
    }

    /// Returns the immutable reference to the inner stream.
    pub fn as_inner_ref(&self) -> &R {
        self.bit_reader.as_inner_ref()
    }

    /// Returns the mutable reference to the inner stream.
    pub fn as_inner_mut(&mut self) -> &mut R {
        self.bit_reader.as_inner_mut()
    }

    /// Unwraps this `Decoder`, returning the underlying reader.
    ///
    /// # Examples
    /// ```
    /// use std::io::Cursor;
    /// use libflate::deflate::Decoder;
    ///
    /// let encoded_data = [243, 72, 205, 201, 201, 87, 8, 207, 47, 202, 73, 81, 4, 0];
    /// let decoder = Decoder::new(Cursor::new(&encoded_data));
    /// assert_eq!(decoder.into_inner().into_inner(), &encoded_data);
    /// ```
    pub fn into_inner(self) -> R {
        self.bit_reader.into_inner()
    }

    fn read_non_compressed_block(&mut self) -> io::Result<()> {
        self.bit_reader.reset();
        let len = self.bit_reader.as_inner_mut().read_u16::<LittleEndian>()?;
        let nlen = self.bit_reader.as_inner_mut().read_u16::<LittleEndian>()?;
        if !len != nlen {
            Err(invalid_data_error!(
                "LEN={} is not the one's complement of NLEN={}",
                len,
                nlen
            ))
        } else {
            let old_len = self.buffer.len();
            self.buffer.reserve(len as usize);
            unsafe { self.buffer.set_len(old_len + len as usize) };
            self.bit_reader.as_inner_mut().read_exact(
                &mut self.buffer[old_len..],
            )?;
            Ok(())
        }
    }
    fn read_compressed_block<H>(&mut self, huffman: H) -> io::Result<()>
    where
        H: symbol::HuffmanCodec,
    {
        let symbol_decoder = huffman.load(&mut self.bit_reader)?;
        loop {
            let s = symbol_decoder.decode_unchecked(&mut self.bit_reader);
            self.bit_reader.check_last_error()?;
            match s {
                symbol::Symbol::Literal(b) => {
                    self.buffer.push(b);
                }
                symbol::Symbol::Share { length, distance } => {
                    if self.buffer.len() < distance as usize {
                        return Err(invalid_data_error!(
                            "Too long backword reference: buffer.len={}, distance={}",
                            self.buffer.len(),
                            distance
                        ));
                    }
                    let old_len = self.buffer.len();
                    self.buffer.reserve(length as usize);
                    unsafe {
                        self.buffer.set_len(old_len + length as usize);
                        let start = old_len - distance as usize;
                        let ptr = self.buffer.as_mut_ptr();
                        util::ptr_copy(
                            ptr.offset(start as isize),
                            ptr.offset(old_len as isize),
                            length as usize,
                            length > distance,
                        );
                    }
                }
                symbol::Symbol::EndOfBlock => {
                    break;
                }
            }
        }
        Ok(())
    }
    fn truncate_old_buffer(&mut self) {
        if self.buffer.len() > lz77::MAX_DISTANCE as usize * 4 {
            let new_len = lz77::MAX_DISTANCE as usize;
            unsafe {
                let ptr = self.buffer.as_mut_ptr();
                let src = ptr.offset((self.buffer.len() - new_len) as isize);
                ptr::copy_nonoverlapping(src, ptr, new_len);
            }
            self.buffer.truncate(new_len);
            self.offset = new_len;
        }
    }
}
impl<R> Read for Decoder<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.offset < self.buffer.len() {
            let copy_size = cmp::min(buf.len(), self.buffer.len() - self.offset);
            buf[..copy_size].copy_from_slice(&self.buffer[self.offset..][..copy_size]);
            self.offset += copy_size;
            Ok(copy_size)
        } else if self.eos {
            Ok(0)
        } else {
            let bfinal = self.bit_reader.read_bit()?;
            let btype = self.bit_reader.read_bits(2)?;
            self.eos = bfinal;
            self.truncate_old_buffer();
            match btype {
                0b00 => {
                    self.read_non_compressed_block()?;
                    self.read(buf)
                }
                0b01 => {
                    self.read_compressed_block(symbol::FixedHuffmanCodec)?;
                    self.read(buf)
                }
                0b10 => {
                    self.read_compressed_block(symbol::DynamicHuffmanCodec)?;
                    self.read(buf)
                }
                0b11 => Err(invalid_data_error!(
                    "btype 0x11 of DEFLATE is reserved(error) value"
                )),
                _ => unreachable!(),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::io;
    use deflate::symbol::{HuffmanCodec, DynamicHuffmanCodec};
    use super::*;

    #[test]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn test_issues_3() {
        // see: https://github.com/sile/libflate/issues/3
        let input =
            [180, 253, 73, 143, 28, 201, 150, 46, 8, 254, 150, 184, 139, 75, 18, 69,
             247, 32, 157, 51, 27, 141, 132, 207, 78, 210, 167, 116, 243, 160, 223,
             136, 141, 66, 205, 76, 221, 76, 195, 213, 84, 236, 234, 224, 78, 227, 34,
             145, 221, 139, 126, 232, 69, 173, 170, 208, 192, 219, 245, 67, 3, 15, 149,
             120, 171, 70, 53, 106, 213, 175, 23, 21, 153, 139, 254, 27, 249, 75, 234,
             124, 71, 116, 56, 71, 68, 212, 204, 121, 115, 64, 222, 160, 203, 119,
             142, 170, 169, 138, 202, 112, 228, 140, 38];
        let mut bit_reader = ::bit::BitReader::new(&input[..]);
        assert_eq!(bit_reader.read_bit().unwrap(), false); // not final block
        assert_eq!(bit_reader.read_bits(2).unwrap(), 0b10); // DynamicHuffmanCodec
        DynamicHuffmanCodec.load(&mut bit_reader).unwrap();
    }

    #[test]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn it_works() {
        let input =
            [180, 253, 73, 143, 28, 201, 150, 46, 8, 254, 150, 184, 139, 75, 18,
             69, 247, 32, 157, 51, 27, 141, 132, 207, 78, 210, 167, 116, 243,
             160, 223, 136, 141, 66, 205, 76, 221, 76, 195, 213, 84, 236, 234,
             224, 78, 227, 34, 145, 221, 139, 126, 232, 69, 173, 170, 208, 192,
             219, 245, 67, 3, 15, 149, 120, 171, 70, 53, 106, 213, 175, 23, 21,
             153, 139, 254, 27, 249, 75, 234, 124, 71, 116, 56, 71, 68, 212, 204,
             121, 115, 64, 222, 160, 203, 119, 142, 170, 169, 138, 202, 112, 228,
             140, 38, 171, 162, 88, 212, 235, 56, 136, 231, 233, 239, 113, 249,
             163, 252, 16, 42, 138, 49, 226, 108, 73, 28, 153];
        let mut decoder = Decoder::new(&input[..]);

        let result = io::copy(&mut decoder, &mut io::sink());
        assert!(result.is_err());

        let error = result.err().unwrap();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().starts_with("Too long backword reference"));
    }
}
