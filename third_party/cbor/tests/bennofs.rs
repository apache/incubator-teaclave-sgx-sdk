extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_cbor;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Example {
    foo: Foo,
    payload: u8,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Foo {
    x: u8,
    color: Color,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Color {
    Red,
    Blue,
    Yellow(u8),
}

#[test]
fn test() {
    let foo = Foo {
        x: 0xAA,
        color: Color::Yellow(40),
    };
    let example = Example {
        foo: foo,
        payload: 0xCC,
    };
    let serialized = serde_cbor::ser::to_vec_packed(&example).unwrap();
    let deserialized: Example = serde_cbor::from_slice(&serialized).unwrap();
    assert_eq!(example, deserialized);
}

