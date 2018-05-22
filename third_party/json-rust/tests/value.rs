#[macro_use]
extern crate json;

use json::{ parse, JsonValue, JsonError, Null };

#[test]
fn is_as_string() {
    let string = JsonValue::from("foo");

    assert!(string.is_string());
    assert_eq!(string.as_str().unwrap(), "foo");
}

#[test]
fn is_as_number() {
    let number = JsonValue::from(42);

    assert!(number.is_number());
    assert_eq!(number.as_f64().unwrap(), 42.0f64);
    assert_eq!(number.as_f32().unwrap(), 42.0f32);
    assert_eq!(number.as_u64().unwrap(), 42u64);
    assert_eq!(number.as_u32().unwrap(), 42u32);
    assert_eq!(number.as_u16().unwrap(), 42u16);
    assert_eq!(number.as_u8().unwrap(), 42u8);
    assert_eq!(number.as_usize().unwrap(), 42usize);
    assert_eq!(number.as_i64().unwrap(), 42i64);
    assert_eq!(number.as_i32().unwrap(), 42i32);
    assert_eq!(number.as_i16().unwrap(), 42i16);
    assert_eq!(number.as_i8().unwrap(), 42i8);
    assert_eq!(number.as_isize().unwrap(), 42isize);

    let number = JsonValue::from(-1);

    assert_eq!(number.as_u64(), None);
    assert_eq!(number.as_u32(), None);
    assert_eq!(number.as_u16(), None);
    assert_eq!(number.as_u8(), None);
    assert_eq!(number.as_usize(), None);
    assert_eq!(number.as_i64(), Some(-1));
    assert_eq!(number.as_i32(), Some(-1));
    assert_eq!(number.as_i16(), Some(-1));
    assert_eq!(number.as_i8(), Some(-1));
    assert_eq!(number.as_isize(), Some(-1));

    let number = JsonValue::from(40_000);

    assert_eq!(number.as_u8(), None);
    assert_eq!(number.as_u16(), Some(40_000));
    assert_eq!(number.as_i8(), None);
    assert_eq!(number.as_i16(), None);
    assert_eq!(number.as_i32(), Some(40_000));
}

#[test]
fn as_fixed_point() {
    let number = JsonValue::from(3.14);

    assert_eq!(number.as_fixed_point_u64(4).unwrap(), 31400_u64);
    assert_eq!(number.as_fixed_point_u64(2).unwrap(), 314_u64);
    assert_eq!(number.as_fixed_point_u64(0).unwrap(), 3_u64);

    assert_eq!(number.as_fixed_point_i64(4).unwrap(), 31400_i64);
    assert_eq!(number.as_fixed_point_i64(2).unwrap(), 314_i64);
    assert_eq!(number.as_fixed_point_i64(0).unwrap(), 3_i64);

    let number = JsonValue::from(-3.14);

    assert_eq!(number.as_fixed_point_u64(4), None);
    assert_eq!(number.as_fixed_point_u64(2), None);
    assert_eq!(number.as_fixed_point_u64(0), None);

    assert_eq!(number.as_fixed_point_i64(4).unwrap(), -31400_i64);
    assert_eq!(number.as_fixed_point_i64(2).unwrap(), -314_i64);
    assert_eq!(number.as_fixed_point_i64(0).unwrap(), -3_i64);
}

#[test]
fn is_as_boolean() {
    let boolean = JsonValue::Boolean(true);

    assert!(boolean.is_boolean());
    assert_eq!(boolean.as_bool().unwrap(), true);
}

#[test]
fn is_true() {
    let boolean = JsonValue::Boolean(true);

    assert_eq!(boolean, true);
}

#[test]
fn is_false() {
    let boolean = JsonValue::Boolean(false);

    assert_eq!(boolean, false);
}

#[test]
fn is_null() {
    let null = JsonValue::Null;

    assert!(null.is_null());
}

#[test]
fn is_empty() {
    assert!(Null.is_empty());
    assert!(json::from(0).is_empty());
    assert!(json::from("").is_empty());
    assert!(json::from(false).is_empty());
    assert!(array![].is_empty());
    assert!(object!{}.is_empty());

    assert!(!json::from(1).is_empty());
    assert!(!json::from("foo").is_empty());
    assert!(!json::from(true).is_empty());
    assert!(!array![0].is_empty());
    assert!(!object!{ "foo" => false }.is_empty());
}

