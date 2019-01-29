//! A type safe `Parcel` data transformation pipeline.

pub use self::pipeline::Pipeline;

#[macro_use]
pub mod pipeline;
pub mod compression;
pub mod rotate_bytes;

use Error;
use std;
use std::prelude::v1::*;

/// A hook that sits between reading and writing packets.
///
/// Applies one transformation encoding the data, and
/// performs the opposite transformation to decode it.
pub trait Middleware : std::fmt::Debug
{
    /// Processes some data.
    fn encode_data(&mut self, data: Vec<u8>) -> Result<Vec<u8>, Error>;
    /// Un-processes some data.
    fn decode_data(&mut self, data: Vec<u8>) -> Result<Vec<u8>, Error>;
}

