# Teaclave SGX SDK

[![License](https://img.shields.io/badge/license-Apache-green.svg)](LICENSE)
[![Homepage](https://img.shields.io/badge/site-homepage-blue)](https://teaclave.apache.org/)

Apache Teaclave (incubating) SGX SDK helps developers to write Intel SGX
applications in the Rust programming language, and also known as Rust SGX SDK.

## Getting Started

The SDK container can either be built from source or pulled from docker hub and
run as a docker container.

### Configuration

The docker image now supports Intel ME. If you need it, please refer to the
sgxtime [readme](documents/sgxtime.md) for instructions.

### Prerequisites

* #### Docker
* #### Intel SGX OOT 2.11.0 Driver or DCAP 1.36.2 Driver
* #### Intel SGX SDK v2.12
* #### Intel SGX PSW
* #### Rust nightly-2020-10-25

You can find the [installation guides](https://download.01.org/intel-sgx/sgx-linux/2.9/docs/) for Intel SGX software on the 01.org website.

**Note**: if you are running our SDK on a machine without SGX support, you will still need the simulation versions of the Intel PSW and SDK.

### Pulling a Pre-Built Docker Container

It is assumed that the user has [correctly installed docker](https://docs.docker.com/get-docker/). We provide 5 containers:

* `baiduxlab/sgx-rust:1604-1.1.3`
* `baiduxlab/sgx-rust:1804-1.1.3`
* `baiduxlab/sgx-rust:2004-1.1.3`
* `baiduxlab/sgx-rust:fedora31-1.1.3`
* `baiduxlab/sgx-rust:centos8-1.1.3`

With `latest` pinned to `1604-1.1.3`.

First, pull the docker container of your choosing, this command will download `latest`:
```
$ docker pull baiduxlab/sgx-rust
```
To run the container, we recommend that you download our samplecode

#### Running with Intel SGX Drivers:

We recommend starting by using the github repository as a first volume to run the container on:
```
$ git clone https://github.com/apache/incubator-teaclave-sgx-sdk.git
```
To run the container with SGX support, run:
```
$ docker run -v /path/to/rust-sgx:/root/sgx --device /dev/libsgx -ti baiduxlab/sgx-rust
```
If instead, you are using DCAP drivers you must run:
```
$ docker run -v /path/to/rust-sgx:/root/sgx -ti --device /dev/sgx/enclave --device /dev/sgx/provision baiduxlab/sgx-rust
```
Whilst inside the container, we must start the AESM daemon and define `LD_LIBRARY_PATH`:
```
# LD_LIBRARY_PATH=/opt/intel/sgx-aesm-service/aesm/

# /opt/intel/sgx-aesm-service/aesm/aesm_service
```
If everything has been properly configured, it is now possible to run a quick `helloworld` program inside the container:
```
# cd sgx/samplecode/helloworld

# make

# cd bin

# ./app
```
We recommend you look at other files in the `samplecode` folder to familiarize yourself with programming in our SDK.

#### Running without Intel SGX Drivers:

**Note**: Intel provides a simulation mode so you can develop on regular machines, by building the enclave app using the libraries sgx_urts_sim, lsgx_uae_service_sim, sgx_trts_sim, sgx_tservice_sim.

We recommend starting by using the github repository as a first volume to run the container on:
```
$ git clone https://github.com/apache/incubator-teaclave-sgx-sdk.git
```
To run the container without SGX support, run:
```
$ docker run -v /path/to/rust-sgx:/root/sgx -ti baiduxlab/sgx-rust
```
Once inside the container, when running any of the `samplecode` you may either:
- Modify the Makefile and set `SGX_MODE` to `SW`
- Run `export SGX_MODE=SW`
- Run `make` with the build flag `SGX_MODE=SW`

We may now run our `helloworld` example:

```
# cd sgx/samplecode/helloworld

# make SGX_MODE=SW

# cd bin

# ./app
```
### Building a Docker Image

Make sure Intel SGX SDK is properly installed and service started on the host OS. Then `cd dockerfile` and run `docker build -t rust-sgx-docker -f Dockerfile.1604.nightly .` to build.

## Code Samples

We provide eighteen code samples to help developers understand how to write Enclave code in Rust. These samples are located in the `samplecode` directory.

<details>
<summary>See Code Samples</summary>

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
</details>

## Tips for writing enclaves in Rust

<details>
<summary>See tips</summary>

## Writing EDL

* For fixed-length array in ECALL/OCALL definition, declare it as an array.  For dynamic-length array, use the keyword `size=` to let the Intel SGX knows how many bytes should be copied.

## ECALL Function Naming

* Add `#[no_mangle]` for every ECALL function.

## Passing/returning arrays

* For dynamic-length array, the only way is to use raw pointers in Rust. There are several functions to get/set data using raw pointers such as [`offset`](https://doc.rust-lang.org/1.9.0/std/primitive.pointer.html#method.offset) method. One can also use [`slice::from_raw_parts`](https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html) to convert the array to a slice.

* For Fixed-length array, the above method is acceptable. And according to discussions in [issue 30382](https://github.com/rust-lang/rust/issues/30382) and [issue 31227](https://github.com/rust-lang/rust/issues/31227), thin-pointers (such as fixed-length array) are FFI-safe for now, but undocumented. In the sample codes, we use fixed-length arrays for passing and returning some fixed-length data.
</details>

## Pre-Apache Releases

### Latest: v1.1.3

Supports Intel SGX SDK v2.12, and Rust nightly-2020-10-25. Added support to Ubuntu 20.04. **We strongly recommend users to upgrade to Intel SGX SDK v2.12 and drivers to DCAP 1.36.2 and OOT 2.11.0.** [release_notes](release_notes.md).

<details>
<summary>Version 1.1.2</summary>

### v1.1.2

Supports Intel SGX SDK v2.9.1, and Rust nightly-2020-04-07. v1.1.2 provides a handy crate `sgx_signal`, which enables signal capture. One can easily find the place where exception happens and finally triggered `ud2`. And we added `Backtrace::capture` in sgx_tstd. With the help of Intel SGX SDk v2.9.1's patch, dtor of thread local storage finally works on regular SGX thread and pthread thread. Removed sgx_core_futures since Rust is supporting `async`/`await` in `no_std` environment. Please refer to [release_notes](release_notes.md) for more details.
</details>

<details>
<summary>Version 1.1.1</summary>

### v1.1.1

Supports Intel SGX SDK v2.9, and Rust nightly-2020-03-12. v1.1.1 contains a bunch of bug fix and new proc macro `sgx_align` to help with aligning given structure. For LVI migigation, it only works on C/C++ parts (EDL headers/Intel's libs) and supports both two modes: `MITIGATION-CVE-2020-0551=LOAD` or `MITIGATION-CVE-2020-0551=CF`. To enable it, one need `env "MITIGATION-CVE-2020-0551=LOAD"` to set this environment variable. For detailed information, please refer to [release_notes](release_notes.md) for more details.
</details>

<details>
<summary>Version 1.1.0</summary>

### v1.1.0

Supports Intel SGX SDK v2.7.1, and Rust nightly-2019-11-25. v1.1.0 brings up dynamic static supports by `thread::spawn`, and almost everything of `std::sync`. Also v1.1.0 benefits from Intel SGX SDK's aligned memory allocation primitives to mitigate [INTEL-SA-00219](https://github.com/apache/incubator-mesatee-sgx/wiki/Mitigation-of-Intel-SA-00219-in-Rust-SGX). Besides, we enabled [`is_x86_feature_detected!`](https://github.com/apache/incubator-mesatee-sgx/wiki/%60is_x86_feature_detected%60-in-Rust-SGX-SDK) by parsing a hidden global CPU feature indicator initialized by Intel SGX urts/trts. And we provided Dockerfile for Fedora 27. For detailed information, please refer to [release_notes](release_notes.md) for more details.
</details>

<details>
<summary>Version 1.0.9</summary>

### v1.0.9 Release

Supports Intel SGX SDK v2.6, and Rust nightly-2019-08-01. Bumps everything to edition. Removed third_party directory since we have all of those dependencies forked and maintained with merge bot. Since Intel SGX SDK v2.6 imports some breaking changes in global thread metata, thread local features of v1.0.9 is not works on Intel SGX SDK v2.5. EDL and common headers are changed respectively. For detailed information, please refer to [release_notes](release_notes.md) for more details.
</details>

<details>
<summary>Version 1.0.8</summary>

### v1.0.8 Release

Supports the most recent Rust nightly (nightly-2019-05-22) and Rust stable (stable-2019-05-14). Code coverage support has been added to sgx_cov. Bug fixes in memory allocator and panicking routines. New third party libraries to support kvdb-memorydb. Please refer to [release_notes](release_notes.md) for more details.
</details>

<details>
<summary>Version 1.0.7</summary>

### v1.0.7 Release

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
</details>

<details>
<summary>Version 1.0.6</summary>

### v1.0.6 Release
Fix bugs in sgx_alloc, sgx_types, ucd-generate and improve sgx_tunittest. Added rust-base58. Thanks to @elichai, @cbeck88, @brenzi and @nhynes.
</details>

<details>
<summary>Version 1.0.5</summary>

### v1.0.5 Release
This version supports Rust nightly build (nightly-2019-01-28, v1.34.0) in the master branch and the most recent stable build (stable-2019-01-16, v1.32.0) in the rust-stable branch. It supports the latest Intel SGX SDK **v2.4.0** and Ubuntu Linux 16.04+18.04. We provide support to Intel's Protected Code Loader. We provide sgx_ucrypto and sgx_crypto_helper for using SGX-style crypto primitives in untrusted app and RSA keypair serialization/deserialization in both trusted and untrusted programs. We re-organize ocall related interfaces and provide them in a new crate sgx_libc with a bunch of new ocall functions. In addition, we port net2 to SGX. Please refer to [release_notes](release_notes.md) for further details.
</details>
<details>
<summary>Version 1.0.4</summary>

### v1.0.4 Release
This version supports Rust nightly build (nightly-2018-10-01) in the master branch and the most recent stable build (stable-2018-09-25) in the rust-stable branch. It supports the latest Intel SGX SDK **v2.3.1** and Ubuntu Linux 18.04. It now contains further third party libraries including: bit-vec, chrono, erased-serde, fxhash, nan-preserving-float, num-bigint, quick-error, raft-rs, time, webpki-roots, and yasna. Some third party libraries, like untrusted, parity-wasm and lazy-static, are removed because they support `no_std` and can be used directly from crates.io. We strongly recommend developers upgrade to v1.0.4 and use the most recent Rust release to build it due to the [Security advisory for the standard library](https://blog.rust-lang.org/2018/09/21/Security-advisory-for-std.html). Please refer to [release_notes](release_notes.md) for further details.
</details>

<details>
<summary>Version 1.0.1</summary>

### v1.0.1 Release
This version supports the Rust nightly build (nightly-2018-07-16) in master branch and the most recent Rust stable build (stable-2018-07-10). And it supports the latest Intel SGX SDK **v2.2**. New third party libraries include: bytes, http, iovec, rust-crypto, rust-fnv and rust-threshold-secret-sharing. New code sample 'secretsharing' and 'rust-threshold-secret-sharing' is provided by @davidp94. Please refer to [release_notes](release_notes.md) for further details.
</details>

<details>
<summary>Version 1.0.0</summary>

### v1.0.0 Release
We proudly announce v1.0.0 of rust-sgx-sdk! We port Parity's [Webassembly Interpreter](https://github.com/paritytech/wasmi) to Intel SGX and provide a full functional in-enclave [wasmi sample](samplecode/wasmi), and a [sample solution](samplecode/psi) of two-party private-set-intersection resisting side-channel attacks! From this version, we start to support most recent stable branch of Rust instead of nightly for better stability and future production use. Thus, the [stable branch](https://github.com/apache/teaclave-sgx-sdk/tree/rust-stable) of v1.0.0 supports the most recent Rust stable toolchain (1.26.0 stable-2018-05-07), while the master only supports Rust nightly toolchain of nightly-2018-04-11. Please refer to [release_notes](release_notes.md) for further details.
</details>

<details>
<summary>Version 0.9.8</summary>

### v0.9.8 Release
This version provides security updates regards to recent Spectre attacks in Intel SGX, and supports **Rust stable (2018-03-01)** (in branch named 'rust-stable'). It contains support of [Intel SGX SDK 2.1.2](https://download.01.org/intel-sgx/linux-2.1.2/) and a series of API functions to stop speculative execution on demand. In addition, we provide a ported version of [rust-protobuf](https://crates.io/crates/protobuf) v1.4.4. Please refer to [release_notes](release_notes.md) for further details.
</details>

<details>
<summary>Version 0.9.7</summary>

### v0.9.7 Release
This version provides a new namespace: `sgx_tstd::untrusted`, including `sgx_tstd::untrusted::fs` `sgx_tstd::untrusted::time` and `sgx_tstd::untrusted::path`, providing supports to operation to ocalls in a **untrusted** namespace. The **untrusted** namespace is always enabled no matter `untrusted_*` is set or not. We **urge** the developers to use the `sgx_tstd::untrusted` namespace to port their crates, instead of using the `untrusted_` series of features. Also, we renamed the `untrusted_net` feature to `net` for feature name unification. Please refer to [release_notes](release_notes.md) for further details.
</details>

## Contributing

Teaclave is open source in [The Apache Way](https://www.apache.org/theapacheway/),
we aim to create a project that is maintained and owned by the community. All
kinds of contributions are welcome. Read this [document](CONTRIBUTING.md) to
learn more about how to contribute. Thanks to our
[contributors](https://teaclave.apache.org/contributors/).

## Community

- Join us on our [mailing list](https://lists.apache.org/list.html?dev@teaclave.apache.org).
- Follow us at [@ApacheTeaclave](https://twitter.com/ApacheTeaclave).
- See [more](https://teaclave.apache.org/community/).
