pub use self::simple::Simple;

pub mod simple;

use Error;
use std::prelude::v1::*;
use std::io::prelude::*;

pub trait Transport
{
    fn process_data(&mut self,
                    read: &mut dyn Read) -> Result<(), Error>;

    fn receive_raw_packet(&mut self) -> Result<Option<Vec<u8>>, Error>;

    fn send_raw_packet(&mut self,
                       write: &mut dyn Write,
                       packet: &[u8]) -> Result<(), Error>;
}

