//! Implementation of panics via stack unwinding
//!
//! This crate is an implementation of panics in Rust using "most native" stack
//! unwinding mechanism of the platform this is being compiled for.

#![no_std]
#![unstable(feature = "panic_unwind", issue = "32837")]
#![cfg_attr(all(target_env = "sgx", target_vendor = "mesalock"), feature(rustc_private))]

#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(nll)]
#![feature(panic_unwind)]
#![feature(raw)]
#![feature(staged_api)]
#![feature(std_internals)]
#![feature(unwind_attributes)]
#![feature(abi_thiscall)]
#![panic_runtime]
#![feature(panic_runtime)]
#![feature(rustc_attrs)]

extern crate alloc;
extern crate sgx_libc;
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
}

mod dwarf;

#[rustc_std_internal_symbol]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn __rust_panic_cleanup(payload: *mut u8) -> *mut (dyn Any + Send + 'static) {
    Box::into_raw(imp::cleanup(payload))
}

// Entry point for raising an exception, just delegates to the platform-specific
// implementation.
#[rustc_std_internal_symbol]
#[unwind(allowed)]
pub unsafe extern "C" fn __rust_start_panic(payload: usize) -> u32 {
    let payload = payload as *mut &mut dyn BoxMeUp;
    let payload = (*payload).take_box();
    imp::panic(Box::from_raw(payload))
}
