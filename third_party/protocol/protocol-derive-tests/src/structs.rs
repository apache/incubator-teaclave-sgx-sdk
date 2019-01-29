#[allow(unused_imports)]
use protocol::Parcel;

#[derive(Protocol, Debug, PartialEq, Eq)]
pub struct Foobar {
    a: u8,
    b: u8,
    c: u8,
}

#[derive(Protocol, Debug, PartialEq, Eq)]
pub struct BizBong(u8, u8, pub u8);

#[derive(Protocol, Debug, PartialEq, Eq)]
pub struct PartyInTheFront;

#[derive(Protocol, Debug, PartialEq, Eq)]
pub struct NamedFieldsWithGenerics<A,D> {
    pub value: A,
    pub del: D,
}

#[derive(Protocol, Debug, PartialEq, Eq)]
pub struct UnnamedFieldsWithGenerics<A,D>(A, D);

#[derive(Protocol, Debug, PartialEq, Eq)]
pub struct StructWithExistingBoundedGenerics<A: ::std::fmt::Display + ::std::fmt::Debug +> {
    foo: A,
}

#[test]
fn named_fields_are_correctly_written() {
    assert_eq!(vec![3, '2' as u8, 1], Foobar {
        a: 3,
        b: '2' as u8,
        c: 1,
    }.raw_bytes().unwrap());
}

#[test]
fn named_fields_are_correctly_read() {
    assert_eq!(Foobar {
        a: 3,
        b: '2' as u8,
        c: 1,
    }, Foobar::from_raw_bytes(&[3, '2' as u8, 1]).unwrap());
}

#[test]
fn unnamed_fields_are_correctly_written() {
    assert_eq!(vec![6, 1, 9], BizBong(6,1,9).raw_bytes().unwrap());
}

#[test]
fn unnamed_fields_are_correctly_read() {
    assert_eq!(BizBong(3, 1, 7), BizBong::from_raw_bytes(&[3, 1, 7]).unwrap());
}

#[test]
fn unit_structs_are_correctly_written() {
    assert_eq!(PartyInTheFront.raw_bytes().unwrap(), &[]);
}

#[test]
fn unit_structs_are_correctly_read() {
    assert_eq!(PartyInTheFront, PartyInTheFront::from_raw_bytes(&[]).unwrap());
}

#[test]
fn type_name_is_correct() {
    assert_eq!("PartyInTheFront", PartyInTheFront.type_name());
    assert_eq!("BizBong", BizBong(2,3,1).type_name());
}

