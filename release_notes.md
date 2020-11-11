# Rust SGX SDK v1.1.3

**Supports Intel SGX SDK v2.12**

**Supports Rust nightly-2020-10-25**

**Docker images** We provide docker images: 1604-1.1.3/1804-1.1.3/2004-1.1.3/fedora31-1.1.3/centos8-1.1.3

**Please upgrade asap** Intel fixed couple of bugs in 2.12. Please upgrade as
soon as possible!

# Rust SGX SDK v1.1.2

**Supports Intel SGX SDK v2.9.1**

**Supports Rust nightly-2020-04-07**

**Docker images** We provide docker images: 1604-1.1.2/1804-1.1.2/2004-1.1.2/fedora27-1.1.2. CentOS support is unfinished. Contribution is welcome!

**sgx_signal** enables signal handling. It'll be pretty handy when debugging with `ud2` or `SIGILL` events! Please look at [signal sample](samplecode/unit-test/enclave/src/test_signal.rs) for usage.

**Removed sgx_core_futures** since Rust supports `async`/`await` in `no_std` environment.

**Bug fixes**

**Removed compiler-rt patch**

# Rust SGX SDK v1.1.1

**Supports Intel SGX SDK v2.9**

**Supports Rust nightly-2020-03-12**

**Docker images refactored** Due to the requirement of LVI mitigation, the docker image has to contain very new version of GCC/G++ and GNU binutils with LVI patch. We shipped our docker images with two options: (1) build gcc from source, or (2) use gcc from well-known repo. Please use at your own choice.

**New proc macro `sgx_align`** `sgx_align` can help with mitigate INTEL-SA-00219. Please refer to the [unit test codes](samplecode/unit-test/enclave/src/test_alignstruct.rs) for sample usage.

**Use hashbrown to replace old std::collections** We move to hashbrown v0.7 and skipped v0.6.

**sgx_core_futures** provides basic future primitive. We'll provide some samples later.

**sgx_crypto_helper** is working on both trusted/untrusted side now.

# Rust SGX SDK v1.1.0

**Supports Intel SGX SDK v2.7.1**

**Supports Rust nightly-2019-11-25**

**Fedora 27 supported** Added dockerfile for Fedora 27.

**Threading and synchronization** We implemented `thread::spawn` and ported `std::sync::mpsc`. Please look into unittest for usage.

