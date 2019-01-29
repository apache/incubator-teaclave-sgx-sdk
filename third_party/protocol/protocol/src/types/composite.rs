/// Implements `Parcel` for some struct.
///
/// This is used to enable serialization of fields for arbitrary structs.
///
/// ```none
/// pub struct Foo { a: u8, b: u16 };
///
/// implement_composite_type!(Foo { a, b });
/// ```
#[macro_export]
macro_rules! implement_composite_type {
    ($ty:ident { $( $field_name:ident ),+ }) => {
        impl $crate::Parcel for $ty
        {
            const TYPE_NAME: &'static str = stringify!($ty);

            fn read(read: &mut ::std::io::Read) -> Result<Self, $crate::Error> {
                Ok($ty {
                    $( $field_name: $crate::Parcel::read(read)? ),+
                })
            }

            fn write(&self, write: &mut ::std::io::Write) -> Result<(), $crate::Error> {
                $( self.$field_name.write(write)?; )+

                Ok(())
            }
        }
    }
}


/// Defines a type built out of other `Parcel` types.
///
/// ```none
/// define_composite_type!(Foo {
///     a: u8,
///     b: u16
/// });
/// ```
#[macro_export]
macro_rules! define_composite_type {
    ($ty:ident { $( $field_name:ident : $field_ty:ty ),+ }) => {
        #[derive(Clone, Debug, PartialEq)]
        pub struct $ty
        {
            $( pub $field_name : $field_ty ),+
        }

        implement_composite_type!($ty {
            $( $field_name ),+
        });
    }
}

#[cfg(test)]
#[allow(unused_variables)]
mod test
{
    pub use Parcel;
    pub use std::io::Cursor;

    #[derive(Clone, Debug, PartialEq)]
    pub struct Foo
    {
        baz: String,
        bing: i64,
    }

    implement_composite_type!(Foo { baz, bing });

    define_composite_type!(Bar {
        baz: String,
        bing: i64
    });

    define_composite_type!(Bing {
        a: u8,
        b: u8,
        c: u8
    });

    #[test]
    fn is_consistent_when_using_the_different_macros() {
        let foo = Foo { baz: "baz".to_string(), bing: 32 };
        let bar = Bar { baz: "baz".to_string(), bing: 32 };
        assert_eq!(foo.raw_bytes().unwrap(), bar.raw_bytes().unwrap());
    }

    #[test]
    fn writing_matches_expected_output() {
        let bing = Bing { a: 3, b: 2, c: 1 };
        assert_eq!(&bing.raw_bytes().unwrap(), &[bing.a, bing.b, bing.c]);
    }

    #[test]
    fn reading_reads_expected_value() {
        let bing = Bing { a: 3, b: 2, c: 1 };
        let mut buffer = Cursor::new([bing.a, bing.b, bing.c]);
        let read = Bing::read(&mut buffer).unwrap();

        assert_eq!(read, bing);
    }
}

