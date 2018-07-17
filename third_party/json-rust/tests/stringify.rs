#[macro_use]
extern crate json;

use std::collections::{ HashMap, BTreeMap };
use std::f64;
use json::{ parse, stringify, stringify_pretty, JsonValue, Null };

#[test]
fn stringify_null() {
    assert_eq!(stringify(Null), "null");
}

#[test]
fn stringify_option_none() {
    let foo: Option<String> = None;
    assert_eq!(stringify(foo), "null");
}

#[test]
fn stringify_option_integer() {
    let foo = Some(100);
    assert_eq!(stringify(foo), "100");
}

#[test]
fn stringify_str_slice() {
    assert_eq!(stringify("Foo"), "\"Foo\"");
}

#[test]
fn stringify_string() {
    assert_eq!(stringify("Foo".to_string()), "\"Foo\"");
}

#[test]
fn stringify_number() {
    assert_eq!(stringify(3.141592653589793), "3.141592653589793");
}

#[test]
fn stringify_precise_positive_number() {
    assert_eq!(JsonValue::from(1.2345f64).dump(), "1.2345");
}

#[test]
fn stringify_precise_negative_number() {
    assert_eq!(JsonValue::from(-1.2345f64).dump(), "-1.2345");
}

#[test]
fn stringify_zero() {
    assert_eq!(JsonValue::from(0.0).dump(), "0");
}

#[test]
fn stringify_nan() {
    assert_eq!(JsonValue::from(f64::NAN).dump(), "null");
}

#[test]
fn stringify_infinity() {
    assert_eq!(JsonValue::from(f64::INFINITY).dump(), "null");
    assert_eq!(JsonValue::from(f64::NEG_INFINITY).dump(), "null");
}

#[test]
fn stringify_negative_zero() {
    assert_eq!(JsonValue::from(-0f64).dump(), "-0");
}

#[test]
fn stringify_integer() {
    assert_eq!(stringify(42), "42");
}

#[test]
fn stringify_small_number() {
    assert_eq!(stringify(0.0001), "0.0001");
}

#[test]
fn stringify_large_number() {
    assert_eq!(stringify(1e19), "10000000000000000000");
}

#[test]
fn stringify_very_large_number() {
    assert_eq!(stringify(3.141592653589793e50), "3.141592653589793e50");
}

#[test]
fn stringify_very_large_number_no_fraction() {
    assert_eq!(stringify(7e70), "7e70");
}

#[test]
fn stringify_very_small_number() {
    assert_eq!(stringify(3.141592653589793e-16), "3.141592653589793e-16");
}

#[test]
fn stringify_true() {
    assert_eq!(stringify(true), "true");
}

#[test]
fn stringify_false() {
    assert_eq!(stringify(false), "false");
}

#[test]
fn stringify_array() {
    assert_eq!(stringify(array![10, false, Null]), "[10,false,null]");
}

#[test]
fn stringify_vec() {
    let mut array: Vec<JsonValue> = Vec::new();

    array.push(10.into());
    array.push("Foo".into());

    assert_eq!(stringify(array), r#"[10,"Foo"]"#);
}

#[test]
fn stringify_typed_vec() {
    let array = vec![1, 2, 3];

    assert_eq!(stringify(array), "[1,2,3]");
}

#[test]
fn stringify_typed_opt_vec() {
    let array = vec![Some(1), None, Some(2), None, Some(3)];

    assert_eq!(stringify(array), "[1,null,2,null,3]");
}

#[test]
fn stringify_object() {
    let object = object!{
        "name" => "Maciej",
        "age" => 30
    };

    assert_eq!(object.dump(), r#"{"name":"Maciej","age":30}"#);
    assert_eq!(stringify(object), r#"{"name":"Maciej","age":30}"#);
}

#[test]
fn stringify_raw_object() {
    let mut object = json::object::Object::new();

    object.insert("name", "Maciej".into());
    object.insert("age", 30.into());

    assert_eq!(object.dump(), r#"{"name":"Maciej","age":30}"#);
    assert_eq!(stringify(object), r#"{"name":"Maciej","age":30}"#);
}

#[test]
fn stringify_btree_map() {
    let mut map = BTreeMap::new();

    map.insert("name".into(), "Maciej".into());
    map.insert("age".into(), 30.into());

    // BTreeMap will sort keys
    assert_eq!(stringify(map), r#"{"age":30,"name":"Maciej"}"#);
}

#[test]
fn stringify_hash_map() {
    let mut map = HashMap::new();

    map.insert("name".into(), "Maciej".into());
    map.insert("age".into(), 30.into());

    // HashMap does not sort keys, but depending on hashing used the
    // order can be different. Safe bet is to parse the result and
    // compare parsed objects.
    let parsed = parse(&stringify(map)).unwrap();

    assert_eq!(parsed, object!{
        "name" => "Maciej",
        "age" => 30
    });
}

#[test]
fn stringify_object_with_put() {
    let mut object = JsonValue::new_object();

    object["a"] = 100.into();
    object["b"] = false.into();

    assert_eq!(object.dump(), r#"{"a":100,"b":false}"#);
    assert_eq!(stringify(object), r#"{"a":100,"b":false}"#);
}

#[test]
fn stringify_array_with_push() {
    let mut array = JsonValue::new_array();

    array.push(100).unwrap();
    array.push(Null).unwrap();
    array.push(false).unwrap();
    array.push(Some("foo".to_string())).unwrap();

    assert_eq!(stringify(array), "[100,null,false,\"foo\"]");
}

#[test]
fn stringify_escaped_characters() {
    assert_eq!(stringify("\r____\n___\t\u{8}\u{c}\\\"__"), r#""\r____\n___\t\b\f\\\"__""#);
}

#[test]
fn stringify_dont_escape_forward_slash() {
    assert_eq!(stringify("foo/bar"), r#""foo/bar""#);
}

#[test]
fn stringify_escaped() {
    assert_eq!(stringify("http://www.google.com/\t"), r#""http://www.google.com/\t""#);
}

#[test]
fn stringify_control_escaped() {
    assert_eq!(stringify("foo\u{1f}bar\u{0}baz"), r#""foo\u001fbar\u0000baz""#);
}

#[test]
fn stringify_pretty_object() {
    let object = object!{
        "name" => "Urlich",
        "age" => 50,
        "parents" => object!{
            "mother" => "Helga",
            "father" => "Brutus"
        },
        "cars" => array![ "Golf", "Mercedes", "Porsche" ]
    };

    let expected = "{\n  \"name\": \"Urlich\",\n  \"age\": 50,\n  \"parents\": {\n    \"mother\": \"Helga\",\n    \"father\": \"Brutus\"\n  },\n  \"cars\": [\n    \"Golf\",\n    \"Mercedes\",\n    \"Porsche\"\n  ]\n}";
    assert_eq!(object.pretty(2), expected);
    assert_eq!(stringify_pretty(object, 2), expected);
}
