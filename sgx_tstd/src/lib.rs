// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

//! # The Rust SGX SDK Standard Library
//!
//! The Rust SGX standard library is the foundation of portable Rust SGX SDK,
//! a set of minimal and battle-tested shared abstractions for the Rust SGX
//! ecosystem. Similar to Rust's libstd, it offers core types, like [`Vec<T>`] and
//! [`Option<T>`], library-defined [operations on language
//! primitives](#primitives), [standard macros](#macros), [I/O] and
//! [multithreading], among [many other things][other].
//!
//! `std` is available to all Rust crates by default, just as if each one
//! contained an `extern crate sgx_tstd as std;` import at the [crate root]. Therefore the
//! standard library can be accessed in [`use`] statements through the path
//! `std`, as in [`use std::env`], or in expressions through the absolute path
//! `::std`, as in [`::std::env::args`].

#![cfg_attr(
    all(target_env = "sgx", target_vendor = "mesalock"),
    feature(rustc_private)
)]

#![no_std]
#![needs_panic_runtime]
#![allow(non_camel_case_types)]
#![allow(unused_must_use)]
#![allow(dead_code)]
#![allow(deprecated)]
#![allow(incomplete_features)]
#![allow(unused_assignments)]

#![allow(clippy::declare_interior_mutable_const)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::mem_replace_with_default)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::new_without_default)]
#![allow(clippy::transmute_ptr_to_ptr)]
#![allow(clippy::wrong_self_convention)]

#![feature(rustc_allow_const_fn_unstable)]

#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(allocator_internals)]
#![feature(allow_internal_unsafe)]
#![feature(allow_internal_unstable)]
#![feature(array_error_internals)]
#![feature(asm)]
#![feature(assert_matches)]
#![feature(async_stream)]
#![feature(bench_black_box)]
#![feature(box_syntax)]
#![feature(c_variadic)]
#![feature(c_unwind)]
#![feature(cfg_accessible)]
#![feature(cfg_eval)]
#![feature(cfg_target_has_atomic)]
#![feature(char_error_internals)]
#![feature(char_internals)]
#![feature(concat_idents)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_format_args)]
#![feature(const_raw_ptr_deref)]
#![feature(const_trait_impl)]
#![feature(core_intrinsics)]
#![feature(custom_test_frameworks)]
#![feature(decl_macro)]
#![feature(dropck_eyepatch)]
#![feature(duration_checked_float)]
#![feature(edition_panic)]
#![feature(extend_one)]
#![feature(fn_traits)]
#![feature(format_args_nl)]
#![feature(gen_future)]
#![feature(global_asm)]
#![feature(hashmap_internals)]
#![feature(into_future)]
#![feature(int_error_internals)]
#![feature(iter_zip)]
#![feature(lang_items)]
#![feature(linked_list_remove)]
#![feature(llvm_asm)]
#![feature(log_syntax)]
#![feature(map_try_insert)]
#![feature(maybe_uninit_extra)]
#![feature(maybe_uninit_slice)]
#![feature(needs_panic_runtime)]
#![feature(negative_impls)]
#![feature(never_type)]
#![feature(new_uninit)]
#![feature(once_cell)]
#![feature(panic_info_message)]
#![feature(panic_internals)]
#![feature(panic_unwind)]
#![feature(pin_static_ref)]
#![feature(prelude_import)]
#![feature(ptr_internals)]
#![feature(rustc_attrs)]
#![feature(specialization)]
#![feature(std_internals)]
#![feature(str_internals)]
#![feature(test)]
#![feature(thread_local)]
#![feature(toowned_clone_into)]
#![feature(trace_macros)]
#![feature(try_reserve)]
#![feature(try_reserve_kind)]
#![feature(unboxed_closures)]
#![feature(vec_spare_capacity)]
#![default_lib_allocator]

// Explicitly import the prelude. The compiler uses this same unstable attribute
// to import the prelude implicitly when building crates that depend on std.
#[prelude_import]
#[allow(unused)]
use prelude::v1::*;

extern crate hashbrown;

