# Rust SGX SDK v0.9.0 Release Notes

Almost there! Rust SGX SDK v0.9.0 is coming up as a beta version of the future v1.0.0, with the most desired `sgx::tstd` as well as many new features!

**Support Crate porting.** Porting existing Rust crates into the SGX world becomes **possible**. We have successfully ported several crates from Crates.io with little modifications. Please refer to the section 'Porting Rust Crates' for step-by-step instructions.

**Support programming untrusted components in Rust.** Now v0.9.0 provides `sgx_urts` for the untrusted world! One can write Rust codes to start and run an SGX enclave!

**Support Serialization/deserialization .** Passing structured data to the enclave is no longer a problem! Rust SGX SDK v0.9.0 comes with `sgx_serialize`, providing secure serialization and deserialization APIs!

**Support `stdin/stdout/stderr`.** Now macros like `println!` are available inside SGX enclave.

**Support `backtrace` inside SGX enclaves.** Stack backtrace inside an SGX enclave could be automatically dumped on panicking after enabling the backtrace mechanism.

**Support unit tests.** Now one can write unit tests with Rust style assertions. Please refer to the new crate `sgx_tunittest`.

Rust SGX SDK v0.9.0 supports up-to-date Rust nightly build (tested on rust version 1.22.0-nightly (3c96d40d3 2017-09-28)).

Welcome to report bugs, and we will fix them as soon as possible. Thanks for your contributions!

## New Features
* `sgx_tstd` (previously known as `sgx_tstdc`) is designed to act as `std` in SGX programming. Now `sgx_tstd` **fully** supports of `any`, `ascii` `borrow`, `boxed`, `cell`, `char`, `clone`, `cmp`, `collections`, `convert`, `default`, `error`, `f32`, `f64`, `ffi`, `fmt`, `hash`, `i8`, `i16`, `i32`, `i64`, `io`, `isize`, `iter`, `marker`, `mem`, `num`, `ops`, `option`, `panic`, `prelude`, `ptr`, `rc`, `result`, `slice`, `str`, `string`, `u8`, `u16`, `u32`, `u64`, `usize`, `vec`, `heap`, `i128`, `intrinsics`, `raw`, `u128`, and **partially** supports of `fs`, `os`, `path`, `sync` and `thread`. For details, please refer to the section 'Difference between `sgx_tstd` and Rust `std`'.
* New supports in untrusted world include `sgx_urts` (e.g., untrusted run time), `sgx_ustdio` (e.g., helper functions of stdio) and `sgx_ubacktrace` (e.g., helper functions of backtrace).
* Serialization/deserialization is supported by `sgx_serialzie`.
* Rust style PRNG interface `sgx_rand` along with its `Derive` feature support.
* Backtrace mechanism is provided by mod `backtrace` in `sgx_tstd` and `sgx_ubacktrace` + `libbacktrace`.
* Adopting the new `alloc_system` through `sgx_alloc`.

## New sample codes

In `samplecode` directory, we provide the following new code samples:

* `hello-rust` is the helloworld sample written in pure Rust.
* `backtrace` is a sample showing how to enabling backtrace mechanism inside the enclave.
* `file` shows basic usage of SGX's new file system APIs.
* `unit-test` shows the way of writing unit tests and conduct unit testing.
* `zlib-lazy-static-sample` shows how to use ported third party crates inside enclave.

In `third_party` directory, we provide six crates ported from untrusted world. 

* `inflate` a simple implementation of inflate algorithm.
* `libflate` a more complex implementation of inflate algorithm. It dependents on `adler32-rs` and `byteorder`.
* `lazy-static.rs` a widely used crate for initializing static data structures.
* `yansi` printing colorful characters on terminal.

## Programming with `sgx_tstd` in enclave

`std` is the most important and fundamental crate in Rust. Due to Intel SGX's limitations, many features in Rust's `std` are incompatible with SGX, thus not all features of `std` are available in Intel SGX enclaves.

