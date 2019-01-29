extern crate protocol;
#[macro_use] extern crate protocol_derive;

mod enums;
mod structs;

#[cfg(test)]
use protocol::Parcel;

#[cfg(test)]
fn verify_read_back<P: Parcel + ::std::fmt::Debug + ::std::cmp::PartialEq>(parcel: P) {
    let read_back = P::from_raw_bytes(&parcel.raw_bytes().unwrap()[..]).unwrap();
    assert_eq!(parcel, read_back);
}