#[test]
fn array_len() {
    let data = array![0, 1, 2, 3];

    assert_eq!(data.len(), 4);
}

#[test]
fn array_contains() {
    let data = array![true, Null, 3.14, "foo"];

    assert!(data.contains(true));
    assert!(data.contains(Null));
    assert!(data.contains(3.14));
    assert!(data.contains("foo"));

    assert!(!data.contains(false));
    assert!(!data.contains(42));
    assert!(!data.contains("bar"));
}

#[test]
fn array_push() {
    let mut data = array![1, 2];

    data.push(3).unwrap();

    assert_eq!(data, array![1, 2, 3]);
}

#[test]
fn array_pop() {
    let mut data = array![1, 2, 3];

    assert_eq!(data.pop(), 3);
    assert_eq!(data, array![1, 2]);
}

#[test]
fn array_remove() {
    let mut data = array![1, 2, 3];

    assert_eq!(data.array_remove(1), 2);
    assert_eq!(data, array![1, 3]);
    // Test with index out of bounds
    assert_eq!(data.array_remove(2), JsonValue::Null);
}

#[test]
fn array_members() {
    let data = array![1, "foo"];

    for member in data.members() {
        assert!(!member.is_null());
    }

    let mut members = data.members();

    assert_eq!(members.next().unwrap(), 1);
    assert_eq!(members.next().unwrap(), "foo");
    assert!(members.next().is_none());
}

#[test]
fn array_members_rev() {
    let data = array![1, "foo"];

    for member in data.members() {
        assert!(!member.is_null());
    }

    let mut members = data.members().rev();

    assert_eq!(members.next().unwrap(), "foo");
    assert_eq!(members.next().unwrap(), 1);
    assert!(members.next().is_none());
}

#[test]
fn array_members_mut() {
    let mut data = array![Null, Null];

    for member in data.members_mut() {
        assert!(member.is_null());
        *member = 100.into();
    }

    assert_eq!(data, array![100, 100]);
}

#[test]
fn array_members_mut_rev() {
    let mut data = array![Null, Null];
    let mut item = 100;

    for member in data.members_mut().rev() {
        assert!(member.is_null());
        *member = item.into();
        item += 1;
    }

    assert_eq!(data, array![item - 1, item - 2]);
}

#[test]
fn object_len() {
    let data = object!{
        "a" => true,
        "b" => false
    };

    assert_eq!(data.len(), 2);
}

#[test]
fn object_remove() {
    let mut data = object!{
        "foo" => "bar",
        "answer" => 42
    };

    assert_eq!(data.remove("foo"), "bar");
    assert_eq!(data, object!{ "answer" => 42 });
}

#[test]
fn object_entries() {
    let data = object!{
        "a" => 1,
        "b" => "foo"
    };

    for (_, value) in data.entries() {
        assert!(!value.is_null());
    }

    let mut entries = data.entries();

    let (key, value) = entries.next().unwrap();
    assert_eq!(key, "a");
    assert_eq!(value, 1);

    let (key, value) = entries.next().unwrap();
    assert_eq!(key, "b");
    assert_eq!(value, "foo");

    assert!(entries.next().is_none());
}

#[test]
fn object_entries_rev() {
    let data = object!{
        "a" => 1,
        "b" => "foo"
    };

    for (_, value) in data.entries().rev() {
        assert!(!value.is_null());
    }

    let mut entries = data.entries().rev();

    let (key, value) = entries.next().unwrap();
    assert_eq!(key, "b");
    assert_eq!(value, "foo");

    let (key, value) = entries.next().unwrap();
    assert_eq!(key, "a");
    assert_eq!(value, 1);

    assert!(entries.next().is_none());
}

#[test]
fn object_entries_mut() {
    let mut data = object!{
        "a" => Null,
        "b" => Null
    };

    for (_, value) in data.entries_mut() {
        assert!(value.is_null());
        *value = 100.into();
    }

    assert_eq!(data, object!{
        "a" => 100,
        "b" => 100
    });
}