To offer a user-friendly and secure Rust environment in SGX, we implement `sgx_tstd`. It is very much similar to Rust's `std` and easy to use. Here is a sample usage:

In Cargo.toml of your project, add the following line to include `sgx_tstd`

```
[dependency]
sgx_tstd = { path = "path/to/sgx_tstd" }
```

And add the following lines in your `lib.rs` (use `vec` as an example here):

```
extern crate sgx_tstd as std;
use std::vec::Vec;
```

One can use `sgx_tstd` in the namespace of `std` and write Rust program as usual.

But `sgx_tstd` has its own limitations. We add replace some Rust struct with SGX struct (e.g. `Mutex` now is `SgxMutex`). We rename some of these structs because the implementation of `SgxMutex` is vastly different from Rust's `Mutex`. And we want developers to be clear which mutex they are using.

Please refer to 'Difference between `sgx_tstd` and Rust `std`' for the detail of `sgx_tstd`.

## Porting Rust Crates into the SGX world

The most important thing is that the trusted world is a `no_std` world. If you see linking errors such as `f32` `f64` `panic_fmt` is already defined in `std`, it means that you are linking the Rust's `std` to the enclave. This is absolutely **wrong**. You need to check **every direct and indirect dependent crates** to see if depends on `std`.

Here is a step-by-step instruction for porting crates to the SGX enclave. To be easy, we name the crate you want to port as 'root crate'.

1. For each dependent crate of the root crate, check if it uses something like `std::vec::Vec`. Almost all crates use `std`.
2. Download the source codes of the above identified dependent crates.
3. Fix their `Cargo.toml`. In the dependency section, change the crate location to local file system (`{ path = "path/to/lib" }`)
4. Add `sgx_tstd` to `Cargo.toml` as a new dependency, e.g., `sgx_tstd = { path = "path/to/sgx_tstd" }`.
5. Edit the source code of each `lib.rs`. Add `#[no_std]` at the beginning, after all `#![]` inner attributes.
6. Add an extra statement: `extern crate sgx_tstd as std` to include `sgx_tstd` under the `std` namespace.
7. Eliminate all of the references to unsupported mod/macro/feature of `sgx_tstd`.
8. Compile.

Please look into `third_party` directory for porting samples.

## Experimental new docker image

The latest Intel SGX SDK for linux v1.9 only supports protobuf v2. Protobuf v2 has a lot of known bugs and is out-of-date. We provide an experimental dev environment with the latest protobuf v3.4.1 and a patched Intel SGX SDK v1.9. 

The docker image is on the docker hub now. Please use the following command to download it.

```
docker pull baiduxlab/sgx-rust-experimental
```

## Misc issues and hacks

* In Intel's SGX SDK, `$(SGX_SDK)/lib64/libsgx_tprotected_fs.a` and `libsgx_uprotected_fs.a` contains extra header files, which would probably cause linking problems. To resolve this, one should run the following commands to remove the header files:

```
ar d $(SGX_SDK)/lib64/libsgx_uprotected_fs.a sgx_tprotected_fs_u.h
ar d $(SGX_SDK)/lib64/libsgx_tprotected_fs.a sgx_tprotected_fs_t.h
```

In the docker environment, these two static libraries have been properly patched.

* Linking error on multiple `liblibc`. Crate `libc` is not designed for `#![no_std]` environment. Though it provides features for `no_std`, it cannot be linked to SGX enclaves. To resolve this, one should remove one of the existing `liblibc` rlib. Based on our observations, the larger one is the correct one.

In our docker environment, the extra `liblibc` is removed.

## Difference between `sgx_tstd` and Rust `std`

For now we support a subset of Rust's `std` in the trusted world. There is a full list of supported mods/macros in `sgx_tstd`

### New stuffs in `sgx_tstd`

New mods in `sgx_tstd` : `enclave`, `backtrace`, `cpuid` and `sync::spinlock`.

New macros : `global_ctors_object` and `cfg_if`.

