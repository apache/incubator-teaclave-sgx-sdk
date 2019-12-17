# Note

Please visit our [homepage](https://github.com/apache/teaclave-sgx-sdk) for usage. Thanks!

# sgx_tcrypto_helper

This crate intended to be the corresponding crate of sgx_crypto_helper works in SGX enclave. It shares the source codes with sgx_crypto_helper but with different dependencies. To use this crate, you can add the following lines to your Cargo.toml:

```
sgx_crypto_helper = { package="sgx_tcrypto_helper", git = "https://github.com/apache/teaclave-sgx-sdk" }
```

And then

```
extern crate sgx_crypto_helper; // same to sgx_crypto_helper!
```
