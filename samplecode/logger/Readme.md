# env_logger sample

This sample shows how to use [env_logger](https://github.com/sebasmagri/env_logger) in an Rust-SGX enclave. We maintain a SGX fork of [env_logger] at:

https://github.com/mesalock-linux/env_logger-sgx

It is keep updated with its upstream's Github repo. It depends on a SGX fork of log at:

https://github.com/mesalock-linux/log-sgx

## Usage of Sample code

```
$ make
$ cd bin
$ RUST_LOG=trace ./app
```

```
$ make
$ cd bin
$ RUST_LOG=info ./app
```

## Usage

* To use env_logger, one must be sure the `TCSPolicy` is `0`.

* To use env_logger, one must include `sgx_env.edl` in the enclave's EDL file.

* In Cargo.toml, bring in log and env_logger:

```toml
log = { git = "https://github.com/mesalock-linux/log-sgx" }
env_logger = { git = "https://github.com/mesalock-linux/env_logger-sgx" }
```

* Import log and env_logger as usual:

```rust
#[macro_use] extern crate log
extern crate env_logger;
```

* Initialize and log as usual

```rust
env_logger::init();

info!("starting up");
```

* See the log output

```
$ make
$ cd bin
$ RUST_LOG=trace ./app
```
