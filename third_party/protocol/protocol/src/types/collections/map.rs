use {Parcel, Error};

use std::prelude::v1::*;
use std::collections::{HashMap, BTreeMap};
use std::hash::Hash;

use std::io::prelude::*;

pub type SizeType = u32;

macro_rules! impl_map_type {
    ( $ty:ident => K: $( $k_pred:ident ),+ ) => {
        impl<K, V> Parcel for $ty<K, V>
            where K: Parcel + $( $k_pred +)+,
                  V: Parcel
        {
            const TYPE_NAME: &'static str = stringify!($ty<K,V>);

            fn read(read: &mut dyn Read) -> Result<Self, Error> {
                let mut map = $ty::new();

                let length = SizeType::read(read)?;

                for _ in 0..length {
                    let key = K::read(read)?;
                    let value = V::read(read)?;

                    map.insert(key, value);
                }

                Ok(map)
            }

            fn write(&self, write: &mut dyn Write) -> Result<(), Error> {
                (self.len() as SizeType).write(write)?;

                for (key, value) in self.iter() {
                    key.write(write)?;
                    value.write(write)?;
                }

                Ok(())
            }
        }
    }
}

impl_map_type!(HashMap => K: Hash, Eq);
impl_map_type!(BTreeMap => K: Ord);

