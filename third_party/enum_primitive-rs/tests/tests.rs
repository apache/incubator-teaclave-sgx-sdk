#[macro_use] extern crate enum_primitive as ep;

enum_from_primitive! {
enum Unused {
    A = 17,
    B = 42
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
enum Empty {
}
}

#[test]
fn empty() {
    use ep::FromPrimitive;
    assert_eq!(Empty::from_i32(91), None);
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
enum One {
    A = 17
}
}

#[test]
fn one() {
    use ep::FromPrimitive;
    assert_eq!(One::from_isize(17), Some(One::A));
    assert_eq!(One::from_isize(91), None);
    assert_eq!(One::from_i8(17), Some(One::A));
    assert_eq!(One::from_i8(91), None);
    assert_eq!(One::from_i16(17), Some(One::A));
    assert_eq!(One::from_i16(91), None);
    assert_eq!(One::from_i32(17), Some(One::A));
    assert_eq!(One::from_i32(91), None);
    assert_eq!(One::from_i64(17), Some(One::A));
    assert_eq!(One::from_i64(91), None);
    assert_eq!(One::from_usize(17), Some(One::A));
    assert_eq!(One::from_usize(91), None);
    assert_eq!(One::from_u8(17), Some(One::A));
    assert_eq!(One::from_u8(91), None);
    assert_eq!(One::from_u16(17), Some(One::A));
    assert_eq!(One::from_u16(91), None);
    assert_eq!(One::from_u32(17), Some(One::A));
    assert_eq!(One::from_u32(91), None);
    assert_eq!(One::from_u64(17), Some(One::A));
    assert_eq!(One::from_u64(91), None);
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
enum OneComma {
    A = 17,
}
}

#[test]
fn one_comma() {
    use ep::FromPrimitive;
    assert_eq!(OneComma::from_i32(17), Some(OneComma::A));
    assert_eq!(OneComma::from_i32(91), None);
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
enum Two {
    A = 17,
    B = 42
}
}

#[test]
fn two() {
    use ep::FromPrimitive;
    assert_eq!(PubTwo::from_i32(17), Some(PubTwo::A));
    assert_eq!(PubTwo::from_i32(42), Some(PubTwo::B));
    assert_eq!(PubTwo::from_i32(91), None);
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
enum TwoComma {
    A = 17,
    B = 42,
}
}

#[test]
fn two_comma() {
    use ep::FromPrimitive;
    assert_eq!(TwoComma::from_i32(17), Some(TwoComma::A));
    assert_eq!(TwoComma::from_i32(42), Some(TwoComma::B));
    assert_eq!(TwoComma::from_i32(91), None);
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum PubEmpty {
}
}

#[test]
fn pub_empty() {
    use ep::FromPrimitive;
    assert_eq!(PubEmpty::from_i32(91), None);
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum PubOne {
    A = 17
}
}

#[test]
fn pub_one() {
    use ep::FromPrimitive;
    assert_eq!(PubOne::from_i32(17), Some(PubOne::A));
    assert_eq!(PubOne::from_i32(91), None);
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum PubOneComma {
    A = 17,
}
}

#[test]
fn pub_one_comma() {
    use ep::FromPrimitive;
    assert_eq!(PubOneComma::from_i32(17), Some(PubOneComma::A));
    assert_eq!(PubOneComma::from_i32(91), None);
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum PubTwo {
    A = 17,
    B = 42
}
}

#[test]
fn pub_two() {
    use ep::FromPrimitive;
    assert_eq!(PubTwo::from_i32(17), Some(PubTwo::A));
    assert_eq!(PubTwo::from_i32(42), Some(PubTwo::B));
    assert_eq!(PubTwo::from_i32(91), None);
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum PubTwoComma {
    A = 17,
    B = 42,
}
}

#[test]
fn pub_two_comma() {
    use ep::FromPrimitive;
    assert_eq!(PubTwoComma::from_i32(17), Some(PubTwoComma::A));
    assert_eq!(PubTwoComma::from_i32(42), Some(PubTwoComma::B));
    assert_eq!(PubTwoComma::from_i32(91), None);
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
enum Negative {
    A = -17
}
}

#[test]
fn negative() {
    use ep::FromPrimitive;
    assert_eq!(Negative::from_isize(-17), Some(Negative::A));
    assert_eq!(Negative::from_isize(-91), None);
    assert_eq!(Negative::from_i8(-17), Some(Negative::A));
    assert_eq!(Negative::from_i8(-91), None);
    assert_eq!(Negative::from_i16(-17), Some(Negative::A));
    assert_eq!(Negative::from_i16(-91), None);
    assert_eq!(Negative::from_i32(-17), Some(Negative::A));
    assert_eq!(Negative::from_i32(-91), None);
    assert_eq!(Negative::from_i64(-17), Some(Negative::A));
    assert_eq!(Negative::from_i64(-91), None);
    assert_eq!(Negative::from_usize(!16), Some(Negative::A));
    assert_eq!(Negative::from_usize(!90), None);
    assert_eq!(Negative::from_u8(!16), None);
    assert_eq!(Negative::from_u8(!90), None);
    assert_eq!(Negative::from_u16(!16), None);
    assert_eq!(Negative::from_u16(!90), None);
    assert_eq!(Negative::from_u32(!16), None);
    assert_eq!(Negative::from_u32(!90), None);
    assert_eq!(Negative::from_u64(!16), Some(Negative::A));
    assert_eq!(Negative::from_u64(!90), None);
}

#[test]
fn in_local_mod() {
    mod local_mod {
        enum_from_primitive! {
        #[derive(Debug, PartialEq)]
        pub enum InLocalMod {
            A = 17,
            B = 42,
        }
        }
    }

    use ep::FromPrimitive;
    assert_eq!(local_mod::InLocalMod::from_i32(17), Some(local_mod::InLocalMod::A));
    assert_eq!(local_mod::InLocalMod::from_i32(42), Some(local_mod::InLocalMod::B));
    assert_eq!(local_mod::InLocalMod::from_i32(91), None);
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
#[doc = "Documented"]
pub enum Documented {
    A = 17
}
}

#[test]
fn documented() {
    use ep::FromPrimitive;
    assert_eq!(Documented::from_i32(17), Some(Documented::A));
    assert_eq!(Documented::from_i32(91), None);
}
