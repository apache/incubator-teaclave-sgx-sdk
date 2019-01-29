use {Parcel, Error};
use std::io::prelude::*;

use std::prelude::v1::*;
use uuid::Uuid;

impl Parcel for Uuid
{
    fn read(read: &mut Read) -> Result<Self, Error> {
        let bytes: Result<Vec<u8>, _> = read.bytes().take(16).collect();
        let bytes = bytes?;

        Ok(Uuid::from_bytes(&bytes)?)
    }

    fn write(&self, write: &mut Write) -> Result<(), Error> {
        write.write(self.as_bytes())?;
        Ok(())
    }
}

