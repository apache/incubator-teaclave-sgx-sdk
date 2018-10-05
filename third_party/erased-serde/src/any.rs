use std::prelude::v1::*;
use std::mem;

#[cfg(feature = "unstable-debug")]
use std::intrinsics;

pub struct Any {
    ptr: *mut (),
    drop: fn(*mut ()),
    fingerprint: Fingerprint,

    /// For panic messages only. Not used for comparison.
    #[cfg(feature = "unstable-debug")]
    type_name: &'static str,
}

// These functions are all unsafe. They are not exposed to the user. Declaring
// them as `unsafe fn` would not make the rest of erased-serde any safer or more
// readable.
impl Any {
    // This is unsafe -- caller must not hold on to the Any beyond the lifetime
    // of T.
    //
    // Example of bad code:
    //
    //    let s = "bad".to_owned();
    //    let a = Any::new(&s);
    //    drop(s);
    //
    // Now `a.view()` and `a.take()` return references to a dead String.
    pub(crate) fn new<T>(t: T) -> Self {
        let ptr = Box::into_raw(Box::new(t)) as *mut ();
        let drop = |ptr| drop(unsafe { Box::from_raw(ptr as *mut T) });
        let fingerprint = Fingerprint::of::<T>();

        // Once attributes on struct literal fields are stable, do that instead.
        // https://github.com/rust-lang/rust/issues/41681
        #[cfg(not(feature = "unstable-debug"))]
        {
            Any { ptr, drop, fingerprint }
        }

        #[cfg(feature = "unstable-debug")]
        {
            let type_name = unsafe { intrinsics::type_name::<T>() };
            Any { ptr, drop, fingerprint, type_name }
        }
    }

    // This is unsafe -- caller is responsible that T is the correct type.
    pub(crate) fn view<T>(&mut self) -> &mut T {
        if self.fingerprint != Fingerprint::of::<T>() {
            self.invalid_cast_to::<T>();
        }
        let ptr = self.ptr as *mut T;
        unsafe { &mut *ptr }
    }

    // This is unsafe -- caller is responsible that T is the correct type.
    pub(crate) fn take<T>(self) -> T {
        if self.fingerprint != Fingerprint::of::<T>() {
            self.invalid_cast_to::<T>();
        }
        let ptr = self.ptr as *mut T;
        let box_t = unsafe { Box::from_raw(ptr) };
        mem::forget(self);
        *box_t
    }

    #[cfg(not(feature = "unstable-debug"))]
    fn invalid_cast_to<T>(&self) -> ! {
        panic!("invalid cast; enable `unstable-debug` feature to debug");
    }

    #[cfg(feature = "unstable-debug")]
    fn invalid_cast_to<T>(&self) -> ! {
        let from = self.type_name;
        let to = unsafe { intrinsics::type_name::<T>() };
        panic!("invalid cast: {} to {}", from, to);
    }
}

impl Drop for Any {
    fn drop(&mut self) {
        (self.drop)(self.ptr);
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Fingerprint {
    size: usize,
    align: usize,
    id: usize,
}

impl Fingerprint {
    fn of<T>() -> Fingerprint {
        Fingerprint {
            size: mem::size_of::<T>(),
            align: mem::align_of::<T>(),
            // This is not foolproof -- theoretically Rust or LLVM could
            // deduplicate some or all of these methods. But in practice it's
            // great and I am comfortable relying on this in debug mode to catch
            // bugs early.
            id: Fingerprint::of::<T> as usize,
        }
    }
}

#[test]
fn test_fingerprint() {
    assert_eq!(Fingerprint::of::<usize>(), Fingerprint::of::<usize>());
    assert_eq!(Fingerprint::of::<&str>(), Fingerprint::of::<&'static str>());

    assert_ne!(Fingerprint::of::<usize>(), Fingerprint::of::<isize>());
    assert_ne!(Fingerprint::of::<usize>(), Fingerprint::of::<&usize>());
    assert_ne!(Fingerprint::of::<&usize>(), Fingerprint::of::<&&usize>());
    assert_ne!(Fingerprint::of::<&usize>(), Fingerprint::of::<&mut usize>());

    struct A;
    struct B;
    assert_ne!(Fingerprint::of::<A>(), Fingerprint::of::<B>());
}
