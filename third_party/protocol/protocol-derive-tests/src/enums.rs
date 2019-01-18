#![cfg(test)]

use protocol::Parcel;

#[derive(Protocol, Clone, Debug, PartialEq)]
pub enum WithGenerics<A, B> {
    Foo(A, B),
    Bar,
}

mod string_discriminants {
    #[allow(unused_imports)]
    use protocol::Parcel;
    use verify_read_back;

    #[derive(Protocol, Clone, Debug, PartialEq)]
    #[protocol]
    pub enum PlayerState {
      Stationary,
      Flying { velocity: (f32,f32,f32) },
      Jumping { height: f32 },
    }

    #[derive(Protocol, Debug, PartialEq)]
    #[protocol(discriminant = "string")]
    pub enum Axis { X, Y, Z, Other(String), Bimp { val: u64 } }

    #[derive(Protocol, Debug, PartialEq)]
    #[protocol(discriminant = "string")]
    pub enum RenamedVariant {
        Hello,
        #[protocol(name = "Universe")]
        World,
    }

    #[test]
    fn variant_names_are_discriminators() {
        assert_eq!(vec![0, 0, 0, 1, 'X' as _], Axis::X.raw_bytes().unwrap());
        assert_eq!(vec![0, 0, 0, 5, 'O' as _, 't' as _, 'h' as _, 'e' as _, 'r' as _,
                        0, 0, 0, 4, 'r' as _, 'o' as _, 'l' as _, 'l' as _],
                   Axis::Other("roll".to_owned()).raw_bytes().unwrap());
    }

    #[test]
    fn can_write_and_read_back() {
        verify_read_back(Axis::Other("boop".to_owned()));
        verify_read_back(Axis::X);
        verify_read_back(Axis::Y);
        verify_read_back(Axis::Bimp { val: 77 });
    }

    #[test]
    fn renamed_variants_are_transmitted() {
        assert_eq!(vec![0, 0, 0, 5, 'H' as _, 'e' as _, 'l' as _, 'l' as _, 'o' as _], RenamedVariant::Hello.raw_bytes().unwrap());
        assert_eq!(vec![0, 0, 0, 8, 'U' as _, 'n' as _, 'i' as _, 'v' as _, 'e' as _, 'r' as _, 's' as _, 'e' as _], RenamedVariant::World.raw_bytes().unwrap());
    }

    #[test]
    fn renamed_variants_can_be_written_and_read_back() {
        verify_read_back(RenamedVariant::World);
    }
}

mod generics {
    use verify_read_back;
    use std::fmt;

    #[derive(Protocol, Debug, PartialEq)]
    pub enum EnumWithEmptyGenerics<> { First { a: u32, b: String, c: u64 } }
    #[derive(Protocol, Debug, PartialEq)]
    pub enum EnumWithUnconstrainedType<T> { Variant1 { a: T, b: T }, Variant2 { c: T } }
    #[derive(Protocol, Debug, PartialEq)]
    pub enum EnumWithUnconstrainedTypes<A,B,C,D> { Value { a: A, b: B, c: C, d: D }, Variant2 { a: A } }
    #[derive(Protocol, Debug, PartialEq)]
    pub enum EnumWithConstrainedType<T: Clone + PartialEq + fmt::Debug + fmt::Display> { Variant1 { inner: T }, Variant2 { c: T } }
    #[derive(Protocol, Debug, PartialEq)]
    pub enum EnumWithConstrainedTypes<T: Clone, A: fmt::Debug + fmt::Display, B: Copy> { Variant1 { t: T, a: A, b: B }, Variant2 { c: T } }
    #[derive(Protocol, Debug, PartialEq)]
    pub enum EnumWithWhereClause<T> where T: fmt::Debug + fmt::Display { Variant1 { t: T }, Variant2 { t: T } }
    #[derive(Protocol, Debug, PartialEq)]
    pub enum EnumWithWhereClauses<A,B,C> where A: Copy, B: fmt::Debug + fmt::Display, C: Clone + Copy { Variant1 { a: A, b: B, c: C }, Variant2 { a: A } }

