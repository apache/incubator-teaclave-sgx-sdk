macro_rules! impl_list_type {
    ( $ty:ident => T: $( $ty_pred:ident ),* ) => {
        impl<T> $crate::Parcel for ::std::collections::$ty<T>
            where T: $crate::Parcel $( + $ty_pred )*
        {
            const TYPE_NAME: &'static str = stringify!($ty<T>);

            fn read(read: &mut dyn (::std::io::Read)) -> Result<Self, $crate::Error> {
                let elements = ::types::util::read_list(read)?;
                Ok(elements.into_iter().collect())
            }

            fn write(&self, write: &mut dyn (::std::io::Write))
                -> Result<(), $crate::Error> {
                ::types::util::write_list(write, self.iter())
            }
        }

        #[cfg(test)]
        mod test
        {
            pub use Parcel;
            pub use std::collections::$ty;

            #[test]
            fn can_be_written_and_read_back_correctly() {
                let original: $ty<u32> = [1, 2, 3, 4, 5].iter().cloned().collect();

                let raw_bytes = original.raw_bytes().unwrap();
                let read_deque = $ty::<u32>::from_raw_bytes(&raw_bytes).unwrap();

                assert_eq!(original, read_deque);
            }
        }
    }
}

pub mod linked_list { impl_list_type!(LinkedList => T: ); }
pub mod vec_deque   { impl_list_type!(VecDeque   => T: ); }

pub mod btree_set   { impl_list_type!(BTreeSet   => T: Ord); }

pub mod hash_set {
    use std::hash::Hash;
    impl_list_type!(HashSet => T: Hash, Eq);
}

