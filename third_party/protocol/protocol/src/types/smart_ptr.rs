use {Parcel, Error};

use std::prelude::v1::*;
use std::rc::Rc;
use std::sync::Arc;
use std::ops::Deref;
use std::io::prelude::*;

macro_rules! impl_smart_ptr_type {
    ($ty:ident) => {
        impl<T: Parcel> Parcel for $ty<T>
        {
            const TYPE_NAME: &'static str = stringify!($ty<T>);

            fn read(read: &mut dyn Read) -> Result<Self, Error> {
                let value = T::read(read)?;
                Ok($ty::new(value))
            }

            fn write(&self, write: &mut dyn Write) -> Result<(), Error> {
                self.deref().write(write)
            }
        }
    }
}

impl_smart_ptr_type!(Rc);
impl_smart_ptr_type!(Arc);