#[allow(unused_imports)] // macros from `alloc` are not used on all platforms
#[macro_use]
extern crate alloc as alloc_crate;

// We always need an unwinder currently for backtraces
#[allow(unused_extern_crates)]
extern crate sgx_unwind;
#[cfg(feature = "backtrace")]
extern crate sgx_backtrace_sys;
#[cfg(feature = "backtrace")]
extern crate sgx_demangle;
extern crate sgx_alloc;

#[macro_use]
extern crate sgx_types;
pub use sgx_types::cfg_if;

#[macro_use]
extern crate sgx_trts;
pub use sgx_trts::{
    global_ctors_object,
    global_dtors_object,
    is_x86_feature_detected,
    is_cpu_feature_supported
};

extern crate sgx_tprotected_fs;
extern crate sgx_libc;

// The standard macros that are not built-in to the compiler.
#[macro_use]
mod macros;

// The Rust prelude
pub mod prelude;

// Public module declarations and re-exports
pub use alloc_crate::borrow;
pub use alloc_crate::boxed;
pub use alloc_crate::fmt;
pub use alloc_crate::format;
pub use alloc_crate::rc;
pub use alloc_crate::slice;
pub use alloc_crate::str;
pub use alloc_crate::string;
pub use alloc_crate::vec;
pub use core::any;
pub use core::array;
pub use core::cell;
pub use core::char;
pub use core::clone;
pub use core::cmp;
pub use core::convert;
pub use core::default;
pub use core::future;
pub use core::hash;
pub use core::hint;
pub use core::i128;
pub use core::i16;
pub use core::i32;
pub use core::i64;
pub use core::i8;
pub use core::intrinsics;
pub use core::isize;
pub use core::iter;
pub use core::marker;
pub use core::mem;
pub use core::ops;
pub use core::option;
pub use core::pin;
pub use core::ptr;
pub use core::result;
pub use core::stream;
pub use core::u128;
pub use core::u16;
pub use core::u32;
pub use core::u64;
pub use core::u8;
pub use core::usize;

pub mod f32;
pub mod f64;

#[macro_use]
pub mod thread;
pub mod ascii;
pub mod collections;
pub mod env;
pub mod error;
pub mod ffi;
pub mod sgxfs;
#[cfg(feature = "untrusted_fs")]
pub mod fs;
pub mod io;
pub mod net;
pub mod num;
pub mod os;
pub mod panic;
pub mod path;
pub mod sync;
pub mod time;
pub mod enclave;
pub mod untrusted;

pub mod lazy;

pub mod task {
    //! Types and Traits for working with asynchronous tasks.

    #[doc(inline)]
    pub use core::task::*;

    #[doc(inline)]
    pub use alloc_crate::task::*;
}

// Platform-abstraction modules
#[macro_use]
mod sys_common;
mod sys;

pub mod alloc;

// Private support modules
mod cpuid;
mod panicking;

#[cfg(not(feature = "untrusted_fs"))]
mod fs;

// The runtime entry point and a few unstable public functions used by the
// compiler
pub mod rt;

#[cfg(feature = "backtrace")]
pub mod backtrace;

pub use cpuid::*;
pub use self::thread::{
    rsgx_thread_self,
    rsgx_thread_equal
};

// Re-export macros defined in libcore.
#[allow(deprecated, deprecated_in_future)]
pub use core::{
    assert_eq, assert_ne, debug_assert, debug_assert_eq, debug_assert_ne, matches, r#try, todo,
    unimplemented, unreachable, write, writeln,
};

// Re-export built-in macros defined through libcore.
#[allow(deprecated)]
pub use core::{
    assert, assert_matches, cfg, column, compile_error, concat, concat_idents, const_format_args,
    env, file, format_args, format_args_nl, include, include_bytes, include_str, line, llvm_asm,
    log_syntax, module_path, option_env, stringify, trace_macros,
};

pub use core::primitive;

mod sealed {
    /// This trait being unreachable from outside the crate
    /// prevents outside implementations of our extension traits.
    /// This allows adding more trait methods in the future.
    pub trait Sealed {}
}
