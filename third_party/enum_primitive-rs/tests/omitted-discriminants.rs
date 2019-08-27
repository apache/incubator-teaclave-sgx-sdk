#[macro_use] extern crate enum_primitive;

enum_from_primitive! { enum E { } }
enum_from_primitive! { enum E0 { V0 } }
enum_from_primitive! { enum E0C { V0, } }
enum_from_primitive! { enum E1 { V0 = 0 } }
enum_from_primitive! { enum E1C { V0 = 0, } }
enum_from_primitive! { enum E00 { V0, V1 } }
enum_from_primitive! { enum E00C { V0, V1, } }
enum_from_primitive! { enum E01 { V0, V1 = 1 } }
enum_from_primitive! { enum E01C { V0, V1 = 1, } }
enum_from_primitive! { enum E10 { V0 = 0, V1 } }
enum_from_primitive! { enum E10C { V0 = 0, V1, } }
enum_from_primitive! { enum E11 { V0 = 0, V1 = 1 } }
enum_from_primitive! { enum E11C { V0 = 0, V1 = 1, } }
enum_from_primitive! { enum E000 { V0, V1, V2 } }
enum_from_primitive! { enum E000C { V0, V1, V2, } }
enum_from_primitive! { enum E001 { V0, V1, V2 = 2 } }
enum_from_primitive! { enum E001C { V0, V1, V2 = 2, } }
enum_from_primitive! { enum E010 { V0, V1 = 1, V2 } }
enum_from_primitive! { enum E010C { V0, V1 = 1, V2, } }
enum_from_primitive! { enum E011 { V0, V1 = 1, V2 = 2 } }
enum_from_primitive! { enum E011C { V0, V1 = 1, V2 = 2, } }
enum_from_primitive! { enum E100 { V0 = 0, V1, V2 } }
enum_from_primitive! { enum E100C { V0 = 0, V1, V2, } }
enum_from_primitive! { enum E101 { V0 = 0, V1, V2 = 2 } }
enum_from_primitive! { enum E101C { V0 = 0, V1, V2 = 2, } }
enum_from_primitive! { enum E110 { V0 = 0, V1 = 1, V2 } }
enum_from_primitive! { enum E110C { V0 = 0, V1 = 1, V2, } }
enum_from_primitive! { enum E111 { V0 = 0, V1 = 1, V2 = 2 } }
enum_from_primitive! { enum E111C { V0 = 0, V1 = 1, V2 = 2, } }
enum_from_primitive! { enum E0000 { V0, V1, V2, V3 } }
enum_from_primitive! { enum E0000C { V0, V1, V2, V3, } }
enum_from_primitive! { enum E0001 { V0, V1, V2, V3 = 3 } }
enum_from_primitive! { enum E0001C { V0, V1, V2, V3 = 3, } }
enum_from_primitive! { enum E0010 { V0, V1, V2 = 2, V3 } }
enum_from_primitive! { enum E0010C { V0, V1, V2 = 2, V3, } }
enum_from_primitive! { enum E0011 { V0, V1, V2 = 2, V3 = 3 } }
enum_from_primitive! { enum E0011C { V0, V1, V2 = 2, V3 = 3, } }
enum_from_primitive! { enum E0100 { V0, V1 = 1, V2, V3 } }
enum_from_primitive! { enum E0100C { V0, V1 = 1, V2, V3, } }
enum_from_primitive! { enum E0101 { V0, V1 = 1, V2, V3 = 3 } }
enum_from_primitive! { enum E0101C { V0, V1 = 1, V2, V3 = 3, } }
enum_from_primitive! { enum E0110 { V0, V1 = 1, V2 = 2, V3 } }
enum_from_primitive! { enum E0110C { V0, V1 = 1, V2 = 2, V3, } }
enum_from_primitive! { enum E0111 { V0, V1 = 1, V2 = 2, V3 = 3 } }
enum_from_primitive! { enum E0111C { V0, V1 = 1, V2 = 2, V3 = 3, } }
enum_from_primitive! { enum E1000 { V0 = 0, V1, V2, V3 } }
enum_from_primitive! { enum E1000C { V0 = 0, V1, V2, V3, } }
enum_from_primitive! { enum E1001 { V0 = 0, V1, V2, V3 = 3 } }
enum_from_primitive! { enum E1001C { V0 = 0, V1, V2, V3 = 3, } }
enum_from_primitive! { enum E1010 { V0 = 0, V1, V2 = 2, V3 } }
enum_from_primitive! { enum E1010C { V0 = 0, V1, V2 = 2, V3, } }
enum_from_primitive! { enum E1011 { V0 = 0, V1, V2 = 2, V3 = 3 } }
enum_from_primitive! { enum E1011C { V0 = 0, V1, V2 = 2, V3 = 3, } }
enum_from_primitive! { enum E1100 { V0 = 0, V1 = 1, V2, V3 } }
enum_from_primitive! { enum E1100C { V0 = 0, V1 = 1, V2, V3, } }
enum_from_primitive! { enum E1101 { V0 = 0, V1 = 1, V2, V3 = 3 } }
enum_from_primitive! { enum E1101C { V0 = 0, V1 = 1, V2, V3 = 3, } }
enum_from_primitive! { enum E1110 { V0 = 0, V1 = 1, V2 = 2, V3 } }
enum_from_primitive! { enum E1110C { V0 = 0, V1 = 1, V2 = 2, V3, } }
enum_from_primitive! { enum E1111 { V0 = 0, V1 = 1, V2 = 2, V3 = 3 } }
enum_from_primitive! { enum E1111C { V0 = 0, V1 = 1, V2 = 2, V3 = 3, } }
