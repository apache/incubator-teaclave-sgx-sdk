extern crate serde_cbor;
extern crate serde_bytes;

use serde_bytes::ByteBuf;
use std::collections::BTreeMap;

use serde_cbor::{to_vec, Value, ObjectKey, error, de, Deserializer, from_reader};

#[test]
fn test_string1() {
    let value: error::Result<Value> = de::from_slice(&[0x66, 0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72]);
    assert_eq!(value.unwrap(), Value::String("foobar".to_owned()));
}

#[test]
fn test_string2() {
    let value: error::Result<Value> = de::from_slice(&[0x71, 0x49, 0x20, 0x6d, 0x65, 0x74, 0x20, 0x61, 0x20, 0x74, 0x72, 0x61, 0x76,
        0x65, 0x6c, 0x6c, 0x65, 0x72]);
    assert_eq!(value.unwrap(), Value::String("I met a traveller".to_owned()));
}

#[test]
fn test_string3() {
    let slice = b"\x78\x2fI met a traveller from an antique land who said";
    let value: error::Result<Value> = de::from_slice(slice);
    assert_eq!(value.unwrap(), Value::String("I met a traveller from an antique land who said".to_owned()));
}

#[test]
fn test_byte_string() {
    let value: error::Result<Value> = de::from_slice(&[0x46, 0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72]);
    assert_eq!(value.unwrap(), Value::Bytes(b"foobar".to_vec()));
}

#[test]
fn test_numbers1() {
    let value: error::Result<Value> = de::from_slice(&[0x00]);
    assert_eq!(value.unwrap(), Value::U64(0));
}

#[test]
fn test_numbers2() {
    let value: error::Result<Value> = de::from_slice(&[0x1a, 0x00, 0xbc, 0x61, 0x4e]);
    assert_eq!(value.unwrap(), Value::U64(12345678));
}

#[test]
fn test_numbers3() {
    let value: error::Result<Value> = de::from_slice(&[0x39, 0x07, 0xde]);
    assert_eq!(value.unwrap(), Value::I64(-2015));
}

#[test]
fn test_bool() {
    let value: error::Result<Value> = de::from_slice(b"\xf4");
    assert_eq!(value.unwrap(), Value::Bool(false));
}

#[test]
fn test_trailing_bytes() {
    let value: error::Result<Value> = de::from_slice(b"\xf4trailing");
    assert!(value.is_err());
}

#[test]
fn test_list1() {
    let value: error::Result<Value> = de::from_slice(b"\x83\x01\x02\x03");
    assert_eq!(value.unwrap(), Value::Array(vec![Value::U64(1), Value::U64(2), Value::U64(3)]));
}

#[test]
fn test_list2() {
    let value: error::Result<Value> = de::from_slice(b"\x82\x01\x82\x02\x81\x03");
    assert_eq!(value.unwrap(), Value::Array(vec![Value::U64(1), Value::Array(vec![Value::U64(2), Value::Array(vec![Value::U64(3)])])]));
}

#[test]
fn test_object() {
    let value: error::Result<Value> = de::from_slice(b"\xa5aaaAabaBacaCadaDaeaE");
    let mut object = BTreeMap::new();
    object.insert(ObjectKey::String("a".to_owned()), Value::String("A".to_owned()));
    object.insert(ObjectKey::String("b".to_owned()), Value::String("B".to_owned()));
    object.insert(ObjectKey::String("c".to_owned()), Value::String("C".to_owned()));
    object.insert(ObjectKey::String("d".to_owned()), Value::String("D".to_owned()));
    object.insert(ObjectKey::String("e".to_owned()), Value::String("E".to_owned()));
    assert_eq!(value.unwrap(), Value::Object(object));
}

#[test]
fn test_indefinite_object() {
    let value: error::Result<Value> = de::from_slice(b"\xbfaa\x01ab\x9f\x02\x03\xff\xff");
    let mut object = BTreeMap::new();
    object.insert(ObjectKey::String("a".to_owned()), Value::U64(1));
    object.insert(ObjectKey::String("b".to_owned()), Value::Array(vec![Value::U64(2), Value::U64(3)]));
    assert_eq!(value.unwrap(), Value::Object(object));
}

#[test]
fn test_indefinite_list() {
    let value: error::Result<Value> = de::from_slice(b"\x9f\x01\x02\x03\xff");
    assert_eq!(value.unwrap(), Value::Array(vec![Value::U64(1), Value::U64(2), Value::U64(3)]));
}

#[test]
fn test_indefinite_string() {
    let value: error::Result<Value> = de::from_slice(b"\x7f\x65Mary \x64Had \x62a \x67Little \x64Lamb\xff");
    assert_eq!(value.unwrap(), Value::String("Mary Had a Little Lamb".to_owned()));
}

#[test]
fn test_indefinite_byte_string() {
    let value: error::Result<Value> = de::from_slice(b"\x5f\x42\x01\x23\x42\x45\x67\xff");
    assert_eq!(value.unwrap(), Value::Bytes(b"\x01#Eg".to_vec()));
}

