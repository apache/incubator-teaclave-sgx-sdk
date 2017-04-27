# Rust SGX SDK v0.1.0

## Introduction
Baidu X-Lab provides Rust SGX SDK, to help develop Intel SGX enclaves in Rust
programming language.

### Contents

* Basic Rust SGX SDK libraries
* A patch to solve Rust's floating point compatibility issue.
* Sample enclave codes.
* A well-configured [docker image](https://hub.docker.com/r/baiduxlab/sgx-rust/).

Please refer to our white paper for details.

## What's New

* Support for the latest Intel SGX SDK v1.8 for Linux.
* Support for the latest Rust nightly build (1.18.0-nightly (c58c928e6 2017-04-11)).
* Rust style document.

## System Requirements

* Ubuntu 16.04 LTS
* [Hardware platform supports Intel SGX](https://github.com/ayeks/SGX-hardware)
* Docker (Strongly recommended)
* Rust nightly (Tested on rustc 1.18.0-nightly (c58c928e6 2017-04-11))

## Known Issues and Limitations

* Rust stable branch is unsupported.
* `cargo test` is unsupported.

## Unstable APIs

* `rsgx_get_thread_data`

* `rsgx_get_enclave_base`

* `rsgx_get_heap_base`

* `rsgx_get_heap_size`
