# Rust SGX SDK v0.9.7 Release Notes
**Provide `sgx_tstd::untrusted` namespace** v0.9.7 provides `sgx_tstd::untrusted::{fs,path,time}` which related to ocall functions. They are always enabled no matter `untrusted_fs` or `untrusted_time` feature is enabled or not. The major concern of providing such a namespace is that we want the developer to know they are invoking ocall related functions which brings **untrusted data** into the trusted execution engine. For the best security practice, explicitly importing from `sgx_tstd::untrusted` is better than enabling feature in `Cargo.toml`. We stress that `untrusted_fs` and `untrusted_time` features are designed to be **contingency plans** and should only be enabled when porting an very complex Rust crate to Rust-SGX enclaves.

**Rename feature `untrusted_net` to `net`** `net` is well-known as untrusted and we think `net` is a better choice and similar to current features: `backtrace` and `stdio`.

# Rust SGX SDK v0.9.6 Release Notes
**Support latest Rust nightly build (nightly-2018-02-05-x86_64-unknown-linux-gnu)**

**Security enhancement** Added three features for `sgx_tstd`: `untrusted_fs` `untrusted_time` `untrusted_net` to control the insecure ocall interface. By default, io-related features in `fs/time/net` are **DISABLED**. To enable them, please add feature declarations such as `features = ["untrusted_fs"]` for sgx_tstd in `Cargo.toml`. All sample codes and third party libraries are updated accordingly. Note that data from unstrusted `fs/time` are **UNTRUSTED**and thus use them **AT YOUR OWN RISK**. Data from `net` are well-known as untrusted and need validation instinctively. We strongly recommend our TLS termination for network access, instead of using `net` directly.

**Refined sgxtime and support sgxcounter** Moved the trusted time service to `sgx_tservice::sgxtime` and implemented the monotonic counter in `sgx_tservice::sgxcounter`.

# Rust SGX SDK v0.9.5 Release Notes
**Support latest Rust nightly build (nightly-2018-01-19-x86_64-unknown-linux-gnu)**

**Xargo support** Rust SGX SDK v0.9.5 provides `xargo` support with a target `x86_64-unknown-linux-sgx`. To compile a crate using `xargo`, add a corresponding json config and make appropirate changes to the source code, then compile it with `xargo build -target x86_64-unknown-linux-sgx --release`. Porting is easier! Please refer to the ported [third-party libraries](third_party/) for more details.

**Network access support** We port part of `std::net` to `sgx_tstd::net`. Now `sgx_tstd::net` supports most of socket functions by using 12 ocalls (defined in `edl/sgx_net.edl`).

**Rustls, webpki and ring for TLS support** We port the most famous TLS implementation [rustls](https://github.com/ctz/rustls), along with its dependnecy [webpki](https://github.com/briansmith/webpki) and crypto library [ring](https://github.com/briansmith/ring) into Rust-SGX world. And we provide a pair of TLS client/server application code samples. Please reference to tls sample codes for detail.

**File system access (sgx_tstd::fs and sgx_tstd::sgxfs) support** We port part of `std::fs` to `sgx_tstd::fs` for normal linux files. Also, we provide `sgx_tstd::sgxfs` to support Intel's `protected_fs`, an encrypted file access mechanism.

**Time (sgx_tstd::time and sgx::tservice::SgxTime) support** We port `std::time` to `sgx_tstd::time` and it provides untrusted local time. We implement `sgx::tservice::SgxTime` for the Intel ME based trusted timestamp. To use `SgxTime`, the [iClsclient library](https://software.intel.com/en-us/sgx-sdk/download) and [Dynamic Application Loader (DAL) Host Interface (aka JHI)](https://github.com/intel/dynamic-application-loader-host-interface) are required. Please reference to [sgxtime usage](documents/sgxtime.md) for detail.

**Environment variable operation (sgx_tstd::env) support** We port part of `std::env` to `sgx_tstd::env` to support setting/getting environment variables.

## New third-party libraries

All of the third-party libraries could be compiled by `make` or `XARGO_SGX=1 make`. In this release, we have the following new libraries ported.

1. bincode
2. dtoa
3. heapsize
4. itoa
5. linked-hash-map
6. log
7. ring
8. rust-base64
9. rust-serialize
10. rustls
11. safemem
12. sct
13. serde-rs
14. webpki


## About xargo's sysroot

`xargo` would generate a *sysroot*, including all basic libraries. In the past, everytime a Rust-SGX project is compiled via `make`, the basic Rust-SGX runtime would be compiled. Now, if we use `xargo` to compile (`XARGO_SGX=1 make`), only the **first time** xargo builds the sysroot and saves them in Rust's directory and the basic Rust-SGX libraries would be re-used later.

The current sysroot includes:
1. libcompiler_builtins
2. libcore
3. liblibc
4. libpanic_abort
5. libpanic_unwind
6. libsgx_alloc
7. libsgx_rand
8. libsgx_serialize
9. libsgx_tcrypto
10. libsgx_tdh
11. libsgx_tkey_exchange
12. libsgx_tprotected_fs
13. libsgx_trts
14. libsgx_tse
15. libsgx_tseal
16. libsgx_tservice
17. libsgx_tunittest
18. libstd
19. libstd_unicode
20. libunwind

# Rust SGX SDK v0.9.1 Release Notes

**Support Intel SGX SDK 2.0 for Linux** Intel released Intel SGX SDK 2.0 for Linux recently and we upgraded Rust SGX SDK to support it.

**Support latest Rust nightly build (nightly-2017-11-29-x86_64-unknown-linux-gnu)** We upgraded `sgx_tstd` to support the latest Rust nightly release. On Nov 9th, `librand` had been removed from `libstd` in this [commit](https://github.com/rust-lang/rust/commit/6bc8f164b09b9994e6a2d4c4ca60d7d36c09d3fe) and we did the same change on `sgx_tstd` as well as some other minor changes.

**Provide rusty-machine in Intel SGX** We ported the most popular machine learning library [rusty-machine](https://github.com/AtheMathmo/rusty-machine) to Intel SGX, along with its examples. Please refer to the [Rust SGX version rusty-machine](https://github.com/baidu/rust-sgx-sdk/tree/master/third_party/rusty-machine) and the machine learning code samples for more details.

# Rust SGX SDK v0.9.0 Release Notes

Almost there! Rust SGX SDK v0.9.0 is coming up as a beta version of the future v1.0.0, with the most desired `sgx_tstd` as well as many new features!

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

But `sgx_tstd` has its own limitations. We replace some Rust struct with SGX struct (e.g. `Mutex` now is `SgxMutex`). We rename some of these structs because the implementation of `SgxMutex` is vastly different from Rust's `Mutex`. And we want developers to be clear which mutex they are using.

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