#[test]
fn object_entries_mut_rev() {
    let mut data = object!{
        "a" => Null,
        "b" => Null
    };
    let mut item = 100;

    for (_, value) in data.entries_mut().rev() {
        assert!(value.is_null());
        *value = item.into();
        item += 1;
    }

    assert_eq!(data, object!{
        "a" => item - 1,
        "b" => item - 2
    });
}

#[test]
fn object_dump_minified() {
    let object = object!{
        "name" => "Maciej",
        "age" => 30
    };

    assert_eq!(object.dump(), "{\"name\":\"Maciej\",\"age\":30}");
}

#[test]
fn object_dump_pretty() {
    let object = object!{
        "name" => "Urlich",
        "age" => 50,
        "parents" => object!{
            "mother" => "Helga",
            "father" => "Brutus"
        },
        "cars" => array![ "Golf", "Mercedes", "Porsche" ]
    };

    assert_eq!(object.pretty(2),
               "{\n  \"name\": \"Urlich\",\n  \"age\": 50,\n  \"parents\": {\n    \"mother\": \"Helga\",\n    \"father\": \"Brutus\"\n  },\n  \"cars\": [\n    \"Golf\",\n    \"Mercedes\",\n    \"Porsche\"\n  ]\n}");
}

#[test]
fn null_len() {
    let data = json::Null;

    assert_eq!(data.len(), 0);
}

#[test]
fn index_by_str() {
    let data = object!{
        "foo" => "bar"
    };

    assert_eq!(data["foo"], "bar");
}

#[test]
fn index_by_string() {
    let data = object!{
        "foo" => "bar"
    };

    assert_eq!(data["foo".to_string()], "bar");
}

#[test]
fn index_by_string_ref() {
    let data = object!{
        "foo" => "bar"
    };

    let key = "foo".to_string();
    let ref key_ref = key;

    assert_eq!(data[key_ref], "bar");
}

#[test]
fn index_mut_by_str() {
    let mut data = object!{
        "foo" => Null
    };

    data["foo"] = "bar".into();

    assert_eq!(data["foo"], "bar");
}

#[test]
fn index_mut_by_string() {
    let mut data = object!{
        "foo" => Null
    };

    data["foo".to_string()] = "bar".into();

    assert_eq!(data["foo"], "bar");
}

#[test]
fn index_mut_by_string_ref() {
    let mut data = object!{
        "foo" => Null
    };

    let key = "foo".to_string();
    let ref key_ref = key;

    data[key_ref] = "bar".into();

    assert_eq!(data["foo"], "bar");
}

#[test]
fn object_index_by_str() {
    let val = object!{
        "foo" => "bar"
    };
    if let JsonValue::Object(data) = val {
        assert_eq!(data["foo"], "bar");
    }
}

#[test]
fn object_index_by_string() {
    let val = object!{
        "foo" => "bar"
    };

    if let JsonValue::Object(data) = val {
        assert_eq!(data["foo".to_string()], "bar");
    }
}

#[test]
fn object_index_by_string_ref() {
    let val = object!{
        "foo" => "bar"
    };

    let key = "foo".to_string();
    let ref key_ref = key;

    if let JsonValue::Object(data) = val {
        assert_eq!(data[key_ref], "bar");
    }
}

#[test]
fn object_index_mut_by_str() {
    let val = object!{
        "foo" => Null
    };

    if let JsonValue::Object(mut data) = val {
        data["foo"] = "bar".into();

        assert_eq!(data["foo"], "bar");
    }
}

#[test]
fn object_index_mut_by_string() {
    let val = object!{
        "foo" => Null
    };

    if let JsonValue::Object(mut data) = val {
        data["foo".to_string()] = "bar".into();

        assert_eq!(data["foo"], "bar");
    }
}

#[test]
fn object_index_mut_by_string_ref() {
    let val = object!{
        "foo" => Null
    };

    let key = "foo".to_string();
    let ref key_ref = key;

    if let JsonValue::Object(mut data) = val {
        data[key_ref] = "bar".into();

        assert_eq!(data["foo"], "bar");
    }
}

