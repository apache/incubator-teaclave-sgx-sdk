/// Implement `serde::Serialize` for a trait object that has
/// `erased_serde::Serialize` as a supertrait.
///
/// ```
/// #[macro_use]
/// extern crate erased_serde;
///
/// trait Event: erased_serde::Serialize {
///     /* ... */
/// }
///
/// serialize_trait_object!(Event);
/// #
/// # fn main() {}
/// ```
///
/// The macro supports traits that have type parameters and/or `where` clauses.
///
/// ```
/// # #[macro_use]
/// # extern crate erased_serde;
/// #
/// trait Difficult<T>: erased_serde::Serialize where T: Copy {
///     /* ... */
/// }
///
/// serialize_trait_object!(<T> Difficult<T> where T: Copy);
/// #
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! serialize_trait_object {
    ($($path:tt)+) => {
        __internal_serialize_trait_object!(begin $($path)+);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_serialize_trait_object {
    // Invocation started with `<`, parse generics.
    (begin < $($rest:tt)*) => {
        __internal_serialize_trait_object!(generics () () $($rest)*);
    };

    // Invocation did not start with `<`.
    (begin $first:tt $($rest:tt)*) => {
        __internal_serialize_trait_object!(path () ($first) $($rest)*);
    };

    // End of generics.
    (generics ($($generics:tt)*) () > $($rest:tt)*) => {
        __internal_serialize_trait_object!(path ($($generics)*) () $($rest)*);
    };

    // Generics open bracket.
    (generics ($($generics:tt)*) ($($brackets:tt)*) < $($rest:tt)*) => {
        __internal_serialize_trait_object!(generics ($($generics)* <) ($($brackets)* <) $($rest)*);
    };

    // Generics close bracket.
    (generics ($($generics:tt)*) (< $($brackets:tt)*) > $($rest:tt)*) => {
        __internal_serialize_trait_object!(generics ($($generics)* >) ($($brackets)*) $($rest)*);
    };

    // Token inside of generics.
    (generics ($($generics:tt)*) ($($brackets:tt)*) $first:tt $($rest:tt)*) => {
        __internal_serialize_trait_object!(generics ($($generics)* $first) ($($brackets)*) $($rest)*);
    };

    // End with `where` clause.
    (path ($($generics:tt)*) ($($path:tt)*) where $($rest:tt)*) => {
        __internal_serialize_trait_object!(sendsync ($($generics)*) ($($path)*) ($($rest)*));
    };

    // End without `where` clause.
    (path ($($generics:tt)*) ($($path:tt)*)) => {
        __internal_serialize_trait_object!(sendsync ($($generics)*) ($($path)*) ());
    };

    // Token inside of path.
    (path ($($generics:tt)*) ($($path:tt)*) $first:tt $($rest:tt)*) => {
        __internal_serialize_trait_object!(path ($($generics)*) ($($path)* $first) $($rest)*);
    };

    // Expand into four impls.
    (sendsync ($($generics:tt)*) ($($path:tt)*) ($($bound:tt)*)) => {
        __internal_serialize_trait_object!(impl ($($generics)*) ($($path)*) ($($bound)*));
        __internal_serialize_trait_object!(impl ($($generics)*) ($($path)* + $crate::private::Send) ($($bound)*));
        __internal_serialize_trait_object!(impl ($($generics)*) ($($path)* + $crate::private::Sync) ($($bound)*));
        __internal_serialize_trait_object!(impl ($($generics)*) ($($path)* + $crate::private::Send + $crate::private::Sync) ($($bound)*));
    };

    // The impl.
    (impl ($($generics:tt)*) ($($path:tt)*) ($($bound:tt)*)) => {
        impl<'erased, $($generics)*> $crate::private::serde::Serialize for $($path)* + 'erased where $($bound)* {
            fn serialize<S>(&self, serializer: S) -> $crate::private::Result<S::Ok, S::Error> where S: $crate::private::serde::Serializer {
                $crate::serialize(self, serializer)
            }
        }
    };
}

// TEST ////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use serde;
    use Serialize;

    fn assert_serialize<T: ?Sized + serde::Serialize>() {}

    #[test]
    fn test_plain() {
        trait Trait: Serialize {}

        serialize_trait_object!(Trait);
        assert_serialize::<Trait>();
        assert_serialize::<Trait + Send>();
    }

    #[test]
    fn test_type_parameter() {
        trait Trait<T>: Serialize {}

        serialize_trait_object!(<T> Trait<T>);
        assert_serialize::<Trait<u32>>();
        assert_serialize::<Trait<u32> + Send>();
    }

    #[test]
    fn test_generic_bound() {
        trait Trait<T: PartialEq<T>, U>: Serialize {}

        serialize_trait_object!(<T: PartialEq<T>, U> Trait<T, U>);
        assert_serialize::<Trait<u32, ()>>();
        assert_serialize::<Trait<u32, ()> + Send>();
    }

    #[test]
    fn test_where_clause() {
        trait Trait<T>: Serialize where T: Clone {}

        serialize_trait_object!(<T> Trait<T> where T: Clone);
        assert_serialize::<Trait<u32>>();
        assert_serialize::<Trait<u32> + Send>();
    }
}
