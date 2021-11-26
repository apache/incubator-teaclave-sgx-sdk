use sgx_serialize::{DeSerializable, DeSerializeHelper, Serializable, SerializeHelper};
use std::fmt::Debug;
use std::string::{String, ToString};
use std::vec::Vec;

fn test_serialize_internal<T: Serializable + DeSerializable>(target: &T) -> Option<T> {
    let helper = SerializeHelper::new();
    let data = helper.encode(target).unwrap();
    let helper = DeSerializeHelper::<T>::new(data);
    helper.decode()
}

pub fn test_serialize_struct() {
    #[derive(Serializable, DeSerializable, PartialEq, Debug)]
    struct TestSturct {
        a1: u32,
        a2: u32,
    }
    let a = TestSturct {
        a1: 2017u32,
        a2: 829u32,
    };
    let c = test_serialize_internal::<TestSturct>(&a).unwrap();
    assert_eq!(a, c);

    #[derive(Serializable, DeSerializable, PartialEq, Debug)]
    struct TestStructUnit;
    let a = TestStructUnit;
    let c = test_serialize_internal::<TestStructUnit>(&a).unwrap();
    assert_eq!(a, c);

    #[derive(Serializable, DeSerializable, PartialEq, Debug)]
    struct TestStructNewType(i32);
    let a = TestStructNewType(2017i32);
    let c = test_serialize_internal::<TestStructNewType>(&a).unwrap();
    assert_eq!(a, c);

    #[derive(Serializable, DeSerializable, PartialEq, Debug)]
    struct TestStructTuple(i32, i32);
    let a = TestStructTuple(2017i32, 928i32);
    let c = test_serialize_internal::<TestStructTuple>(&a).unwrap();
    assert_eq!(a, c);
}

pub fn test_serialize_enum() {
    #[derive(Serializable, DeSerializable, PartialEq, Debug)]
    struct TestSturct {
        a1: u32,
        a2: u32,
    }
    #[derive(Serializable, DeSerializable, PartialEq, Debug)]
    enum TestEnum {
        EnumUnit,
        EnumNewType(u32),
        EnumTuple(u32, u32),
        EnumStruct { a1: i32, a2: i32 },
        EnumSubStruct(TestSturct),
    }

    let a = TestSturct {
        a1: 2017u32,
        a2: 928u32,
    };
    let a = TestEnum::EnumSubStruct(a);
    let c = test_serialize_internal::<TestEnum>(&a).unwrap();
    assert_eq!(a, c);

    let a = TestEnum::EnumTuple(2017, 829);
    let c = test_serialize_internal::<TestEnum>(&a).unwrap();
    assert_eq!(a, c);

    let a = TestEnum::EnumNewType(2017);
    let c = test_serialize_internal::<TestEnum>(&a).unwrap();
    assert_eq!(a, c);

    let a = TestEnum::EnumStruct { a1: 2017, a2: 829 };
    let c = test_serialize_internal::<TestEnum>(&a).unwrap();
    assert_eq!(a, c);
}

