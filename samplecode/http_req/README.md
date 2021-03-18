# http_req-sgx-example

Showcases [http_req-sgx](https://github.com/mesalock-linux/http_req-sgx) inside an SGX enclave.

## Instruction

Follow instructions in [apache/teaclave-sgx-sdk](https://github.com/apache/teaclave-sgx-sdk) to setup development environment. Alternatively, use [rust-sdk-helper](https://github.com/piotr-roslaniec/rust-sdk-helper).

Please make sure that you have environment variable `SGX_SDK_RUST` points to the root of this sdk.

Then execute `run.bash` to compile and run the project.
