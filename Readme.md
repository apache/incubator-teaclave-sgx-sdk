![Rust SGX Logo](rustsgx.png)

[![Gitter](https://badges.gitter.im/rust-sgx-sdk/community.svg)](https://gitter.im/rust-sgx-sdk/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)

# Rust SGX SDK
Rust SGX SDK helps developers write Intel SGX applications in Rust programming language. [[CCS'17 Paper pdf]](documents/ccsp17.pdf) [[CCS'19 Paper pdf]](https://dingelish.com/ccs19.pdf)

Good News! Our paper "Building and Maintaining a Third-Party Library Supply Chain for Productive and Secure SGX Enclave Development" has been accepted by ICSE'20 SEIP track. See you guys in Seoul!

We open-sourced [gbdt-rs](https://github.com/mesalock-linux/gbdt-rs), a light-weight, amazingly fast, memory safe, and trustworthy gradient boosting decision tree implementation and the [paper](documents/gbdt.pdf) has been accepted by IEEE S&P'19! It is optimized for SGX!

To achieve better security, we recommend developers to apply [Non-bypassable Security Paradigm (NbSP)](https://github.com/apache/teaclave-sgx-sdk/blob/master/documents/nbsp.pdf) to the system design and implementation.

To help understand this project and know how to use it, we are writing some [wiki](https://github.com/apache/teaclave-sgx-sdk/wiki) articles. Please [send me an email](mailto:rustsgx@gmail.com?subject=Wiki%20page%20needed:) if you'd like to see other topics. We'll add it as soon as possible.

Current wiki pages:

* [Mitigation of Intel SA 00219 in Rust SGX](https://github.com/apache/incubator-mesatee-sgx/wiki/Mitigation-of-Intel-SA-00219-in-Rust-SGX)

* [`is_x86_feature_detected` in Rust SGX SDK](https://github.com/apache/incubator-mesatee-sgx/wiki/%60is_x86_feature_detected%60-in-Rust-SGX-SDK)

* [The World of Forked crates](https://github.com/apache/teaclave-sgx-sdk/wiki/The-World-of-Forked-crates) introduces the forked crate ecosystem, and provides some guidelines and usage, and show how we secure them.

* [Setup gdb 7.11 on Ubuntu 18.04 for VSCode sgx-gdb remote debugging](https://github.com/apache/teaclave-sgx-sdk/wiki/Setup-gdb-7.11-on-Ubuntu-18.04-for-VSCode---sgx-gdb-remote-debugging) If you encounter errors like `gdb.error: syntax error in expression, near )0x7ffff4127370 = 0.`, probably you need to follow this instruction to setup gdb 7. Thanks to @akoskinas for this great instruction!

* [Performance Optimization Tips](https://github.com/apache/teaclave-sgx-sdk/wiki/Performance-Optimization-Tips)

* [Use VSCode rls+rust-analysis+sgx-gdb for graphic developing (not in docker)](https://github.com/apache/teaclave-sgx-sdk/wiki/Use-VSCode---rls---rust-analysis---sgx-gdb-for-graphic-developing-(not-in-docker))

* [Debugging local enclave in docker](https://github.com/apache/teaclave-sgx-sdk/wiki/Debugging-a-local-Rust-SGX-enclave-in-docker-with-sgx-gdb)

* Everything about [environment setup](https://github.com/apache/teaclave-sgx-sdk/wiki/Environment-Setup)

## v1.1.3

Supports Intel SGX SDK v2.12, and Rust nightly-2020-10-25. Added support to Ubuntu 20.04. **We strongly recommend users to upgrade to Intel SGX SDK v2.12 and drivers to DCAP 1.36.2 and OOT 2.11.0.** [release_notes](release_notes.md).

## v1.1.2

Supports Intel SGX SDK v2.9.1, and Rust nightly-2020-04-07. v1.1.2 provides a handy crate `sgx_signal`, which enables signal capture. One can easily find the place where exception happens and finally triggered `ud2`. And we added `Backtrace::capture` in sgx_tstd. With the help of Intel SGX SDk v2.9.1's patch, dtor of thread local storage finally works on regular SGX thread and pthread thread. Removed sgx_core_futures since Rust is supporting `async`/`await` in `no_std` environment. Please refer to [release_notes](release_notes.md) for more details.

## v1.1.1

Supports Intel SGX SDK v2.9, and Rust nightly-2020-03-12. v1.1.1 contains a bunch of bug fix and new proc macro `sgx_align` to help with aligning given structure. For LVI migigation, it only works on C/C++ parts (EDL headers/Intel's libs) and supports both two modes: `MITIGATION-CVE-2020-0551=LOAD` or `MITIGATION-CVE-2020-0551=CF`. To enable it, one need `env "MITIGATION-CVE-2020-0551=LOAD"` to set this environment variable. For detailed information, please refer to [release_notes](release_notes.md) for more details.

## v1.1.0

Supports Intel SGX SDK v2.7.1, and Rust nightly-2019-11-25. v1.1.0 brings up dynamic static supports by `thread::spawn`, and almost everything of `std::sync`. Also v1.1.0 benefits from Intel SGX SDK's aligned memory allocation primitives to mitigate [INTEL-SA-00219](https://github.com/apache/incubator-mesatee-sgx/wiki/Mitigation-of-Intel-SA-00219-in-Rust-SGX). Besides, we enabled [`is_x86_feature_detected!`](https://github.com/apache/incubator-mesatee-sgx/wiki/%60is_x86_feature_detected%60-in-Rust-SGX-SDK) by parsing a hidden global CPU feature indicator initialized by Intel SGX urts/trts. And we provided Dockerfile for Fedora 27. For detailed information, please refer to [release_notes](release_notes.md) for more details.

## v1.0.9 Release

Supports Intel SGX SDK v2.6, and Rust nightly-2019-08-01. Bumps everything to edition. Removed third_party directory since we have all of those dependencies forked and maintained with merge bot. Since Intel SGX SDK v2.6 imports some breaking changes in global thread metata, thread local features of v1.0.9 is not works on Intel SGX SDK v2.5. EDL and common headers are changed respectively. For detailed information, please refer to [release_notes](release_notes.md) for more details.

## v1.0.8 Release

Supports the most recent Rust nightly (nightly-2019-05-22) and Rust stable (stable-2019-05-14). Code coverage support has been added to sgx_cov. Bug fixes in memory allocator and panicking routines. New third party libraries to support kvdb-memorydb. Please refer to [release_notes](release_notes.md) for more details.

## v1.0.7 Release

Supports Intel SGX SDK v2.5. Master branch supports Rust nightly build (nightly-2019-04-26) and stable branch supports Rust stable build (stable-2019-04-25).  Refactored `sgx_tstd` to support `mio`. More sample codes added, including Java/Go clients for ue-ra (Thanks to @bradyjoestar)!. And we are maintaining forks of popular crates on Github organization [mesalock-linux](https://github.com/mesalock-linux). The ported crates are syncing with the original crates with the help of [Pull](https://pull.now.sh) bot and we manually port almost all tests from the original crates to test if the ported crate works well in SGX. Please refer to [release_notes](release_notes.md) for further details.

We changed the built-in EDL files. Please carefully upgrade your EDL files on `import` statements. If you encountered any problems during compilation, please create issue and let me know. Thanks!

**ATTENTION**: (Ubuntu Channel) Starts from Intel SGX SDK 2.8, `aesmd` requires a environment variable to start. If you are using docker, please start `aesmd` as:
```
LD_LIBRARY_PATH=/opt/intel/sgx-aesm-service/aesm /opt/intel/sgx-aesm-service/aesm/aesm_service
```

Starts from Intel SGX SDK 2.5, `aesmd` requires a environment variable to start. If you are using docker, please start `aesmd` as:
```
LD_LIBRARY_PATH=/opt/intel/libsgx-enclave-common/aesm /opt/intel/libsgx-enclave-common/aesm/aesm_service
```

(CentOS Channel) As of 2.6, CentOS branch of Intel SGX SDK is still in format of bin executable. Please start the `aesmd` as past:
```
LD_LIBRARY_PATH=/opt/intel/sgxpsw/aesm /opt/intel/sgxpsw/aesm/aesm_service
```

## v1.0.6 Release
Fix bugs in sgx_alloc, sgx_types, ucd-generate and improve sgx_tunittest. Added rust-base58. Thanks to @elichai, @cbeck88, @brenzi and @nhynes.

## v1.0.5 Release
This version supports Rust nightly build (nightly-2019-01-28, v1.34.0) in the master branch and the most recent stable build (stable-2019-01-16, v1.32.0) in the rust-stable branch. It supports the latest Intel SGX SDK **v2.4.0** and Ubuntu Linux 16.04+18.04. We provide support to Intel's Protected Code Loader. We provide sgx_ucrypto and sgx_crypto_helper for using SGX-style crypto primitives in untrusted app and RSA keypair serialization/deserialization in both trusted and untrusted programs. We re-organize ocall related interfaces and provide them in a new crate sgx_libc with a bunch of new ocall functions. In addition, we port net2 to SGX. Please refer to [release_notes](release_notes.md) for further details.

## v1.0.4 Release
This version supports Rust nightly build (nightly-2018-10-01) in the master branch and the most recent stable build (stable-2018-09-25) in the rust-stable branch. It supports the latest Intel SGX SDK **v2.3.1** and Ubuntu Linux 18.04. It now contains further third party libraries including: bit-vec, chrono, erased-serde, fxhash, nan-preserving-float, num-bigint, quick-error, raft-rs, time, webpki-roots, and yasna. Some third party libraries, like untrusted, parity-wasm and lazy-static, are removed because they support `no_std` and can be used directly from crates.io. We strongly recommend developers upgrade to v1.0.4 and use the most recent Rust release to build it due to the [Security advisory for the standard library](https://blog.rust-lang.org/2018/09/21/Security-advisory-for-std.html). Please refer to [release_notes](release_notes.md) for further details.

## v1.0.1 Release
This version supports the Rust nightly build (nightly-2018-07-16) in master branch and the most recent Rust stable build (stable-2018-07-10). And it supports the latest Intel SGX SDK **v2.2**. New third party libraries include: bytes, http, iovec, rust-crypto, rust-fnv and rust-threshold-secret-sharing. New code sample 'secretsharing' and 'rust-threshold-secret-sharing' is provided by @davidp94. Please refer to [release_notes](release_notes.md) for further details.

## v1.0.0 Release
We proudly announce v1.0.0 of rust-sgx-sdk! We port Parity's [Webassembly Interpreter](https://github.com/paritytech/wasmi) to Intel SGX and provide a full functional in-enclave [wasmi sample](samplecode/wasmi), and a [sample solution](samplecode/psi) of two-party private-set-intersection resisting side-channel attacks! From this version, we start to support most recent stable branch of Rust instead of nightly for better stability and future production use. Thus, the [stable branch](https://github.com/apache/teaclave-sgx-sdk/tree/rust-stable) of v1.0.0 supports the most recent Rust stable toolchain (1.26.0 stable-2018-05-07), while the master only supports Rust nightly toolchain of nightly-2018-04-11. Please refer to [release_notes](release_notes.md) for further details.

## v0.9.8 Release
This version provides security updates regards to recent Spectre attacks in Intel SGX, and supports **Rust stable (2018-03-01)** (in branch named 'rust-stable'). It contains support of [Intel SGX SDK 2.1.2](https://download.01.org/intel-sgx/linux-2.1.2/) and a series of API functions to stop speculative execution on demand. In addition, we provide a ported version of [rust-protobuf](https://crates.io/crates/protobuf) v1.4.4. Please refer to [release_notes](release_notes.md) for further details.

## v0.9.7 Release
This version provides a new namespace: `sgx_tstd::untrusted`, including `sgx_tstd::untrusted::fs` `sgx_tstd::untrusted::time` and `sgx_tstd::untrusted::path`, providing supports to operation to ocalls in a **untrusted** namespace. The **untrusted** namespace is always enabled no matter `untrusted_*` is set or not. We **urge** the developers to use the `sgx_tstd::untrusted` namespace to port their crates, instead of using the `untrusted_` series of features. Also, we renamed the `untrusted_net` feature to `net` for feature name unification. Please refer to [release_notes](release_notes.md) for further details.

## Run Rust SGX applications in Mesalock Linux
[MesaLock Linux](https://github.com/mesalock-linux/mesalock-distro) is a general purpose Linux distribution which aims to provide a safe and secure user space environment. Now we can run Rust SGX applications in Mesalock Linux within a few steps. Please refer to the [tutorial](documents/sgx_in_mesalock_linux.md) for details.

## Requirement

Ubuntu 16.04 or 18.04

[Intel SGX SDK 2.9.1 for Linux](https://01.org/intel-software-guard-extensions/downloads) installed

Docker (Recommended)


## Configuration

The docker image now supports Intel ME. If you need it, please refer to the sgxtime [readme](documents/sgxtime.md) for instructions.

### Native without docker (Not recommended)

Install Intel SGX driver and SDK first. And refer to [Dockerfile](dockerfile/Dockerfile.1604.nightly) or stable branch [Dockerfile](Dockerfile.1604.stable) to setup your own native Rust-SGX environment.

### Using docker (Recommended) without ME support

* As of v1.1.3, we provide 5 docker images: `baiduxlab/sgx-rust:1604-1.1.3` `baiduxlab/sgx-rust:1804-1.1.3` `baiduxlab/sgx-rust:2004-1.1.3` `baiduxlab/sgx-rust:fedora31-1.1.3`, `baiduxlab/sgx-rust:centos8-1.1.3`. The `latest` tag pins on `baiduxlab/sgx-rust:1804-1.1.3`.

First, make sure Intel SGX Driver for 2.11 is installed and functions well. `/dev/isgx` should appear. If you use DCAP driver, please ensure `/dev/sgx/enclave` and `/dev/sgx/provision` both appear.

`$ docker pull baiduxlab/sgx-rust`

Third, start a docker with sgx device support and the Rust SGX SDK.

`$ docker run -v /your/path/to/rust-sgx:/root/sgx -ti --device /dev/isgx baiduxlab/sgx-rust`

If you use DCAP, the command is:

`$ docker run -v /your/path/to/rust-sgx:/root/sgx -ti --device /dev/sgx/enclave --device /dev/sgx/provision baiduxlab/sgx-rust`

Next, start the aesm service inside the docker

`root@docker:/# LD_LIBRARY_PATH=/opt/intel/sgx-aesm-service/aesm/ /opt/intel/sgx-aesm-service/aesm/aesm_service &`

Finally, check if the sample code works

`root@docker:~/sgx/samplecode/helloworld# make`

`root@docker:~/sgx/samplecode/helloworld# cd bin`

`root@docker:~/sgx/samplecode/helloworld/bin# ./app`

## Build the docker image by yourself

Make sure Intel SGX SDK is properly installed and service started on the host OS. Then `cd dockerfile` and run `docker build -t rust-sgx-docker -f Dockerfile.1604.nightly .` to build.

## Use simulation mode for non SGX-enabled machine (includes macOS)

Intel provides a simulation mode so you can develop on regular machines, by building the enclave app using the libraries `sgx_urts_sim`, `lsgx_uae_service_sim`, `sgx_trts_sim`, `sgx_tservice_sim`.

First, pull the docker image. If you'd like to work on stable branch of Rust and `rust-stable` branch of this SDK, please pull `baiduxlab/sgx-rust-stable` instead.

`$ docker pull baiduxlab/sgx-rust`

Second, start a docker with the Rust SGX SDK.

`$ docker run -v /your/path/to/rust-sgx:/root/sgx -ti baiduxlab/sgx-rust`

But when building any sample code, set the `SGX_MODE` to `SW` in `Makefile`.

`root@docker:~/sgx/samplecode/helloworld# vi Makefile`

Replace `SGX_MODE ?= HW` with `SGX_MODE ?= SW`

or run `export SGX_MODE=SW` in your terminal.

Finally, check if the sample code works

`root@docker:~/sgx/samplecode/helloworld# make`

`root@docker:~/sgx/samplecode/helloworld# cd bin`

`root@docker:~/sgx/samplecode/helloworld/bin# ./app`

If not set, you could get an error:
```
Info: Please make sure SGX module is enabled in the BIOS, and install SGX driver afterwards.
Error: Invalid SGX device.
```

# Documents

The online documents for SDK crates can be found [here](https://dingelish.github.io/).

We recommend to use `cargo doc` to generate documents for each crate in this SDK by yourself.  The auto generated documents are easy to read and search.

# Sample Codes

We provide eighteen sample codes to help developers understand how to write Enclave codes in Rust. These codes are located at `samplecode` directory.

* `helloworld` is a very simple app. It shows some basic usages of argument passing, Rust string and ECALL/OCALLs.

* `crypto` shows the usage of crypto APIs provided by Intel SGX libraries. It does some crypto calculations inside the enclave, which is recommended in most circumstances.

* `localattestation` is a sample ported from the original Intel SGX SDK. It shows how to do local attestation in Rust programming language.

* `sealeddata` sample shows how to seal secret data in an enclave and how to verify the sealed data.

* `thread` sample is a sample ported from the original Intel SGX SDK, showing some basic usages of threading APIs.

* `remoteattestation` sample shows how to make remote attestation with Rust SGX SDK. The sample is forked from [linux-sgx-attestation](https://github.com/svartkanin/linux-sgx-remoteattestation) and credits to Blackrabbit (blackrabbit256@gmail.com). The enclave in Rust is shipped in this sample and Makefiles are modified accordingly.

* `hugemem` sample shows how to use huge mem in SGX enclave. In this sample, we allocate reserve 31.75GB heap space and allocate 31.625GB buffers!

* `file` sample shows how to read/write files in SGX enclave.

* `hello-rust` is the helloworld sample writtin in pure Rust.

* `backtrace` is a sample showing how to enabling backtrace mechanism inside the enclave.

* `unit-test` shows the way of writing unit tests and conduct unit testing.

* `zlib-lazy-static-sample` shows how to use ported third party crates inside enclave.

* `machine-learning` shows how to use [rusty-machine](https://github.com/AtheMathmo/rusty-machine) for machine learning inside Intel SGX enclave.

* `tls` contains a pair of TLS client/server runs perfectly in SGX enclave!

* `sgxtime` shows how to acquire trusted timestamp via Intel ME. Please refer to this [instruction](documents/sgxtime.md) for detail.

* `protobuf` shows how to use the ported `rust-protobuf` to pass messages to the enclave using protobuf. Please install protobuf-compiler by `apt-get install protobuf-compiler` and protobuf-codegen by `cargo install protobuf-codegen --vers=2.8.1` before compiling this sample.

* `wasmi` shows how to pass WebAssembly test suites using the ported WebAssembly interpreter.

* `psi` is a prototype solution of the Private-Set-Intersection problem.

* `secretsharing` shows the usage of Shamir sharing in Rust-SGX environment (provided by @davidp94).

* `switchless` shows the usage of latest "switchless" execution model provided by intel. Please pay attention to the Makefile and the position of link flag "-lsgx_tswitchless".

* `mutual-ra` provides remote attestation based TLS connection between SGX enclaves. See the [readme](samplecode/mutual-ra/Readme.md) for details.

* `ue-ra` provides remote attestation based TLS connection between an untrusted party and one SGX enclave. See the [readme](samplecode/ue-ra/Readme.md) for details.

* `sgx-cov` shows how to use lcov with Rust SGX enclave to generate code coverage report. See the [readme](samplecode/sgx-cov/Readme.md) for details.

* `tcmalloc` shows how to link Rust-SGX enclave with tcmalloc (provided by Intel SGX SDK), and test its performance with different kinds of workload.

# Tips for writing enclaves in Rust

## Writing EDL

* For fixed-length array in ECALL/OCALL definition, declare it as an array.  For dynamic-length array, use the keyword `size=` to let the Intel SGX knows how many bytes should be copied.

## ECALL Function Naming

* Add `#[no_mangle]` for every ECALL function.

## Passing/returning arrays

* For dynamic-length array, the only way is to use raw pointers in Rust. There are several functions to get/set data using raw pointers such as [`offset`](https://doc.rust-lang.org/1.9.0/std/primitive.pointer.html#method.offset) method. One can also use [`slice::from_raw_parts`](https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html) to convert the array to a slice.

* For Fixed-length array, the above method is acceptable. And according to discussions in [issue 30382](https://github.com/rust-lang/rust/issues/30382) and [issue 31227](https://github.com/rust-lang/rust/issues/31227), thin-pointers (such as fixed-length array) are FFI-safe for now, but undocumented. In the sample codes, we use fixed-length arrays for passing and returning some fixed-length data.

# License

The Apache Teaclave Rust-SGX SDK is provided under the Apache license. Please refer to the [License file](LICENSE) for details.

# Authors

Ran Duan, Long Li, Chan Zhao, Shi Jia, Yu Ding, Yulong Zhang, Huibo Wang, Yueqiang Cheng, Lenx Wei, Tanghui Chen

# Acknowledgement

Thanks to [Prof. Jingqiang Lin](http://people.ucas.ac.cn/~0010268) for his contribution to this project.

# Contacts

Yu Ding, dingelish@gmail.com

