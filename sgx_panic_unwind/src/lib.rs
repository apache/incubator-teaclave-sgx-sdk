//! Implementation of panics via stack unwinding
//!
//! This crate is an implementation of panics in Rust using "most native" stack
//! unwinding mechanism of the platform this is being compiled for. This
//! essentially gets categorized into three buckets currently:
//!
//! 1. MSVC targets use SEH in the `seh.rs` file.
//! 2. Emscripten uses C++ exceptions in the `emcc.rs` file.
//! 3. All other targets use libunwind/libgcc in the `gcc.rs` file.
//!
//! More documentation about each implementation can be found in the respective
//! module.

#![no_std]
#![unstable(feature = "panic_unwind", issue = "32837")]
#![cfg_attr(
    all(target_env = "sgx", target_vendor = "mesalock"),
    feature(rustc_private)
)]

#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(panic_unwind)]
#![feature(staged_api)]
#![feature(std_internals)]
#![feature(abi_thiscall)]
#![feature(rustc_attrs)]
#![panic_runtime]
#![feature(panic_runtime)]
#![feature(c_unwind)]

extern crate alloc;
extern crate sgx_unwind;

use alloc::boxed::Box;
use core::any::Any;
use core::panic::BoxMeUp;

#[path = "gcc.rs"]
mod imp;

extern "C" {
    /// Handler in libstd called when a panic object is dropped outside of
    /// `catch_unwind`.
    fn __rust_drop_panic() -> !;

    /// Handler in libstd called when a foreign exception is caught.
    fn __rust_foreign_exception() -> !;
}

/// # Safety
#[rustc_std_internal_symbol]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn __rust_panic_cleanup(payload: *mut u8) -> *mut (dyn Any + Send + 'static) {
    Box::into_raw(imp::cleanup(payload))
}

/// # Safety
// Entry point for raising an exception, just delegates to the platform-specific
// implementation.
#[rustc_std_internal_symbol]
pub unsafe fn __rust_start_panic(payload: *mut &mut dyn BoxMeUp) -> u32 {
    let payload = Box::from_raw((*payload).take_box());

    imp::panic(payload)
}
