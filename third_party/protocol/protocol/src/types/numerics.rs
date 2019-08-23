use {Parcel, Error, ByteOrder};

use std::prelude::v1::*;
use std::io::prelude::*;

use num_traits::{FromPrimitive, ToPrimitive};
use byteorder::{ReadBytesExt, WriteBytesExt};

/// An integer value that can be serialized and deserialized.
pub trait Integer : Parcel + FromPrimitive + ToPrimitive { }

impl Parcel for bool
{
    const TYPE_NAME: &'static str = "bool";

    fn read(read: &mut dyn Read) -> Result<Self, Error> {
        if read.read_u8()? == 0 { Ok(false) } else { Ok(true) }
    }

    fn write(&self, write: &mut dyn Write) -> Result<(), Error> {
        write.write_u8(if *self { 1 } else { 0 })?;
        Ok(())
    }
}

impl Parcel for u8
{
    const TYPE_NAME: &'static str = "u8";

    fn read(read: &mut dyn Read) -> Result<Self, Error> { Ok(read.read_u8()?) }
    fn write(&self, write: &mut dyn Write) -> Result<(), Error> { write.write_u8(*self)?; Ok(()) }
}

impl Parcel for i8
{
    const TYPE_NAME: &'static str = "i8";

    fn read(read: &mut dyn Read) -> Result<Self, Error> { Ok(read.read_i8()?) }
    fn write(&self, write: &mut dyn Write) -> Result<(), Error> { write.write_i8(*self)?; Ok(()) }
}

impl Parcel for u16
{
    const TYPE_NAME: &'static str = "u16";

    fn read(read: &mut dyn Read) -> Result<Self, Error> { Ok(read.read_u16::<ByteOrder>()?) }
    fn write(&self, write: &mut dyn Write) -> Result<(), Error> { write.write_u16::<ByteOrder>(*self)?; Ok(()) }
}

impl Parcel for i16
{
    const TYPE_NAME: &'static str = "i16";

    fn read(read: &mut dyn Read) -> Result<Self, Error> { Ok(read.read_i16::<ByteOrder>()?) }
    fn write(&self, write: &mut dyn Write) -> Result<(), Error> { write.write_i16::<ByteOrder>(*self)?; Ok(()) }
}

impl Parcel for u32
{
    const TYPE_NAME: &'static str = "u32";

    fn read(read: &mut dyn Read) -> Result<Self, Error> { Ok(read.read_u32::<ByteOrder>()?) }
    fn write(&self, write: &mut dyn Write) -> Result<(), Error> { write.write_u32::<ByteOrder>(*self)?; Ok(()) }
}

impl Parcel for i32
{
    const TYPE_NAME: &'static str = "i32";

    fn read(read: &mut dyn Read) -> Result<Self, Error> { Ok(read.read_i32::<ByteOrder>()?) }
    fn write(&self, write: &mut dyn Write) -> Result<(), Error> { write.write_i32::<ByteOrder>(*self)?; Ok(()) }
}

impl Parcel for u64
{
    const TYPE_NAME: &'static str = "u64";

    fn read(read: &mut dyn Read) -> Result<Self, Error> { Ok(read.read_u64::<ByteOrder>()?) }
    fn write(&self, write: &mut dyn Write) -> Result<(), Error> { write.write_u64::<ByteOrder>(*self)?; Ok(()) }
}

impl Parcel for i64
{
    const TYPE_NAME: &'static str = "i64";

    fn read(read: &mut dyn Read) -> Result<Self, Error> { Ok(read.read_i64::<ByteOrder>()?) }
    fn write(&self, write: &mut dyn Write) -> Result<(), Error> { write.write_i64::<ByteOrder>(*self)?; Ok(()) }
}

impl Parcel for f32
{
    const TYPE_NAME: &'static str = "f32";

    fn read(read: &mut dyn Read) -> Result<Self, Error> { Ok(read.read_f32::<ByteOrder>()?) }
    fn write(&self, write: &mut dyn Write) -> Result<(), Error> { write.write_f32::<ByteOrder>(*self)?; Ok(()) }
}

impl Parcel for f64
{
    const TYPE_NAME: &'static str = "f64";

    fn read(read: &mut dyn Read) -> Result<Self, Error> { Ok(read.read_f64::<ByteOrder>()?) }
    fn write(&self, write: &mut dyn Write) -> Result<(), Error> { write.write_f64::<ByteOrder>(*self)?; Ok(()) }
}

impl Integer for u8 { }
impl Integer for i8 { }
impl Integer for u16 { }
impl Integer for i16 { }
impl Integer for u32 { }
impl Integer for i32 { }
impl Integer for u64 { }
impl Integer for i64 { }

