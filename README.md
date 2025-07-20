# Teaclave SGX SDK 

[![License](https://img.shields.io/badge/license-Apache-green.svg)](LICENSE)
[![Homepage](https://img.shields.io/badge/site-homepage-blue)](https://teaclave.apache.org/)

**Apache Teaclave SGX SDK** is a Rust SDK for developing Intel SGX applications. It enables developers to write secure, privacy-preserving applications using Intel Software Guard Extensions (SGX) technology with the safety and performance benefits of the Rust programming language.

## Overview

The Apache Teaclave SGX SDK provides a comprehensive development environment for building Intel SGX enclaves in Rust. The current version (v2.0) offers significant improvements over the [legacy v1.1](https://github.com/apache/incubator-teaclave-sgx-sdk/tree/master) , including:

- **Modern Build System**: Supports `cargo build` with `no_std`, `xargo build`, and `cargo-std-aware` modes
- **Rich Ecosystem**: Direct support for Tokio and Tonic in enclave programming without modifications
- **Lightweight Architecture**: Refactored Intel's SGX SDK using Rust, requiring only a minimal portion of Intel's original SDK
- **Robust Testing**: Comprehensive testing framework with well-tested `sgx_tstd` standard library
- **Simplified Dependencies**: Eliminates the need to maintain 100+ third-party dependencies; most Rust crates work without modifications

## Build System

The SDK supports multiple build modes to accommodate different development preferences:

- **`BUILD_STD=cargo`** (default): Uses the new std-aware cargo build system
- **`BUILD_STD=no`**: Traditional `no_std` cargo build for minimal footprint
- **`BUILD_STD=xargo`**: Uses xargo build with customized sysroot

## Sample Applications

The following sample applications demonstrate various SGX SDK capabilities:

- **backtrace**: Stack trace functionality in SGX enclaves
- **cov**: Code coverage analysis tools
- **crypto**: Cryptographic operations within enclaves
- **helloworld**: Basic SGX enclave example
- **httpreq**: HTTP client functionality
- **hyper-rustls-https-server**: HTTPS server using Hyper and Rustls
- **logger**: Logging capabilities for SGX applications
- **regex**: Regular expression processing
- **rpc**: Remote procedure calls using Tonic and Tokio
- **seal**: Data sealing and unsealing operations
- **switchless**: Switchless call optimization
- **zlib-lazy-static-sample**: Compression with lazy static initialization

*Note: Migration of additional v1.1 samples to v2.0 is ongoing.*

## Getting Started

For detailed installation instructions, development guides, and API documentation, please visit:

- **Project Website**: [https://teaclave.apache.org/](https://teaclave.apache.org/)
- **Documentation**: [Teaclave SGX SDK Documentation](https://teaclave.apache.org/sgx-sdk-docs/)
- **API Reference**: [Teaclave SGX SDK API Reference](https://teaclave.apache.org/api-docs/sgx-sdk/)

## Contributing

Teaclave is developed in the open following [The Apache Way](https://www.apache.org/theapacheway/). We strive to maintain a project that is community-driven and inclusive.

We welcome all forms of contributions. Please refer to our [Contributing Guide](https://teaclave.apache.org/contributing) for more information. A big thank-you to all our [contributors](https://teaclave.apache.org/contributors/)!

## Community

- 📬 Join our [mailing list](https://lists.apache.org/list.html?dev@teaclave.apache.org)
- 🐦 Follow us on [Twitter @ApacheTeaclave](https://twitter.com/ApacheTeaclave)
- 🌐 Learn more at [teaclave.apache.org/community](https://teaclave.apache.org/community/)