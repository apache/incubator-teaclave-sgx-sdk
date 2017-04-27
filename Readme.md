# Rust SGX SDK
This Rust SGX SDK helps developers write Intel SGX enclaves in Rust programming language. 

## Requirement
Ubuntu 16.04

[Intel SGX SDK 1.8 for Linux](https://01.org/zh/intel-softwareguard-extensions) installed

Docker (Recommended)

## Configuration

### Using docker (Recommended)
First, make sure Intel SGX Driver 1.8 is installed and functions well. `/dev/isgx` should be appeared.

Second, pull the docker image

`$ docker pull baiduxlab/sgx-rust`

Third, start a docker with sgx device support and the Rust SGX SDK.

`$ docker run -v /your/path/to/rust-sgx:/root/sgx -ti --device /dev/isgx baiduxlab/sgx-rust`

Next, start the aesm service inside the docker

`root@docker:/# /opt/intel/sgxpsw/aesm/aesm_service &`

Finally, check if the sample code works

`root@docker:~/sgx/samplecode/helloworld# make`

`root@docker:~/sgx/samplecode/helloworld# cd bin`

`root@docker:~/sgx/samplecode/helloworld/bin# ./app`

### Native without docker (Not recommended)

Install Intel SGX driver and SDK first. And refer to Dockerfile for detail. 

## Build the docker image by yourself

Make sure Intel SGX SDK is properly installed and service started on the host
OS. Then `cd dockerfile` and run `docker build -t rust-sgx-docker` to build.

# Documents

The online documents for SDK crates can be found
[here](https://dingelish.github.io/).

We recommend to use `cargo doc` to generate documents for each crate in this
SDK by yourself.  The auto generated documents are easy to read and search.

# Sample Codes

We provide five sample codes to help developers understand how to write Enclave
codes in Rust. These codes are located at `samplecode` directory.

* `helloworld` is a very simple app. It shows some basic usages of argument
passing, Rust string and ECALL/OCALLs.

* `crypto` shows the usage of crypto APIs provided by Intel SGX libraries. It
does some crypto calculations inside the enclave, which is recommended in most
circumstances.

* `localattestation` is a sample ported from the original Intel SGX SDK. It
shows how to do local attestation in Rust programming language.

* `sealeddata` sample shows how to seal secret data in an enclave and how to
verify the sealed data. 

* `thread` sample is a sample ported from the original Intel SGX SDK, showing
some basic usages of threading APIs. 

# Tips for writing enclaves in Rust

## Writing EDL

* For fixed-length array in ECALL/OCALL definition, declare it as an array.  For
dynamic-length array, use the keyword `size=` to let the Intel SGX knows how
many bytes should be copied.

## ECALL Function Naming

* Add `#[no_mangle]` for every ECALL function.

## Passing/returning arrays

* For dynamic-length array, the only way is to use raw pointers in Rust. There
are several functions to get/set data using raw pointers such as
[`offset`](https://doc.rust-lang.org/1.9.0/std/primitive.pointer.html#method.offset)
method. One can also use
[`slice::from_raw_parts`](https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html)
to convert the array to a slice.

* For Fixed-length array, the above method is acceptable. And according to
discussions in [issue 30382](https://github.com/rust-lang/rust/issues/30382)
and [issue 31227](https://github.com/rust-lang/rust/issues/31227),
thin-pointers (such as fixed-length array) are FFI-safe for now, but
undocumented. In the sample codes, we use fixed-length arrays for passing and
returning some fixed-length data.

# License

Baidu Rust-SGX SDK is provided under the BSD license. Please refer to the [License file](LICENSE)
for details.

# Authors

Ran Duan, Long Li, Shi Jia, Yu Ding, Lenx Wei, Tanghui Chen

# Contacts

Yu Ding, dingelish@gmail.com