#[test]
fn fmt_string() {
    let data: JsonValue = "foobar".into();

    assert_eq!(format!("{}", data), "foobar");
    assert_eq!(format!("{:#}", data), r#""foobar""#);
}

#[test]
fn fmt_number() {
    let data: JsonValue = 42.into();

    assert_eq!(format!("{}", data), "42");
    assert_eq!(format!("{:#}", data), "42");
}

#[test]
fn fmt_boolean() {
    let data: JsonValue = true.into();

    assert_eq!(format!("{}", data), "true");
    assert_eq!(format!("{:#}", data), "true");
}

#[test]
fn fmt_null() {
    let data = Null;

    assert_eq!(format!("{}", data), "null");
    assert_eq!(format!("{:#}", data), "null");
}

#[test]
fn fmt_array() {
    let data = array![1, true, "three"];

    assert_eq!(format!("{}", data), r#"[1,true,"three"]"#);
    assert_eq!(format!("{:#}", data), "[\n    1,\n    true,\n    \"three\"\n]");
}

#[test]
fn fmt_object() {
    let data = object!{
        "foo" => "bar",
        "answer" => 42
    };

    assert_eq!(format!("{}", data), r#"{"foo":"bar","answer":42}"#);
    assert_eq!(format!("{:#}", data), "{\n    \"foo\": \"bar\",\n    \"answer\": 42\n}");
}

#[test]
fn error_unexpected_character() {
    let err = parse("\n\nnulX\n").unwrap_err();

    assert_eq!(err, JsonError::UnexpectedCharacter {
        ch: 'X',
        line: 3,
        column: 4,
    });

    assert_eq!(format!("{}", err), "Unexpected character: X at (3:4)");
}

#[test]
fn error_unexpected_unicode_character() {
    let err = parse("\n\nnul🦄\n").unwrap_err();

    assert_eq!(err, JsonError::UnexpectedCharacter {
        ch: '🦄',
        line: 3,
        column: 4,
    });

    assert_eq!(format!("{}", err), "Unexpected character: 🦄 at (3:4)");
}

#[test]
fn error_unexpected_token() {
    let err = parse("\n  [\n    null,\n  ]  \n").unwrap_err();

    assert_eq!(err, JsonError::UnexpectedCharacter {
        ch: ']',
        line: 4,
        column: 3,
    });

    assert_eq!(format!("{}", err), "Unexpected character: ] at (4:3)");
}

#[test]
fn writer_generator() {
    let data = object!{
        "foo" => array!["bar", 100, true]
    };

    let mut buf = Vec::new();

    data.write(&mut buf).expect("Can't fail with a Vec");

    assert_eq!(String::from_utf8(buf).unwrap(), r#"{"foo":["bar",100,true]}"#);
}

#[test]
fn pretty_writer_generator() {
    let data = object!{
        "foo" => array!["bar", 100, true]
    };

    let mut buf = Vec::new();

    data.write_pretty(&mut buf, 4).expect("Can't fail with a Vec");

    assert_eq!(String::from_utf8(buf).unwrap(), "{\n    \"foo\": [\n        \"bar\",\n        100,\n        true\n    ]\n}");
}

#[test]
fn equality() {
    let left = object!{
        "foo" => array!["bar", 100, true]
    };

    let left_copy = object!{
        "foo" => array!["bar", 100, true]
    };

    let left_string = object!{
        "foo" => array![JsonValue::String("bar".to_string()), 100, true]
    };

    let left_short = object!{
        "foo" => array![JsonValue::Short(unsafe { json::short::Short::from_slice("bar") }), 100, true]
    };

    let change_bool = object!{
        "foo" => array!["bar", 100, false]
    };

    let change_string = object!{
        "foo" => array![JsonValue::String("sna".to_string()), 100, true]
    };

    let change_short = object!{
        "foo" => array![JsonValue::Short(unsafe { json::short::Short::from_slice("sna") }), 100, true]
    };

    assert_eq!(left, left_copy);
    assert_eq!(left, left_string);
    assert_eq!(left, left_short);
    assert_ne!(left, change_bool);
    assert_ne!(left, change_string);
    assert_ne!(left, change_short);
}
