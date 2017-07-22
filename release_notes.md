# Rust SGX SDK v0.2.0

## Introduction

Baidu X-Lab provides Rust SGX SDK v0.2.0, to help develop Intel SGX enclaves in Rust programming language.

## What's New

* **Support for huge enclave memory (31.75 GB tested).** Please configure higher upper limit of heap size in Enclave.config.xml. The new [hugemem sample](https://github.com/baidu/rust-sgx-sdk/tree/master/samplecode/hugemem) is tested with Linux 16.04 + Xeon E3 1280 V5 + 64 GB memory.
* Support for newer Rust nightly build (rust version 1.20.0-nightly (nightly-2017-07-06).
* Support for the latest Intel SGX SDK v1.9.
* More support of error codes, SGX data structures, API functions.
* New trait `BytewiseEquality`, which means the equality of two corresponding object can be calculated by memory compare functions.
* New trait `ContiguousMemory`, which means the corresponding object takes up contiguous memory region. This is important to Intel SGX programming. Many APIs are re-designed to use this new trait.
* Support exception handling: `catch_unwind`, `panik` and `poisoning`.
* Add a customized implementation of Rust oom (Out-of-memory) exception handling routine. Developer can use `sgx_trts::rsgx_oom` to catch and memory allocation exceptions and unwind safely.
* More threading support. Now `park()` and `unpark()` are available for threading.
* Add support for Thread Local Storage. Currently the Thread Local Storage is only provided to enclave enforced "Bound TCS" policy.
* Add support for Rust style `Once` call.
* Add support for global data, using new macro `init_global_object`. Global data could be initiated dynamically on the first ECALL.
* Add support for Read-write lock (`sgx_tstdc::rwlock`).
* Add support for spinlock (`sgx_tstdc::spinlock`).
* Add support for CPUID (`rsgx_cpuid`).
* Add constant time memory compare API `consttime_memequal` for crypto use.
* Add API support for `sgx_get_ps_sec_prop_ex` in `sgx_tservice`.

## Known Issues and Limitations

* `cargo test` is unsupported.
* Rust-style backtrace on unexpected error is unsupported.
* Rust has recently merged a patch [(rustc: Implement the #[global_allocator] attribute)](https://github.com/rust-lang/rust/commit/695dee063bcd40f154bb27b7beafcb3d4dd775ac#diff-28f2fd684ad47d385427678d96d2dcd4) which significantly changes the behavior of liballoc. Thus `set_oom_handler` is no longer available since nightly-2017-07-07. Due to its instability, v0.2.0 Rust SGX SDK keeps using the old liballoc instead of new liballoc_system. As a result, nightly version rustc after 2017-07-06 will not successfully compile `sgx_trts`.
* For Thread Local Storage variables in "Bound TCS" enclave, destructors can be defined/implemented but never invoked automatically on thread's death.

## Coming up in future

* More sample codes and a whitepaper.
* Upcoming module `sgx_urts` will provide untrusted part support. Developer can write untrusted codes and create enclave in Rust.
* Port [rustls](https://github.com/ctz/rustls) into SGX enclave.

