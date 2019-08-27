#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

bitflags! {
    #[derive(Serialize, Deserialize)]
    struct Flags: u32 {
        const A = 1;
        const B = 2;
        const C = 4;
        const D = 8;
    }
}

#[test]
fn serialize() {
    let flags = Flags::A | Flags::B;

    let serialized = serde_json::to_string(&flags).unwrap();

    assert_eq!(serialized, r#"{"bits":3}"#);
}

#[test]
fn deserialize() {
    let deserialized: Flags = serde_json::from_str(r#"{"bits":12}"#).unwrap();

    let expected = Flags::C | Flags::D;

    assert_eq!(deserialized.bits, expected.bits);
}