    #[test]
    fn can_read_back_empty_generics() {
        verify_read_back(EnumWithEmptyGenerics::First { a: 22, b: "boop".to_owned(), c: !0 });
    }

    #[test]
    fn can_read_back_single_unconstrained_type() {
        let v: EnumWithUnconstrainedType<String> = EnumWithUnconstrainedType::Variant2 { c: "hello".to_owned() };
        verify_read_back(v);
    }

    #[test]
    fn can_read_back_multiple_unconstrained_types() {
        let v = EnumWithUnconstrainedTypes::Value {
            a: "hello".to_string(), b: 55u8, c: 128u64, d: 99i64,
        };
        verify_read_back(v);
    }

    #[test]
    fn can_read_back_single_constrained_type() {
        let v: EnumWithConstrainedType<String> = EnumWithConstrainedType::Variant1 { inner: "hello".to_owned() };
        verify_read_back(v);
    }

    #[test]
    fn can_read_back_multiple_constrained_types() {
        let v = EnumWithConstrainedTypes::Variant1 { t: "hello".to_owned(), a: 250u8, b: 155i16 };
        verify_read_back(v);
    }

    #[test]
    fn can_read_back_where_clause() {
        let v = EnumWithWhereClause::Variant1 { t: "hello".to_owned() };
        verify_read_back(v);
    }

    #[test]
    fn can_read_back_where_clauses() {
        let v = EnumWithWhereClauses::Variant1 { a: 7u16, b: "hello".to_owned(), c: 99u8 };
        verify_read_back(v);
    }
}

mod integer_discriminants {
    #[allow(unused_imports)]
    use protocol::Parcel;

    #[derive(Protocol, Debug, PartialEq, Eq)]
    #[protocol(discriminant = "integer")]
    pub enum BoatKind {
        Speedboat { warp_speed_enabled: bool },
        Dingy(u8, u8),
        Fart,
    }

    #[test]
    fn named_fields_are_correctly_written() {
        assert_eq!(vec![0, 0, 0, 1, 1], BoatKind::Speedboat {
            warp_speed_enabled: true,
        }.raw_bytes().unwrap());
    }

    #[test]
    fn unnamed_fields_are_correctly_written() {
        assert_eq!(vec![0, 0, 0, 2, // discriminator
                        0xf1, 0xed], BoatKind::Dingy(0xf1, 0xed).raw_bytes().unwrap());
    }

    #[test]
    fn unit_variants_are_correctly_written() {
        assert_eq!(vec![0, 0, 0, 3], // discriminator
                   BoatKind::Fart.raw_bytes().unwrap());
    }

    #[test]
    fn named_fields_are_correctly_read() {
        assert_eq!(BoatKind::Speedboat {
            warp_speed_enabled: true,
        }, BoatKind::from_raw_bytes(&[0, 0, 0, 1, 1]).unwrap());
    }

    #[test]
    fn unnamed_fields_are_correctly_read() {
        assert_eq!(BoatKind::Dingy(99, 78),
                   BoatKind::from_raw_bytes(&[0, 0, 0, 2, 99, 78]).unwrap());
    }

    #[test]
    fn unit_variants_are_correctly_read() {
        assert_eq!(BoatKind::Fart,
                   BoatKind::from_raw_bytes(&[0, 0, 0, 3]).unwrap());
    }
}

#[derive(Protocol)]
enum OneVariant { A }

#[derive(Protocol)]
enum BuzzyBee { B(u32, u32) }

#[test]
fn type_name_is_correct() {
    assert_eq!("OneVariant", OneVariant::A.type_name());
    assert_eq!("BuzzyBee", BuzzyBee::B(2,1).type_name());
}
