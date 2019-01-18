use {Parcel, Error};

use std::prelude::v1::*;
use std::io::prelude::*;

impl<T: Parcel> Parcel for Option<T>
{
    const TYPE_NAME: &'static str = "Option<T>";

    fn read(read: &mut Read) -> Result<Self, Error> {
        let is_some = bool::read(read)?;

        if is_some {
            let value = T::read(read)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn write(&self, write: &mut Write) -> Result<(), Error> {
        self.is_some().write(write)?;

        if let Some(ref value) = *self {
            value.write(write)?;
        }

        Ok(())
    }
}

