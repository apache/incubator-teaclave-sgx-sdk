#[macro_use]
extern crate json;

use json::number::Number;
use json::{ parse, JsonValue, Null };

#[test]
fn parse_true() {
    assert_eq!(parse("true").unwrap(), true);
}

#[test]
fn parse_false() {
    assert_eq!(parse("false").unwrap(), false);
}

#[test]
fn parse_null() {
    assert!(parse("null").unwrap().is_null());
}

#[test]
fn parse_number() {
    assert_eq!(parse("3.141592653589793").unwrap(), 3.141592653589793);
}

#[test]
fn uncode_identifier() {
    assert!(parse("[C3A9] <=> [é]").is_err());
}

#[test]
fn parse_period_requires_digit() {
    assert!(parse("[1.]").is_err());
}

#[test]
fn parse_small_number() {
    assert_eq!(parse("0.05").unwrap(), 0.05);
}

#[test]
fn parse_very_long_float() {
    let parsed = parse("2.22507385850720113605740979670913197593481954635164564e-308").unwrap();

    // Handles convertion correctly
    assert_eq!(parsed.as_f64().unwrap(), 2.225073858507201e-308);

    // Exhausts u64
    assert_eq!(parsed, unsafe { Number::from_parts_unchecked(true, 2225073858507201136, -326) });
}

#[test]
fn parse_very_long_exponent() {
    let parsed = parse("1e999999999999999999999999999999999999999999999999999999999999").unwrap();

    assert_eq!(parsed, unsafe { Number::from_parts_unchecked(true, 1, 32767) });
}

#[test]
fn parse_integer() {
    assert_eq!(parse("42").unwrap(), 42);
}

#[test]
fn parse_negative_zero() {
    assert_eq!(parse("-0").unwrap(), JsonValue::from(-0f64));
}

#[test]
fn parse_negative_integer() {
    assert_eq!(parse("-42").unwrap(), -42);
}

#[test]
fn parse_number_with_leading_zero() {
    assert!(parse("01").is_err());
}

#[test]
fn parse_negative_number_with_leading_zero() {
    assert!(parse("-01").is_err());
}

#[test]
fn parse_number_with_e() {
    assert_eq!(parse("5e2").unwrap(), 500);
    assert_eq!(parse("5E2").unwrap(), 500);
}

#[test]
fn parse_number_with_positive_e() {
    assert_eq!(parse("5e+2").unwrap(), 500);
    assert_eq!(parse("5E+2").unwrap(), 500);
}

#[test]
fn parse_number_with_negative_e() {
    assert_eq!(parse("5e-2").unwrap(), 0.05);
    assert_eq!(parse("5E-2").unwrap(), 0.05);
}

#[test]
fn parse_number_with_invalid_e() {
    assert!(parse("0e").is_err());
}

#[test]
fn parse_large_number() {
    assert_eq!(parse("18446744073709551616").unwrap(), 18446744073709552000f64);
}

#[test]
fn parse_array() {
    assert_eq!(parse(r#"[10, "foo", true, null]"#).unwrap(), array![
        10,
        "foo",
        true,
        Null
    ]);

    assert_eq!(parse("[]").unwrap(), array![]);

    assert_eq!(parse("[[]]").unwrap(), array![array![]]);
}

#[test]
fn parse_object() {
    // Without trailing comma
    assert_eq!(parse(r#"

    {
        "foo": "bar",
        "num": 10
    }

    "#).unwrap(), object!{
        "foo" => "bar",
        "num" => 10
    });

    // Trailing comma in macro
    assert_eq!(parse(r#"

    {
        "foo": "bar",
        "num": 10
    }

    "#).unwrap(), object!{
        "foo" => "bar",
        "num" => 10,
    });
}

#[test]
fn parse_object_duplicate_fields() {
    assert_eq!(parse(r#"

    {
        "foo": 0,
        "bar": 1,
        "foo": 2
    }

    "#).unwrap(), object!{
        "foo" => 2,
        "bar" => 1
    });
}

#[test]
fn parse_object_with_array(){
    assert_eq!(parse(r#"

    {
        "foo": [1, 2, 3]
    }

    "#).unwrap(), object!{
        "foo" => array![1, 2, 3]
    });
}