New SGX structures (corresponding to Rust's std) : 

| Exist Structs | Rust Sgx Structs |
|---|---|
| `std::fs::File` | `sgx_tstd::fs::SgxFile` |
| `std::thread::Thread` | `sgx_tstd::thread::SgxThread` |
| `std::thread::ThreadId` | `sgx_tstd::thread::SgxThreadId` |
| `std::sync::Mutex` | `sgx_tstd::sync::SgxMutex` |
| `std::sync::MutexGuard` | `sgx_tstd::sync::SgxMutexGuard` |
| `std::sync::Condvar` | `sgx_tstd::sync::SgxCondvar` |
| `std::sync::RwLock` | `sgx_tstd::sync::SgxRwLock` |
| `std::sync::RwLockReadGuard` | `sgx_tstd::sync::SgxRwLockReadGuard` |
| `std::sync::RwLockwriteGuard` | `sgx_tstd::sync::SgxRwLockwriteGuard` |



### Fully supported mods in `sgx_tstd`

| Mod | Description |
| --- | --- |
| any  | This module implements the `Any` trait. |
| ascii | Operations on ASCII strings and characters. |
| borrow | A module for working with borrowed data. |
| boxed | A pointer type for heap allocation. |
| cell | Shareable mutable containers. |
| char | A character type. |
| clone |  The clone trait for types that cannot be 'implicitly copied'. |
| cmp | Functionality for ordering and comparison. |
| collections | Collection types. |
| convert | Traits for conversions between types. |
| default | The default trait for types which may have meaningful default values.  |
| error | Trais for working with Errors. |
| f32 | 32 bit float point support. |
| f64 | 64 bit float point support.  |
| ffi | Utilities related to FFI bindings. |
| fmt | Utilities for formatting and printing Strings. |
| hash | Generic hashing support. |
| i8 | The 8-bit signed integer type. |
| i16 | The 6-bit signed integer type. |
| i32 | The 32-bit signed integer type. |
| i64 | The 64-bit signed integer type. |
| io | Traits, helpers, and type definitions for core I/O functionality. |
| isize | The pointer-sized signed integer type. |
| iter | Composable external iteration. |
| marker | Primitive traits and types representing basic properties of types. |
| mem | Basic functions for dealing with memory.  |
| num | Additional functionality for numeric.  |
| ops | Overloadable operators. |
| option | Optional values. |
| panic | Panic support in the standard library. |
| prelude | The Rust Prelude. |
| ptr | Raw, unsafe pointers, `*const T`, and `*mut T`. |
| rc | Single-threaded reference-counting pointers. ’Rc’ stands for ‘Reference Counted’ |
| result | Error handling with the Result type. |
| slice | A dynamically-sized view into a contiguous sequence, `[T]`. |
| str | Unicode string slices. |
| string | A UTF-8 encoded, growable string. |
| u8 | The 8-bit unsigned integer type. |
| u16 | The 16-bit unsigned integer type. |
| u32 | The 32-bit unsigned integer type. |
| u64 | The 64-bit unsigned integer type. |
| usize | The pointer-sized unsigned integer type. |
| vec | A contiguous growable array type with heap-allocated content, written `Vec<T>`.|
| heap | dox |
| i128 | The 128-bit signed integer type. |
| intrinsics | Rustc compiler intrinsics. |
| raw | Contains struct definitions for the layout of compiler built-in types. |
| u128 | The 128-bit unsigned integer type. |

### Fully supported macros in `sgx_tstd`

| Macros | Description |
| --- | --- |
| assert | Ensure that a boolean expression is true at runtime. |
| assert_eq | Asserts that two expressions are equal to each other (using PartialEq). |
| assert_ne | Asserts that two expressions are not equal to each other (using PartialEq). |
| cfg | Boolean evaluation of configuration flags. |
| column | A macro which expands to the column number on which it was invoked. |
| compile_error | Unconditionally causes compilation to fail with the given error message when encountered. |
| concat | Concatenates literals into a static string slice. |
| debug_assert | Ensure that a boolean expression is true at runtime. |
| debug_assert_eq | Asserts that two expressions are equal to each other. |
| debug_assert_ne | Asserts that two expressions are not equal to each other. |
| env | Inspect an environment variable at compile time. |
| eprint | Macro for printing to the standard error. |
| eprintln | "Macro for printing to the standard error with a newline." |
| file | A macro which expands to the file name from which it was invoked. |
| format | Use the syntax described in std::fmt to create a value of type string. See std::fmt for more information. |
| format_arg | The core macro for formatted string creation & output. |
| include | Parse a file as an expression or an item according to the context. |
| include_bytes | Includes a file as a reference to a byte array. |
| include_str | Includes a utf8-encoded file as a string. |
| line | A macro which expands to the line number on which it was invoked. |
| module_path | Expands to a string that represents the current module path. |
| option_env | Optionally inspect an environment variable at compile time. |
| panic | The entry point for panic of Rust threads. |
| print | Macro for printing to the standard output. |
| println | Macro for printing to the standard output with a newline.|
| stringify | A macro which stringifies its argument. |
| thread_local | Declare a new thread local storage key of type std::thread::LocalKey. |
| concat_idents | Concatenate identifiers into one identifier. |
| try | Helper macro for reducing boilerplate code for matching Result together with converting downstream errors. |
| unimplemented | A standardized placeholder for marking unfinished code. It panics with the message “not yet implemented” when executed. |
| unreachable | A utility macro for indicating unreachable code. |
| vec | Creates a Vec containing the arguments. |
| write | Write formatted data into a buffer. |
| writeln | Write formatted data into a buffer, with a newline appended. |
### Partially supported mods/traits in `sgx_tstd`

`std::fs`, `std::os`, `std::path`, `std::sync`, `std::thread` are partially supported in `sgx_tstd`. Here we only list the unsupported parts of them. All the other parts are supported.

1. `std::fs`
Unsupported stuffs:
`fs::Dirbuilder; fs::DirEntry; fs::FileType; fs::Metadata; fs::Permissions; fs::ReadDir; fs::canonicalize; fs::create_dir; fs::create_dir_all; fs::hard_link; fs::metadata; fs::read_dir; fs::read_link; fs::remove_dir; fs::remove_dir_all; fs::rename; fs::set_permissions; fs::soft_link; fs::symlink_metadata; fs::File::sync_all; fs::File::sync_data; fs::File::set_len; fs::File::metadata; fs::File::try_clone; fs::File::set_permissions; Debug for fs::File; AsRawFd for fs::File; FromRawFd for fs::File; IntoRawFd for fs::File; FileEx for fs::File; Clone for fs::OpenOptions; Debug for fs::OpenOptions; OpenOptionsExt for fs::OpenOptions;`

2. `std::os`
Unsupported stuffs:
`os::linux; os::raw; os::unix::io; os::unix::net; os::unix::process; os::unix::raw; os::unix::thead; os::unix::fs::DirBuilderExt; os::unix::fs::DirEntryExt; os::unix::fs::FileTypeExt; os::unix::fs::MetadataExt; os::unix::fs::OpenOptionsExt; os::unix::fs::PermissionsExt;`

3. `std::path`
Unsupported stuffs:
`path::Path::maetadata; path::Path::symlink_metadat; path::Path::canonicalize; path::Path::read_link; path::Path::read_dir; path::Path::exists; path::Path::is_file; path::Path::is_dir;`

4. `std::sync`
Unsupported stuffs:
`sync::mpsc; sync::WaitTimeoutResult; sync::Condvar::wait_timeout_ms; sync::Condvar::wait_timeout;`

5. `std::thread`
Unsupported stuffs:
`thread::Builder; thread::JoinHandle; thread::park_timeout; thread::park_timeout_ms; thread::sleep; thread::sleep_ms; thread::spawn; thread::yield_now; thread::Result; thread::Thread::name; Debug for thread::Thread;`

### Unsupported stuffs in `sgx_tstd`
Mod : `std::env`, `std::net`, `std::process`, `std::time`
Macro : `select`

