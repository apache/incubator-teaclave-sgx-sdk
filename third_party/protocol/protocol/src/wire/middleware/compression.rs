//! A middleware for compressing all transmitted data.

use Error;
use wire;

use std::io::prelude::*;

/// Defines a compression algorithm.
#[derive(Copy, Clone, Debug)]
pub enum Algorithm
{
    /// The zlib compression algorithm.
    ///
    /// https://en.wikipedia.org/wiki/Zlib
    Zlib,
}

/// Compression middleware.
#[derive(Clone, Debug)]
pub enum Compression
{
    /// No compression or decompression should be applied to the data.
    Disabled,
    /// Compression and decompression should be applied to the data.
    Enabled(Algorithm),
}

use std::prelude::v1::*;
impl wire::Middleware for Compression
{
    fn encode_data(&mut self, data: Vec<u8>) -> Result<Vec<u8>, Error> {
        match *self {
            Compression::Enabled(algorithm) => match algorithm {
                Algorithm::Zlib => {
                    use libflate::zlib::Encoder;
                    let mut e = Encoder::new(Vec::new()).unwrap();
                    e.write_all(&data)?;

                    Ok(e.finish().into_result()?)
                },
            },
            Compression::Disabled => Ok(data),
        }
    }

    /// Un-processes some data.
    fn decode_data(&mut self, data: Vec<u8>) -> Result<Vec<u8>, Error> {
        match *self {
            Compression::Enabled(algorithm) => match algorithm {
                Algorithm::Zlib => {
                    use libflate::zlib::Decoder;
                    let mut e = Decoder::new(&data[..]).unwrap();
                    let mut buf = Vec::new();
                    e.read_to_end(&mut buf)?;
                    Ok(buf)
                },
            },
            Compression::Disabled => Ok(data),
        }
    }
}

