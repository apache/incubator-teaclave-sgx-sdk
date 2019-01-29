
/// Defines a new struct-based packet.
#[macro_export]
macro_rules! define_packet
{
    // Define a normal packet.
    ( $ty:ident { $( $field_name:ident : $field_ty:ty),+ }) => {
        #[derive(Clone, Debug, PartialEq)]
        pub struct $ty
        {
            $( pub $field_name : $field_ty ),+
        }

        impl $crate::Parcel for $ty
        {
            const TYPE_NAME: &'static str = stringify!($ty);

            fn read(read: &mut ::std::io::Read) -> Result<Self, $crate::Error> {
                #[allow(unused_imports)]
                use $crate::Parcel;

                Ok($ty {
                    $( $field_name : $crate::Parcel::read(read)?, )+
                })
            }

            fn write(&self, write: &mut ::std::io::Write) -> Result<(), $crate::Error> {
                #[allow(unused_imports)]
                use $crate::Parcel;

                $( self.$field_name.write(write)?; )+

                Ok(())
            }
        }
    };

    // Define an empty packet.
    ( $ty:ident ) => {
        #[derive(Clone, Debug, PartialEq)]
        pub struct $ty;

        impl $crate::Parcel for $ty
        {
            const TYPE_NAME: &'static str = stringify!($ty);

            fn read(_read: &mut ::std::io::Read) -> Result<Self, $crate::Error> {
                Ok($ty)
            }

            fn write(&self, _write: &mut ::std::io::Write) -> Result<(), $crate::Error> {
                Ok(())
            }
        }
    };
}

/// Defines a packet kind enum.
///
/// You can use any type that implements `Parcel` as the packet ID.
#[macro_export]
macro_rules! define_packet_kind
{
    ( $ty:ident : $id_ty:ty { $( $packet_id:expr => $packet_ty:ident ),+ } ) => {
        #[derive(Clone, Debug, PartialEq)]
        pub enum $ty
        {
            $( $packet_ty($packet_ty) ),+
        }

        impl $ty
        {
            /// Gets the ID of the packet.
            pub fn packet_id(&self) -> $id_ty {
                match *self {
                    $( $ty::$packet_ty(..) => $packet_id ),+
                }
            }
        }

        impl $crate::Parcel for $ty
        {
            const TYPE_NAME: &'static str = stringify!($ty);

            fn read(read: &mut ::std::io::Read) -> Result<Self, $crate::Error> {
                let packet_id = <$id_ty as $crate::Parcel>::read(read)?;

                let packet = match packet_id {
                    $( $packet_id => $ty::$packet_ty(<$packet_ty as $crate::Parcel>::read(read)?), )+
                    _ => return Err($crate::ErrorKind::UnknownPacketId.into()),
                };

                Ok(packet)
            }

            fn write(&self, write: &mut ::std::io::Write) -> Result<(), $crate::Error> {
                #[allow(unused_imports)]
                use $crate::Parcel;

                self.packet_id().write(write)?;

                match *self {
                    $( $ty::$packet_ty(ref p) => <$packet_ty as $crate::Parcel>::write(p, write)? ),+
                }

                Ok(())
            }
        }
    }
}

