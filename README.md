# Teaclave SGX SDK v2.0 preview

Teaclave SGX SDK v2.0 is a revamped version. It is much more developer friendly
compared to v1.1.

- supports `cargo build` + `no_std`, `xargo build` and `cargo-std-aware` mode.
- Tokio and Tonic is directly available to enclave programming without any change.
- Refactored Intel's SDK using Rust. Only a small portion of Intel's SDK is required.
- Improved testing framework. `sgx_tstd` is well tested now.
- No need to maintain 100+ 3rd party dependencies. Most dependencies are `use`-able without any change.

## Build system

We still maintain the legacy `no_std` cargo build support, and `xargo build` with a customized sysroot. v2.0 supports `cargo-std-aware` as well.

To switch from these build modes, please specify
- `BUILD_STD=no` to use traditional `no_std` cargo build
- `BUILD_STD=cargo` (default setting) to use the new std aware cargo build
- `BUILD_STD=xargo` to use xargo build


## Samples

We are still working on porting all v1.1 samples to v2.0. Current available samples include:

- backtrace
- cov
- hellworld
- httpreq
- rpc (Tonic + Tokio)
- seal
- switchless
