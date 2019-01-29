use {Parcel, Error};

use std::prelude::v1::*;
use std::io::prelude::*;

impl<T0, T1> Parcel for (T0, T1)
    where T0: Parcel, T1: Parcel
{
    const TYPE_NAME: &'static str = "(T0, T1)";

    fn read(read: &mut Read) -> Result<Self, Error> {
        let v0 = T0::read(read)?;
        let v1 = T1::read(read)?;
        Ok((v0, v1))
    }

    fn write(&self, write: &mut Write) -> Result<(), Error> {
        self.0.write(write)?;
        self.1.write(write)?;

        Ok(())
    }
}

impl<T0, T1, T2> Parcel for (T0, T1, T2)
    where T0: Parcel, T1: Parcel, T2: Parcel
{
    const TYPE_NAME: &'static str = "(T0, T1, T2)";

    fn read(read: &mut Read) -> Result<Self, Error> {
        let v0 = T0::read(read)?;
        let v1 = T1::read(read)?;
        let v2 = T2::read(read)?;
        Ok((v0, v1, v2))
    }

    fn write(&self, write: &mut Write) -> Result<(), Error> {
        self.0.write(write)?;
        self.1.write(write)?;
        self.2.write(write)?;

        Ok(())
    }
}

impl<T0, T1, T2, T3> Parcel for (T0, T1, T2, T3)
    where T0: Parcel, T1: Parcel, T2: Parcel, T3: Parcel
{
    const TYPE_NAME: &'static str = "(T0, T1, T2, T3)";

    fn read(read: &mut Read) -> Result<Self, Error> {
        let v0 = T0::read(read)?;
        let v1 = T1::read(read)?;
        let v2 = T2::read(read)?;
        let v3 = T3::read(read)?;
        Ok((v0, v1, v2, v3))
    }

    fn write(&self, write: &mut Write) -> Result<(), Error> {
        self.0.write(write)?;
        self.1.write(write)?;
        self.2.write(write)?;
        self.3.write(write)?;

        Ok(())
    }
}

impl<T0, T1, T2, T3, T4> Parcel for (T0, T1, T2, T3, T4)
    where T0: Parcel, T1: Parcel, T2: Parcel, T3: Parcel, T4: Parcel
{
    const TYPE_NAME: &'static str = "(T0, T1, T2, T3, T4)";

    fn read(read: &mut Read) -> Result<Self, Error> {
        let v0 = T0::read(read)?;
        let v1 = T1::read(read)?;
        let v2 = T2::read(read)?;
        let v3 = T3::read(read)?;
        let v4 = T4::read(read)?;
        Ok((v0, v1, v2, v3, v4))
    }

    fn write(&self, write: &mut Write) -> Result<(), Error> {
        self.0.write(write)?;
        self.1.write(write)?;
        self.2.write(write)?;
        self.3.write(write)?;
        self.4.write(write)?;

        Ok(())
    }
}

