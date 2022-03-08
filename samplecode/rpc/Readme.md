# Tonic based gRPC client and server in enclave

Tonic and tokio is now available in enclave. Place this in Cargo.toml and you're good to go.

```toml
[dependencies]
prost = "0.9"
tokio = { version = "1.0", features = ["rt-multi-thread", "time", "fs", "macros", "net"] }
tonic = { version = "0.6.2", features = ["tls", "compression"]  }

[build-dependencies]
tonic-build = { version = "0.6.2", features = ["prost", "compression"] }
```

## Tokio runtime configuration

### tokio::main annotation sample

`tokio::main` by default creates a worker pool with `logical core count`
number of threads. As a result, TCS number has to be `logical core count` + 1.
The additional 1 TCS is reserved for the initializer thread.

A dual socket Xeon 8352S server platform (with SGX2 support) has 128 logical
cores. To run the server enclave natively, one need to set TCSnum to 129.

```Rust
//#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let greeter = MyGreeter::default();

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}

// Ecall function
#[no_mangle]
pub extern "C" fn run_server() -> SgxStatus {
    match main() {
        Ok(_) => SgxStatus::Success,
        Err(e) => {
            println!("Failed to run server: {}", e);
            SgxStatus::Unexpected
        }
    }
}
```

### tokio::runtime::Builder sample

Tokio's runtime builder allows us to configure the runtime in detail. The following sample shows how to set a worker pool to use 32 threads (resulted in 33 in TCSnum).

```Rust
#[no_mangle]
pub extern "C" fn run_server() -> SgxStatus {
    let result = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(32) // TCS = 32 + 1 = 33. 1 reserved for initializer
        .enable_all()
        .build()
        .map(|rt| rt.block_on(main()));

    match result {
        Ok(Ok(_)) => SgxStatus::Success,
        Ok(Err(e)) => {
            println!("Failed to run server: {}", e);
            SgxStatus::Unexpected
        },
        Err(e) => {
            println!("Failed to create tokio runtime in enclave: {}", e);
            SgxStatus::Unexpected
        }
    }
}
```

In this case, `main` function is just a regular `async` function without any annotation.