#[test]
fn parse_nested_object() {
    assert_eq!(parse(r#"

    {
        "l10n": [ {
            "product": {
                "inStock": {
                    "DE": "Lieferung innerhalb von 1-3 Werktagen"
                }
            }
        } ]
    }

    "#).unwrap(), object!{
        "l10n" => array![ object!{
            "product" => object!{
                "inStock" => object!{
                    "DE" => "Lieferung innerhalb von 1-3 Werktagen"
                }
            }
        } ]
    });
}

#[test]
fn parse_and_index_from_object() {
    let data = parse("{ \"pi\": 3.14 }").unwrap();
    let ref pi = data["pi"];

    assert_eq!(pi, 3.14);
}

#[test]
fn parse_and_index_mut_from_object() {
    let mut data = parse(r#"

    {
        "foo": 100
    }

    "#).unwrap();

    assert_eq!(data["foo"], 100);

    data["foo"] = 200.into();

    assert_eq!(data["foo"], 200);
}

#[test]
fn parse_and_index_mut_from_null() {
    let mut data = parse("null").unwrap();

    assert!(data["foo"]["bar"].is_null());

    // test that data didn't coerece to object
    assert!(data.is_null());

    data["foo"]["bar"] = 100.into();

    assert!(data.is_object());
    assert_eq!(data["foo"]["bar"], 100);

    assert_eq!(data.dump(), r#"{"foo":{"bar":100}}"#);
}

#[test]
fn parse_and_index_from_array() {
    let data = parse(r#"[100, 200, false, null, "foo"]"#).unwrap();

    assert_eq!(data[0], Number::from(100));
    assert_eq!(data[1], 200);
    assert_eq!(data[2], false);
    assert_eq!(data[3], Null);
    assert_eq!(data[4], "foo");
    assert_eq!(data[5], Null);
}

#[test]
fn parse_and_index_mut_from_array() {
    let mut data = parse(r#"[100, 200, false, null, "foo"]"#).unwrap();

    assert!(data[3].is_null());
    assert!(data[5].is_null());

    data[3] = "modified".into();
    data[5] = "implicid push".into();

    assert_eq!(data[3], "modified");
    assert_eq!(data[5], "implicid push");
}

#[test]
fn parse_escaped_characters() {
    let data = parse(r#"

    "\r\n\t\b\f\\\/\""

    "#).unwrap();

    assert!(data.is_string());
    assert_eq!(data, "\r\n\t\u{8}\u{c}\\/\"");
}

#[test]
fn parse_escaped_unicode() {
    let data = parse(r#"

    "\u2764\ufe0f"

    "#).unwrap();

    assert_eq!(data, "❤️");
}

#[test]
fn parse_escaped_unicode_surrogate() {
    let data = parse(r#"

    "\uD834\uDD1E"

    "#).unwrap();

    assert_eq!(data, "𝄞");
}

#[test]
fn parse_escaped_unicode_surrogate_fail() {
    let err = parse(r#"

    "\uD834 \uDD1E"

    "#);

    assert!(err.is_err());
}

#[test]
fn parse_deeply_nested_arrays_and_objects() {
    let depth = 256;
    let mut text = String::new();
    for _ in 0..depth {
        text.push_str("[{\"a\":");
    }
    text.push_str("null");
    for _ in 0..depth {
        text.push_str("}]");
    }

    parse(&text).unwrap();

    assert!(true);
}

#[test]
fn parse_error_after_depth_limit() {
    let depth = 5000;
    let mut text = String::new();
    for _ in 0..depth {
        text.push_str("[{\"a\":");
    }
    text.push_str("null");
    for _ in 0..depth {
        text.push_str("}]");
    }

    assert_eq!(parse(&text), Err(json::Error::ExceededDepthLimit));
}

#[test]
fn does_not_panic_on_single_unicode_char() {
    let bytes = vec![0xC3, 0xA5];

    let string = String::from_utf8(bytes).unwrap();

    assert!(parse(&string).is_err());
}

#[test]
fn does_not_panic_on_single_zero() {
    let source = "0";

    parse(source).unwrap();
}

#[test]
fn does_not_panic_on_huge_numbers() {
    let mut string = String::from("8");

    for _ in 1..32787 {
        string.push('0');
    }

    let _ = json::parse(&string);
}