#[test]
fn test_float() {
    let value: error::Result<Value> = de::from_slice(b"\xfa\x47\xc3\x50\x00");
    assert_eq!(value.unwrap(), Value::F64(100000.0));
}


#[test]
fn test_self_describing() {
    let value: error::Result<Value> = de::from_slice(&[0xd9, 0xd9, 0xf7, 0x66, 0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72]);
    assert_eq!(value.unwrap(), Value::String("foobar".to_owned()));
}


#[test]
fn test_f16() {
    let mut x: Value = de::from_slice(&[0xf9, 0x41, 0x00]).unwrap();
    assert_eq!(x, Value::F64(2.5));
    x = de::from_slice(&[0xf9, 0x41, 0x90]).unwrap();
    assert_eq!(x, Value::F64(2.78125));
    x = de::from_slice(&[0xf9, 0x50, 0x90]).unwrap();
    assert_eq!(x, Value::F64(36.5));
    x = de::from_slice(&[0xf9, 0xd0, 0x90]).unwrap();
    assert_eq!(x, Value::F64(-36.5));
}

#[test]
fn test_crazy_list() {
    let slice = b"\x88\x1b\x00\x00\x00\x1c\xbe\x99\x1d\xc7\x3b\x00\x7a\xcf\x51\xdc\x51\x70\xdb\x3a\x1b\x3a\x06\xdd\xf5\xf6\xf7\xfb\x41\x76\x5e\xb1\xf8\x00\x00\x00\xf9\x7c\x00";
    let value: Vec<Value> = de::from_slice(slice).unwrap();
    assert_eq!(value, vec![
        Value::U64(123456789959),
        Value::I64(-34567897654325468),
        Value::I64(-456787678),
        Value::Bool(true),
        Value::Null,
        Value::Null,
        Value::F64(23456543.5),
        Value::F64(::std::f64::INFINITY)]);
}

#[test]
fn test_nan() {
    let value: f64 = de::from_slice(b"\xf9\x7e\x00").unwrap();
    assert!(value.is_nan());
}

#[test]
fn test_32f16() {
    let value: f32 = de::from_slice(b"\xf9\x50\x00").unwrap();
    assert_eq!(value, 32.0f32);
}

#[test]
// The file was reported as not working by user kie0tauB
// but it parses to a cbor value.
fn test_kietaub_file() {
    let file = include_bytes!("kietaub.cbor");
    let value_result: error::Result<Value> = de::from_slice(file);
    value_result.unwrap();
}

#[test]
fn test_option_roundtrip() {
    let obj1 = Some(10u32);

    let v = to_vec(&obj1).unwrap();
    let obj2: Result<Option<u32>, _> = serde_cbor::de::from_reader(&v[..]);
    println!("{:?}", obj2);

    assert_eq!(obj1, obj2.unwrap());
}

#[test]
fn test_option_none_roundtrip() {
    let obj1 = None;

    let v = to_vec(&obj1).unwrap();
    println!("{:?}", v);
    let obj2: Result<Option<u32>, _> = serde_cbor::de::from_reader(&v[..]);

    assert_eq!(obj1, obj2.unwrap());
}

#[test]
fn test_variable_length_map() {
    let slice = b"\xbf\x67\x6d\x65\x73\x73\x61\x67\x65\x64\x70\x6f\x6e\x67\xff";
    let value: Value = de::from_slice(slice).unwrap();
    let mut map = BTreeMap::new();
    map.insert(ObjectKey::String("message".to_string()), Value::String("pong".to_string()));
    assert_eq!(value, Value::Object(map))
}

#[test]
fn test_object_determinism_roundtrip() {
    let expected = b"\xa2aa\x01ab\x82\x02\x03";

    // 0.1% chance of not catching failure
    for _ in 0..10 {
        assert_eq!(&to_vec(&de::from_slice::<Value>(expected).unwrap()).unwrap(), expected);
    }
}

#[test]
fn stream_deserializer() {
    let slice = b"\x01\x66foobar";
    let mut it = Deserializer::from_slice(slice).into_iter::<Value>();
    assert_eq!(Value::U64(1), it.next().unwrap().unwrap());
    assert_eq!(Value::String("foobar".to_string()), it.next().unwrap().unwrap());
    assert!(it.next().is_none());
}

#[test]
fn stream_deserializer_eof() {
    let slice = b"\x01\x66foob";
    let mut it = Deserializer::from_slice(slice).into_iter::<Value>();
    assert_eq!(Value::U64(1), it.next().unwrap().unwrap());
    assert!(it.next().unwrap().unwrap_err().is_eof());
}

#[test]
fn test_large_bytes() {
    let expected = (0..2 * 1024 * 1024).map(|i| (i * 7) as u8).collect::<Vec<_>>();
    let expected = ByteBuf::from(expected);
    let v = to_vec(&expected).unwrap();

    let actual = from_reader(&v[..]).unwrap();
    assert_eq!(expected, actual);
}