**`is_x86_feature_detected`** We found a global feature indicator [`g_cpu_feature_indicator`](https://github.com/apache/teaclave-sgx-sdk/wiki/%60is_x86_feature_detected%60-in-Rust-SGX-SDK) and enabled `is_x86_feature_detected` without triggering `cpuid` instruction.

**New aligned allocator primitives and data structures** To mitigate [Intel-SA-00219](https://www.intel.com/content/www/us/en/security-center/advisory/intel-sa-00219.html), we provided `AlignBox<T>` for dynamic allocation, and aligned key types such as `sgx_align_key_256bit_t`, `sgx_align_key_128bit_t` for static allocation. To help understand this vulnerability, please look into the wiki article [Mitigation of Intel SA 00219 in Rust SGX](https://github.com/apache/teaclave-sgx-sdk/wiki/Mitigation-of-Intel-SA-00219-in-Rust-SGX).

**Link flag change** In this version, `-lsgx_tcxx` must be placed before `-lsgx_tstdc`. We've changed all `Makefile`s.

# Rust SGX SDK v1.0.9 Release Notes

**Supports Rust nightly-2019-08-01** in master branch (rustc 1.38.0). Stable branch would be pushed later.

**Supports Intel SGX SDK v2.6**

**CentOS 7.6 supported** Added dockerfile for CentOS 7.6.

**New sgx crates** sgx_backtrace, sgx_backtrace_sys, sgx_demangle, sgx_panic_abort, sgx_panic_unwind.

**Removed libbacktrace and new libunwind is in sgx_unwind**

**No longer requires libsgx_tcxx** unless the enclave depends on protected_fs or tse.

**Removed all local third_party crates** We forked everything and maintain them by merge bot. We'll merge the commits periodically, and provide a world map of the forked crates very soon.

**Upgrade Notes** Please update your edl files as well as the common headers. And please adjust the Xargo.toml if your project are using xargo.

**The Apache Incubator** Rust SGX SDK would be soon transferred to the Apache Incubator, as a subproject of MesaTEE. It'll be in a repo like `apache/incubator-rust-sgx-sdk`, while the former link `baidu/rust-sgx-sdk` still works well.

# Rust SGX SDK v1.0.8 Release Notes

**Supports Rust nightly-2019-05-22** in master branch (rustc 1.36.0)

**Supports Rust stable-2019-05-14** in stable branch (rustc 1.34.2)

* Bug fix in sgx_alloc. Correct the MIN_ALIGN to 8 bytes according to Intel's memory alloctor.

* Bug fix in sgx_tstd/panicking.rs. Now master branch can output panic strings correctly.

* Fix `eprintln!` support in sgx_tstd.

* New code sample: kvdb-memdb and its dependencies. Thanks to @bradyjoestar !

* Fixed all Makefiles. Put only `sgx_trts` in the "whole" linking group.

* Fixed xargo sysroots and remove unused "ghost" crates under "xargo" directory. Now there is no duplicated SDK crates in the project.

* Deprecated `sgx_tstd::Error::type_id` function. See Rust issue 60784.

* sgx_tunittest is edition now! Thanks to @elichai!

* add `fn source` in sgx_tstd::error::Error

* Fix env var bugs in sgx_libc, sgx_urts, and sgx_ustdc.

* sgx_cov leverages lcov to generate code coverage report for SGX enclave. Please refer to sgx-cov code sample for details.

# Rust SGX SDK v1.0.7 Release Notes

**Supports Intel SGX SDK v2.5**

**Supports Rust nightly-2019-04-26** in master branch (rustc 1.36.0)

**Supports Rust stable-2019-04-25** in stable branch (rustc 1.34.1)

**Refactored `sgx_tstd` to support mio**

**New sample code mio** shows how to use ported version of mio in SGX enclave.

**sgx_tunittest gives more information on return and supports closures** Thanks to @elichai!

**sgx_crypto_helper can export public key now** Please refer to static-data-distribution for usages.

**sgx_tcrypto_helper can be directly used for enclave** Thanks to @brenzi and @electronix!

**sealeddata sample supports `T` and `[T]` and serialized data structures** Thanks to @matthias-g!

**`quote_type` is configurable now in all Rust-based remote attestation sample codes** Thanks to @bradyjoestar!

**New sample code tr-mpc** Thanks to @bradyjoestar!

**New sample code: Go and Java ue-ra client** Thanks to @bradyjoestar!

**New sample code sgxcounter** shows how to use Monotonic Counter in SGX.

# Rust SGX SDK v1.0.6 Release Notes

**Add proper support to memalign in sgx_alloc** Thanks to @cbeck88.

**Use `core::mem::zeroed` to get a zero-initialized struct** Thanks to @cbeck88.

**Fix ucd-generate lazy_static dep** Thanks to @nhynes.

**Added support for closures in sgx_tunittest** Thanks to @elichai.

**Added rust-base58** Thanks to @brenzi.

# Rust SGX SDK v1.0.5 Release Notes

**Upgrade Recommended** Intel issued a security advisory [INTEL-SA-00202](https://www.intel.com/content/www/us/en/security-center/advisory/intel-sa-00202.html) and fixed the problem in Intel SGX SDK v2.4.

**Support Intel SGX SDK v2.4**. We add a [patch](https://github.com/intel/linux-sgx/pull/359) to Intel SGX SDK to fix aesm signature verification error.

**Support Rust nightly-2019-01-28** in master branch (rustc 1.34.0).

**Support Rust stable-2019-01-17** in stable branch (rustc 1.32.0).

**Removed dependency of `posix_memalign`**.

**Refactored dockerfiles**.

**New sgx_libc crate** is isolated from `sgx_trts::libc`. It provides a bunch of extra ocalls in this release.

**Renamed vendor name from unknown to mesalock** in every target json file.

**Refactored sgx_trts**.

**The net2 crate** is ported into SGX enclave. Now one can create a socket or start listening on a port in SGX enclave (with built-in ocalls).

**Mesalink support** Now one can establish a remote attestation based TLS connection to enclave using [Mesalink](https://github.com/mesalock-linux/mesalink). A working example is [here](https://github.com/mesalock-linux/mesalink/tree/master/examples/sgx_uera_client).

**New sgx_ucrypto crate** enables using Intel SGX style crypto primitives in untrusted app.

**New sgx_crypto_helper** helps serialize/deserialize RSA keypair in either untrusted app or SGX enclave.

**New code sample: hello-regex** shows how to use regex in SGX enclave.

**New code sample: static_data_distribution** shows how to use sgx_crypto_helper to statically distribute secrets to SGX enclave with dynamic RSA key provisioning.

**New code sample: net2** shows how to create a socket/listen on a port using net2 crate.

**New code sample: pcl** shows how to use Intel's Protected Code Loader to encrypt an enclave binary and launch the encrypted binary.

**Upgrade serde-rs** to 1.0.84.

**New third-party libraries ported** regex, aho-corasick, fst, memchr, memmap-rs, thread_local, ucd-generate, utf8-ranges, version_check.

**Known issue** remoteattestation sample is not working in 18.04 because it depends on old log4cpp v1.0. Please use ue-ra or mutual-ra instead.

# Rust SGX SDK v1.0.4 Release Notes

**Upgrade recommended** Rust community has fixed a [memory bug](https://blog.rust-lang.org/2018/09/21/Security-advisory-for-std.html) in [liballoc](https://github.com/rust-lang/rust/commit/8ac88d375e00c91a3db5d78852048322f88be3c1) recently. We strongly recommend to upgrade to rust-sgx-sdk v1.0.4 and use the most recent Rust releases to build it.

**Support Intel SGX SDK v2.3.1** We skip Intel SDK v2.3 due to a [logic error patched in 2.3.1](https://github.com/intel/linux-sgx/pull/313).

**Support Rust nightly-2018-10-01** in master branch

**Support Rust stable-2018-09-25** in stable branch

**New third party libraries** bit-vec, chrono, erased-serde, fxhash, nan-preserving-float, num-bigint, quick-error, raft-rs, time, webpki-roots, yasna

**mutual-ra code sample** contains an implementation of remote attestation based TLS channel between enclaves. The algorithm comes from Intel's [paper](https://github.com/cloud-security-research/sgx-ra-tls/blob/master/whitepaper.pdf).

**ue-ra code sample** contains an implementation of remote attestation based TLS channel between untrusted party and enclave, use the same algorithm above.

**switchless code sample** shows how to use the new **Switchless** model provided by Intel.

**Refactored dockerfile** Since Intel has provided support to Ubuntu 18.04, we could remove the experimental docker image. Now we provide docker images for ubuntu 16.04 and 18.04 with both Rust nightly and stable releases.

**AI Model serialize/deserialize in rusty-machine** Resolved in [issue 35](https://github.com/baidu/rust-sgx-sdk/issues/35). One can serialize a rusty-machine model into a json string and deserialize from it.

**Third party crates upgraded/discontinued** Upgraded ring/webpki/rustls, wasmi/wabt-rs-core. Removed lazy-static, parity-wasm and untrusted because these crates supports `no_std` and could be used directly from crates.io.


# Rust SGX SDK v1.0.1 Release Notes

**Support Intel SGX SDK v2.2**

**Support Rust nightly-2018-07-16**

**Support Rust stable-2018-07-10**

**New third party libraries** bytes, http, iovec, rust-crypto, rust-fnv and rust-threshold-secret-sharing.

**New code sample** Thanks to @davidp94 for the secretsharing code sample.


# Rust SGX SDK v1.0.0 Release Notes

Baidu X-Lab provides Rust SGX SDK that is a bundle of basic libraries, scripts and ported libraries for developing Intel SGX programs in Rust programming language. Based on this SDK, developers could easily build up their SGX programs in Rust. Rust SGX SDK provides the strongest defence and helps protect the secret data reside in an enclave effectively even when the OS is compromised. It is important to real world data privacy and cloud security. Since the first day of open source, we have recevied many recommendations and supports from both academic and industry. Today, we are proudly releasing the 1.0.0 version of Rust SGX SDK, indicating that Rust SGX SDK is becoming stable and ready for production.

Intel SGX is being well adopted by industry, such as Microsoft, Ali cloud and IBM, which indicates that SGX's ability for trusted computing and data protection has been accepted by giant companies and the software stack of Intel SGX is becoming more and more critical. Ideally, the SGX application should guarantee safety from the first line of its code, instead of consumpting tremenduous of engineer-months for code auditing and fuzzing. Thus, C/C++ is not the first choice of programming language for Intel SGX applications due to the lack of memory safety guarantees. To this end, we proposed **Rust SGX SDK** which brings the best practice of memory safety to SGX projects, and reduces the workload of developing flawless SGX projects significantly. Based on this, we can leverage new techniques such as [**Non-bypassable Security Paradigm**](documents/nbsp.pdf) to assist the formal verification of critical security attributes on large projects, which is believed to be the state-of-art of practical application security guarantee. Apart from the C/C++ SDK provided by Intel, Rust SGX SDK is the only recommended SDK listed on Intel SGX's [homepage](https://software.intel.com/en-us/sgx).

From v1.0.0, Rust SGX SDK is heading towards stability and production. As a proof of concept, we provide a solution to the classic Private-Set-Intersection problem. PSI is a cryptographic technique that allows two parties to compute the intersection of their sets without revealing anything except the intersection. The PSI solution is very useful in many cases such as threat intelligence exchanging and sharing. In this proof of concept, we build a fair, trusted, reliable and attestable arbiter which can compute the intersection set with almost zero overhead and guarantee safety and security. In addition, the PSI algorithm is side channel resistant.

What's more, we provide a set of ported in-enclave WebAssembly interpreter and code samples. The support of WebAssembly (wasm) in Rust SGX SDK is an experimental feature in this version. As the hottest target platform, WebAssembly has been supported by major programming languages and compilers. Microsoft, Google, Apple and Mozilla support WebAssembly in their browser's Javascript engines. LLVM, Rust and Go provide experimental wasm as target platform and Parity has released v1.10 recently to support [Wasm Smart Contracts](https://paritytech.io/parity-1-10-opportunity-released/). With the help of SGX WebAssembly interpreter, executing programs written in major programming languages and smart contract is within a stone's throw.

Good news! Rust SGX SDK proposal has been adopted by [RustFest'18](https://paris.rustfest.eu/schedule/) and we'll present this work in Paris this week!


**WebAssembly interpreter** We port the Parity's [wasmi](https://github.com/paritytech/wasmi) to Intel SGX (see ported third party libraries at [parity-wasm](third_party/parity-wasm)/[wabt-rs-core](third_party/wabt-rs-core)/[wasmi](third_party/wasmi) and provide the [wasmi code sample](samplecode/wasmi). The sample code shows how to use the ported WebAssembly interpreter to passes all 70 cases in [WebAssembly testsuite](https://github.com/WebAssembly/testsuite/tree/c538faa43217146f458b9bc2d4b704d0a4d80963)! . We put the ported interpreter inside the SGX enclave and provide a well-defined enclave interface for passing WebAssembly codes as input and get its results in the untrusted world. With the ported WebAssembly interpreter, one can easily execute wasm codes and protect its data safely using Intel SGX and benefits from Rust's memory safety guarantees!

**Private set intersection sample** As a best use case of Intel SGX, we provide a sample solution of Private-Set-Intersection in [psi](samplecode/psi) code sample. It is derived from the remote attestation sample and can solve the two-party private-set-intersection problem perfectly and **resists side-channel attacks**!

**Moving to rust-stable** From v1.0.0, rust-sgx-sdk is going to be more stable and prepared for production. So stable branch of Rust is the best choice for the future of rust-sgx-sdk. In this version, we support the most recent Rust stable toolchain (stable-2018-05-10) in [rust-stable](https://github.com/baidu/rust-sgx-sdk/tree/rust-stable) branch and we are not catching up with the most recent nightly build due to a series of changes and unfinished codes reside in libstd and only support nightly-2018-04-12 in the master. We **strongly recommend** using the [rust-stable](https://github.com/baidu/rust-sgx-sdk/tree/rust-stable) branch for better stability and production use.

**Support Intel SGX SDK v2.1.3**

**Updated all docker images** All [sgx-rust](https://hub.docker.com/r/baiduxlab/sgx-rust/)/[sgx-rust-experimental](https://hub.docker.com/r/baiduxlab/sgx-rust-experimental/)/[sgx-rust-stable](https://hub.docker.com/r/baiduxlab/sgx-rust-stable/) are updated accordingly. If you met problems similar to ["Docker pull failed with unauthorized: authentication required"](https://github.com/baidu/rust-sgx-sdk/issues/4), please check your network or wait for the service to recover.

# Rust SGX SDK v0.9.8 Release Notes

**New branch rust-stable** We provide a new branch to support stable channel of Rust in a new branch 'rust-stable'. It contains modified libraries and a customized xargo. The customized cargo allows Rust stable to compile sysroot by demonstrating `RUSTC_BOOTSTRAP` as a env var. We provide a new docker image `baiduxlab/sgx-rust-stable` as long as its [dockerfile](dockerfile/rust-stable).

**Support Intel SGX SDK v2.1.2**

**Support Rust nightly 2018-03-16**

**Provide APIs against spectre attack** We provide `sgx_trts::{rsgx_lfence,rsgx_sfence,rsgx_mfence}` to help developers stop speculative execution on demand. We urge SGX developers to look at Intel's latest development [guide](https://software.intel.com/sites/default/files/managed/e1/ec/SGX_SDK_Developer_Guidance-CVE-2017-5753.pdf) and another [guide](https://software.intel.com/sites/default/files/managed/e1/ec/180309_SGX_SDK_Developer_Guidance_Edger8r.pdf). To defend against spectre, developers **must** rewrite their enclaves according to the guidance from Intel. We show how to rewrite SGX enclave to defend against spectre in TLS client/server and local attestation code samples.

**New API `rsgx_is_enclave_crashed`** We provide `sgx_trts::rsgx_is_enclave_crashed` corresponding to a new feature of Intel SGX SDK 2.1.2.

**rust-protobuf** We provide a ported [protobuf](https://crates.io/crates/protobuf) library for SGX enclave at [protobuf](third_party/protobuf). And we provide an example showing how to use it at [protobuf code sample](samplecode/protobuf). Attention: please install the rust-protobuf compiler by `cargo install protobuf --vers=1.4.4` before build the sample project.

# Rust SGX SDK v0.9.7 Release Notes
**Provide `sgx_tstd::untrusted` namespace** v0.9.7 provides `sgx_tstd::untrusted::{fs,path,time}` which are related to ocall functions. They are always enabled no matter `untrusted_fs` or `untrusted_time` feature is enabled or not. The major concern of providing such a namespace is that we want the developer to know they are invoking ocall related functions that brings **untrusted data** into the **trusted** execution engine. For the best security practice, explicitly importing from `sgx_tstd::untrusted` is better than enabling feature in `Cargo.toml`. We stress that `untrusted_fs` and `untrusted_time` features are designed to be **contingency plans** and should only be enabled when porting a very complex Rust crate to a Rust-SGX enclave.

**Rename feature `untrusted_net` to `net`** `net` is well-known as untrusted and we believe `net` is a better choice and similar to other features: such as `backtrace` and `stdio`.

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

