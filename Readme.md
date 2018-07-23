# Rust SGX SDK
Rust SGX SDK helps developers write Intel SGX applications in Rust programming language. [[Paper pdf]](documents/ccsp17.pdf)

To achieve better security, we recommend developers to apply [Non-bypassable Security Paradigm (NbSP)](https://github.com/baidu/rust-sgx-sdk/blob/master/documents/nbsp.pdf) to the system design and implementation.

## v1.0.1 Release
This version supports the Rust nightly build (nightly-2018-07-16) in master branch and the most recent Rust stable build (stable-2018-07-10). And it supports the latest Intel SGX SDK **v2.2**. New third party libraries include: bytes, http, iovec, rust-crypto, rust-fnv and rust-threshold-secret-sharing. New code sample 'secretsharing' and 'rust-threshold-secret-sharing' is provided by @davidp94. Please refer to [release_notes](release_notes.md) for further details.

## v1.0.0 Release
We proudly announce v1.0.0 of rust-sgx-sdk! We port Parity's [Webassembly Interpreter](https://github.com/paritytech/wasmi) to Intel SGX and provide a full functional in-enclave [wasmi sample](samplecode/wasmi), and a [sample solution](samplecode/psi) of two-party private-set-intersection resisting side-channel attacks! From this version, we start to support most recent stable branch of Rust instead of nightly for better stability and future production use. Thus, the [stable branch](https://github.com/baidu/rust-sgx-sdk/tree/rust-stable) of v1.0.0 supports the most recent Rust stable toolchain (1.26.0 stable-2018-05-07), while the master only supports Rust nightly toolchain of nightly-2018-04-11. Please refer to [release_notes](release_notes.md) for further details.

## v0.9.8 Release
This version provides security updates regards to recent Spectre attacks in Intel SGX, and supports **Rust stable (2018-03-01)** (in branch named 'rust-stable'). It contains support of [Intel SGX SDK 2.1.2](https://download.01.org/intel-sgx/linux-2.1.2/) and a series of API functions to stop speculative execution on demand. In addition, we provide a ported version of [rust-protobuf](https://crates.io/crates/protobuf) v1.4.4. Please refer to [release_notes](release_notes.md) for further details.

## v0.9.7 Release
This version provides a new namespace: `sgx_tstd::untrusted`, including `sgx_tstd::untrusted::fs` `sgx_tstd::untrusted::time` and `sgx_tstd::untrusted::path`, providing supports to operation to ocalls in a **untrusted** namespace. The **untrusted** namespace is always enabled no matter `untrusted_*` is set or not. We **urge** the developers to use the `sgx_tstd::untrusted` namespace to port their crates, instead of using the `untrusted_` series of features. Also, we renamed the `untrusted_net` feature to `net` for feature name unification. Please refer to [release_notes](release_notes.md) for further details.

## Run Rust SGX applications in Mesalock Linux
[MesaLock Linux](https://github.com/mesalock-linux/mesalock-distro) is a general purpose Linux distribution which aims to provide a safe and secure user space environment. Now we can run Rust SGX applications in Mesalock Linux within a few steps. Please refer to the [tutorial](documents/sgx_in_mesalock_linux.md) for details.

## Requirement

Ubuntu 16.04

[Intel SGX SDK 2.2 for Linux](https://01.org/intel-software-guard-extensions/downloads) installed

Docker (Recommended)


## Configuration

The docker image now supports Intel ME. If you need it, please refer to the sgxtime [readme](documents/sgxtime.md) for instructions.

### Native without docker (Not recommended)

Install Intel SGX driver and SDK first. And refer to [Dockerfile](https://github.com/baidu/rust-sgx-sdk/blob/master/dockerfile/Dockerfile) or stable branch [Dockerfile](https://github.com/baidu/rust-sgx-sdk/blob/master/dockerfile/rust-stable/Dockerfile) to setup your own native Rust-SGX environment.

### Using docker (Recommended) without ME support

First, make sure Intel SGX Driver 2.2 is installed and functions well. `/dev/isgx` should be appeared.

Second, pull the docker image. If you'd like to work on stable branch of Rust and `rust-stable` branch of this SDK, please pull `baiduxlab/sgx-rust-stable` instead.

`$ docker pull baiduxlab/sgx-rust`

Third, start a docker with sgx device support and the Rust SGX SDK.

`$ docker run -v /your/path/to/rust-sgx:/root/sgx -ti --device /dev/isgx baiduxlab/sgx-rust`

Next, start the aesm service inside the docker

`root@docker:/# /opt/intel/sgxpsw/aesm/aesm_service &`

Finally, check if the sample code works

`root@docker:~/sgx/samplecode/helloworld# make`

`root@docker:~/sgx/samplecode/helloworld# cd bin`

`root@docker:~/sgx/samplecode/helloworld/bin# ./app`

## Build the docker image by yourself

Make sure Intel SGX SDK is properly installed and service started on the host OS. Then `cd dockerfile` and run `docker build -t rust-sgx-docker` to build.

## Use simulation mode for non SGX-enabled machine (includes macOS)

Intel provides a simulation mode so you can develop on regular machines, by building the enclave app using the libraries `sgx_urts_sim`, `lsgx_uae_service_sim`, `sgx_trts_sim`, `sgx_tservice_sim`.

First, pull the docker image. If you'd like to work on stable branch of Rust and `rust-stable` branch of this SDK, please pull `baiduxlab/sgx-rust-stable` instead.

`$ docker pull baiduxlab/sgx-rust`

Second, start a docker with the Rust SGX SDK.

`$ docker run -v /your/path/to/rust-sgx:/root/sgx -ti baiduxlab/sgx-rust`

But when building any sample code, set the `SGX_MODE` to `SW` in `Makefile`.

`root@docker:~/sgx/samplecode/helloworld# vi Makefile`

Replace `SGX_MODE ?= HW` with `SGX_MODE ?= SW`

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

* `protobuf` shows how to use the ported `rust-protobuf` to pass messages to the enclave using protobuf. Please install protobuf-compiler by `apt-get install protobuf-compiler` and protobuf-codegen by `cargo install protobuf-codegen --vers=2.0.3` before compiling this sample.

* `wasmi` shows how to pass WebAssembly test suites using the ported WebAssembly interpreter.

* `psi` is a prototype solution of the Private-Set-Intersection problem.

* `secretsharing` shows the usage of Shamir sharing in Rust-SGX environment (provided by @davidp94).

# Samples of ported third-party libraries

As of v0.9.5, we provide 25 ported third-party libraries. All of them could be compiled using xargo (`XARGO_SGX=1` make) or cargo (`make`).

# Tips for writing enclaves in Rust

## Writing EDL

* For fixed-length array in ECALL/OCALL definition, declare it as an array.  For dynamic-length array, use the keyword `size=` to let the Intel SGX knows how many bytes should be copied.

## ECALL Function Naming

* Add `#[no_mangle]` for every ECALL function.

## Passing/returning arrays

* For dynamic-length array, the only way is to use raw pointers in Rust. There are several functions to get/set data using raw pointers such as [`offset`](https://doc.rust-lang.org/1.9.0/std/primitive.pointer.html#method.offset) method. One can also use [`slice::from_raw_parts`](https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html) to convert the array to a slice.

* For Fixed-length array, the above method is acceptable. And according to discussions in [issue 30382](https://github.com/rust-lang/rust/issues/30382) and [issue 31227](https://github.com/rust-lang/rust/issues/31227), thin-pointers (such as fixed-length array) are FFI-safe for now, but undocumented. In the sample codes, we use fixed-length arrays for passing and returning some fixed-length data.

# License

Baidu Rust-SGX SDK is provided under the BSD license. Please refer to the [License file](LICENSE) for details.

# Authors

Ran Duan, Long Li, Shi Jia, Yu Ding, Lenx Wei, Tanghui Chen

![Baidu X-Lab Logo](https://raw.githubusercontent.com/baidu/rust-sgx-sdk/master/logo_25.png)

# Acknowledgement

Thanks to [Prof. Jingqiang Lin](http://people.ucas.ac.cn/~0010268) for his contribution to this project.

# Contacts

Yu Ding, dingelish@gmail.com