pub fn test_serialize_base() {
    #[derive(Serializable, DeSerializable, PartialEq, Clone, Debug)]
    struct Struct {
        a: (),
        b: u8,
        c: u16,
        d: u32,
        e: u64,
        f: usize,

        g: i8,
        h: i16,
        i: i32,
        j: i64,
        k: isize,

        l: char,
        m: String,
        n: f32,
        o: f64,
        p: bool,
        q: Option<u32>,
    }

    fn check_round_trip<T: Serializable + DeSerializable + PartialEq + Debug>(values: Vec<T>) {
        for value in values {
            let decoded = test_serialize_internal::<T>(&value).unwrap();
            assert_eq!(value, decoded);
        }
    }

    fn test_unit() {
        check_round_trip(vec![(), (), (), ()]);
    }

    fn test_u8() {
        let mut vec = vec![];
        for i in ::std::u8::MIN..::std::u8::MAX {
            vec.push(i);
        }
        check_round_trip(vec);
    }

    fn test_u16() {
        for i in ::std::u16::MIN..::std::u16::MAX {
            check_round_trip(vec![1, 2, 3, i, i, i]);
        }
    }

    fn test_u32() {
        check_round_trip(vec![1, 2, 3, ::std::u32::MIN, 0, 1, ::std::u32::MAX, 2, 1]);
    }

    fn test_u64() {
        check_round_trip(vec![1, 2, 3, ::std::u64::MIN, 0, 1, ::std::u64::MAX, 2, 1]);
    }

    fn test_usize() {
        check_round_trip(vec![
            1,
            2,
            3,
            ::std::usize::MIN,
            0,
            1,
            ::std::usize::MAX,
            2,
            1,
        ]);
    }

    fn test_i8() {
        let mut vec = vec![];
        for i in ::std::i8::MIN..::std::i8::MAX {
            vec.push(i);
        }
        check_round_trip(vec);
    }

    fn test_i16() {
        for i in ::std::i16::MIN..::std::i16::MAX {
            check_round_trip(vec![-1, 2, -3, i, i, i, 2]);
        }
    }

    fn test_i32() {
        check_round_trip(vec![
            -1,
            2,
            -3,
            ::std::i32::MIN,
            0,
            1,
            ::std::i32::MAX,
            2,
            1,
        ]);
    }

    fn test_i64() {
        check_round_trip(vec![
            -1,
            2,
            -3,
            ::std::i64::MIN,
            0,
            1,
            ::std::i64::MAX,
            2,
            1,
        ]);
    }

    fn test_isize() {
        check_round_trip(vec![
            -1,
            2,
            -3,
            ::std::isize::MIN,
            0,
            1,
            ::std::isize::MAX,
            2,
            1,
        ]);
    }

    fn test_bool() {
        check_round_trip(vec![false, true, true, false, false]);
    }

    fn test_f32() {
        let mut vec = vec![];
        for i in -100..100 {
            vec.push((i as f32) / 3.0);
        }
        check_round_trip(vec);
    }

    fn test_f64() {
        let mut vec = vec![];
        for i in -100..100 {
            vec.push((i as f64) / 3.0);
        }
        check_round_trip(vec);
    }

    fn test_char() {
        let vec = vec!['a', 'b', 'c', 'd', 'A', 'X', ' ', '#', 'Ö', 'Ä', 'µ', '€'];
        check_round_trip(vec);
    }

    fn test_string() {
        let vec = vec![
            "abcbuÖeiovÄnameÜavmpßvmea€µsbpnvapeapmaebn".to_string(),
            "abcbuÖganeiovÄnameÜavmpßvmea€µsbpnvapeapmaebn".to_string(),
            "abcbuÖganeiovÄnameÜavmpßvmea€µsbpapmaebn".to_string(),
            "abcbuÖganeiovÄnameÜavmpßvmeabpnvapeapmaebn".to_string(),
            "abcbuÖganeiÄnameÜavmpßvmea€µsbpnvapeapmaebn".to_string(),
            "abcbuÖganeiovÄnameÜavmpßvmea€µsbpmaebn".to_string(),
            "abcbuÖganeiovÄnameÜavmpßvmea€µnvapeapmaebn".to_string(),
        ];

        check_round_trip(vec);
    }

    fn test_option() {
        check_round_trip(vec![Some(-1i8)]);
        check_round_trip(vec![Some(-2i16)]);
        check_round_trip(vec![Some(-3i32)]);
        check_round_trip(vec![Some(-4i64)]);
        check_round_trip(vec![Some(-5isize)]);

        let none_i8: Option<i8> = None;
        check_round_trip(vec![none_i8]);

        let none_i16: Option<i16> = None;
        check_round_trip(vec![none_i16]);

        let none_i32: Option<i32> = None;
        check_round_trip(vec![none_i32]);

        let none_i64: Option<i64> = None;
        check_round_trip(vec![none_i64]);

        let none_isize: Option<isize> = None;
        check_round_trip(vec![none_isize]);
    }

    fn test_struct() {
        check_round_trip(vec![Struct {
            a: (),
            b: 10,
            c: 11,
            d: 12,
            e: 13,
            f: 14,

            g: 15,
            h: 16,
            i: 17,
            j: 18,
            k: 19,

            l: 'x',
            m: "abc".to_string(),
            n: 20.5,
            o: 21.5,
            p: false,
            q: None,
        }]);

        check_round_trip(vec![Struct {
            a: (),
            b: 101,
            c: 111,
            d: 121,
            e: 131,
            f: 141,

            g: -15,
            h: -16,
            i: -17,
            j: -18,
            k: -19,

            l: 'y',
            m: "def".to_string(),
            n: -20.5,
            o: -21.5,
            p: true,
            q: Some(1234567),
        }]);
    }

    #[derive(PartialEq, Clone, Debug, Serializable, DeSerializable)]
    enum Enum {
        Variant1,
        Variant2(usize, f32),
        Variant3 { a: i32, b: char, c: bool },
    }

    fn test_enum() {
        check_round_trip(vec![
            Enum::Variant1,
            Enum::Variant2(1, 2.5),
            Enum::Variant3 {
                a: 3,
                b: 'b',
                c: false,
            },
            Enum::Variant3 {
                a: -4,
                b: 'f',
                c: true,
            },
        ]);
    }

    fn test_sequence() {
        let mut vec = vec![];
        for i in -100i64..100i64 {
            vec.push(i * 100000);
        }

        check_round_trip(vec![vec]);
    }

    fn test_hash_map() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        for i in -100i64..100i64 {
            map.insert(i * 100000, i * 10000);
        }

        check_round_trip(vec![map]);
    }

    fn test_tuples() {
        check_round_trip(vec![('x', (), false, 0.5f32)]);
        check_round_trip(vec![(9i8, 10u16, 1.5f64)]);
        check_round_trip(vec![(-12i16, 11u8, 12usize)]);
        check_round_trip(vec![(1234567isize, 100000000000000u64, 99999999999999i64)]);
        check_round_trip(vec![(String::new(), "some string".to_string())]);
    }

    test_unit();
    test_u8();
    test_u16();
    test_u32();
    test_u64();
    test_usize();
    test_i8();
    test_i16();
    test_i32();
    test_i64();
    test_isize();
    test_bool();
    test_f32();
    test_f64();
    test_char();
    test_string();
    test_option();
    test_struct();
    test_enum();
    test_sequence();
    test_hash_map();
    test_tuples();
}
