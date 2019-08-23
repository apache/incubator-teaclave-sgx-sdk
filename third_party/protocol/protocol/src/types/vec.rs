use {Parcel, Error};
use types;
use std::io::prelude::*;
use std;

/// A newtype wrapping `Vec<T>` but with a custom length prefix type.
#[derive(Clone, Debug, PartialEq)]
pub struct Vec<S: types::Integer, T: Parcel>
{
    /// The inner `Vec<T>`.
    pub elements: std::vec::Vec<T>,
    _a: std::marker::PhantomData<S>,
}

impl<S: types::Integer, T: Parcel> Vec<S,T>
{
    /// Creates a new `Vec` from a list of elements.
    pub fn new(elements: std::vec::Vec<T>) -> Self {
        Vec { elements: elements, _a: std::marker::PhantomData }
    }
}

impl<S: types::Integer, T: Parcel> Parcel for Vec<S, T>
{
    const TYPE_NAME: &'static str = "protocol::Vec<S,T>";

    fn read(read: &mut dyn Read) -> Result<Self, Error> {
        let elements = types::util::read_list_ext::<S,T>(read)?;
        Ok(Self::new(elements))
    }

    fn write(&self, write: &mut dyn Write) -> Result<(), Error> {
        types::util::write_list_ext::<S,T,_>(write, self.elements.iter())
    }
}

/// Stuff relating to `std::vec::Vec<T>`.
mod std_vec {
    use {Error, Parcel};
    use types;
    use std::io::prelude::*;
    use std::prelude::v1::*;

    impl<T: Parcel> Parcel for Vec<T>
    {
        const TYPE_NAME: &'static str = "Vec<T>";

        fn read(read: &mut dyn Read) -> Result<Self, Error> {
            types::util::read_list(read)
        }

        fn write(&self, write: &mut dyn Write) -> Result<(), Error> {
            types::util::write_list(write, self.iter())
        }
    }
}

