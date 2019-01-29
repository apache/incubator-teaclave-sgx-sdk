use {Parcel, Error};
use wire::middleware;

use std::io::prelude::*;
use std::io::Cursor;
use std::prelude::v1::*;
use std;

/// A datagram-based packet pipeline.
#[derive(Clone, Debug)]
pub struct Pipeline<P: Parcel, M: middleware::Pipeline>
{
    pub middleware: M,

    _a: std::marker::PhantomData<P>,
}

impl<P,M> Pipeline<P,M>
    where P: Parcel, M: middleware::Pipeline
{
    pub fn new(middleware: M) -> Self {
        Pipeline {
            middleware: middleware,
            _a: std::marker::PhantomData,
        }
    }

    /// Reads a packet from a buffer which contains a single packet.
    pub fn receive_from(&mut self, buffer: &mut Read)
        -> Result<P, Error> {
        let raw_bytes: Result<Vec<u8>, _> = buffer.bytes().collect();
        let raw_bytes = raw_bytes?;

        let mut bytes = Cursor::new(self.middleware.decode_data(raw_bytes)?);
        P::read(&mut bytes)
    }

    /// Writes a packet into a buffer.
    pub fn send_to(&mut self, buffer: &mut Write, packet: &P)
        -> Result<(), Error> {
        let bytes = self.middleware.encode_data(packet.raw_bytes()?)?;
        buffer.write(&bytes)?;
        Ok(())
    }
}

