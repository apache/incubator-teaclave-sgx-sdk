extern crate erased_serde;
extern crate serde_json;
extern crate serde_cbor;

use std::collections::BTreeMap as Map;
use std::io;

use erased_serde::{Serialize, Serializer, Deserializer};

#[test]
fn serialization() {
    // Construct some serializers.
    let json = &mut serde_json::ser::Serializer::new(io::stdout());
    let cbor = &mut serde_cbor::ser::Serializer::new(io::stdout());

    // The values in this map are boxed trait objects. Ordinarily this would not
    // be possible with serde::Serializer because of object safety, but type
    // erasure makes it possible with erased_serde::Serializer.
    let mut formats: Map<&str, Box<Serializer>> = Map::new();
    formats.insert("json", Box::new(Serializer::erase(json)));
    formats.insert("cbor", Box::new(Serializer::erase(cbor)));

    // These are boxed trait objects as well. Same thing here - type erasure
    // makes this possible.
    let mut values: Map<&str, Box<Serialize>> = Map::new();
    values.insert("vec", Box::new(vec!["a", "b"]));
    values.insert("int", Box::new(65536));

    // Pick a Serializer out of the formats map.
    let format = formats.get_mut("json").unwrap();

    // Pick a Serialize out of the values map.
    let value = values.get("vec").unwrap();

    // This line prints `["a","b"]` to stdout.
    value.erased_serialize(format).unwrap();
}

#[test]
fn deserialization() {
    static JSON: &'static [u8] = br#"{"A": 65, "B": 66}"#;
    static CBOR: &'static [u8] = &[162, 97, 65, 24, 65, 97, 66, 24, 66];

    // Construct some deserializers.
    let json = &mut serde_json::de::Deserializer::from_slice(JSON);
    let cbor = &mut serde_cbor::de::Deserializer::from_slice(CBOR);

    // The values in this map are boxed trait objects, which is not possible
    // with the normal serde::Deserializer because of object safety.
    let mut formats: Map<&str, Box<Deserializer>> = Map::new();
    formats.insert("json", Box::new(Deserializer::erase(json)));
    formats.insert("cbor", Box::new(Deserializer::erase(cbor)));

    // Pick a Deserializer out of the formats map.
    let format = formats.get_mut("json").unwrap();

    let data: Map<String, usize> = erased_serde::deserialize(format).unwrap();

    println!("{}", data["A"] + data["B"]);
}
