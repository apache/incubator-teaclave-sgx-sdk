/////////////////////////////////////////////////////////////////////
// Suppose these are the real traits from Serde.

trait Querializer {}

trait Generic {
    // Not object safe because of this generic method.
    fn generic_fn<Q: Querializer>(&self, querializer: Q);
}

impl<'a, T: ?Sized> Querializer for &'a T where T: Querializer {}

impl<'a, T: ?Sized> Generic for Box<T> where T: Generic {
    fn generic_fn<Q: Querializer>(&self, querializer: Q) {
        (**self).generic_fn(querializer)
    }
}

/////////////////////////////////////////////////////////////////////
// This is an object-safe equivalent that interoperates seamlessly.

trait ErasedGeneric {
    fn erased_fn(&self, querializer: &Querializer);
}

impl Generic for ErasedGeneric {
    // Depending on the trait method signatures and the upstream
    // impls, could also implement for:
    //
    //   - &'a ErasedGeneric
    //   - &'a (ErasedGeneric + Send)
    //   - &'a (ErasedGeneric + Sync)
    //   - &'a (ErasedGeneric + Send + Sync)
    //   - Box<ErasedGeneric>
    //   - Box<ErasedGeneric + Send>
    //   - Box<ErasedGeneric + Sync>
    //   - Box<ErasedGeneric + Send + Sync>
    fn generic_fn<Q: Querializer>(&self, querializer: Q) {
        self.erased_fn(&querializer)
    }
}

impl<T> ErasedGeneric for T where T: Generic {
    fn erased_fn(&self, querializer: &Querializer) {
        self.generic_fn(querializer)
    }
}

fn main() {
    struct T;
    impl Querializer for T {}

    struct S;
    impl Generic for S {
        fn generic_fn<Q: Querializer>(&self, _querializer: Q) {
            println!("querying the real S");
        }
    }

    // Construct a trait object.
    let trait_object: Box<ErasedGeneric> = Box::new(S);

    // Seamlessly invoke the generic method on the trait object.
    //
    // THIS LINE LOOKS LIKE MAGIC. We have a value of type trait
    // object and we are invoking a generic method on it.
    trait_object.generic_fn(T);
}
