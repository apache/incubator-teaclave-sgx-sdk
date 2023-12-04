# Microsoft Azure Attestation Sample

This sample demonstrate how to use Microsoft Azure Attestation to do a ECDSA remote attestation of a DCAP quote. To simplify the implementation, both quote generation and report verification are included into a single executable file.

In your real-world applications, the `generate_quote` step shall be in your server backend, which generate a quote of the SGX enclave and contains a piece of custom runtime data (e.g. pubkey for encrypted communications). The sample code inject a static byte array into the enclave but your application shall generate the "runtime data" during the runtime. For example, you can generate a RSA key pair and put the pubkey into the quote and the private key is protected by the enclave.

The `validate_json_web_token` step shall be in your client-side code to verify the JWT from your server as a remote attestation report. If the JWT is valid, you may use the claims to verify the measurements and use the claim of runtime data for your specific application. [JWT](https://jwt.io/libraries) is supported by almost every mainstream programming languages. The JWT-based RA report can be easily integrated into web applications or Web3 smart contracts without relying on Intel SGX SDK or any other dependencies on the client-side.

Since the EPID remote attestation will not be available for the 3rd or newer generations of Xeon Scalable Processors, the other remote attestation [sample code](../remoteattestation) cannot be used for newer Intel Xeon Scalable Processors (Icelake or newer). The ECDSA attestation is the only solution with Intel SGX for scalable confidential cloud computing (supporting up to 1TB EPC; [source](https://www.intel.com/content/www/us/en/newsroom/news/xeon-scalable-platform-built-sensitive-workloads.html#gs.15v18u)).

# Initial Setup for ECDSA Attestation

Before you run this sample code, you need to install the [SGX DCAP driver](https://github.com/intel/SGXDataCenterAttestationPrimitives), which is built-in with latest Linux kernel.

You need to install SGX SDK and all dependent linux packages with this [guide](https://download.01.org/intel-sgx/latest/linux-latest/docs/Intel_SGX_SW_Installation_Guide_for_Linux.pdf).

You don't need to setup your own cache server since Azure provides this server internally. Your `PCCS_URL` in your `sgx_default_qcnl.conf` should point to Azure internal cache service "https://global.acccache.azure.net/sgx/certification/v4/".

Please use this [link](https://www.intel.com/content/www/us/en/developer/articles/guide/intel-software-guard-extensions-data-center-attestation-primitives-quick-install-guide.html) as a reference for initial configuration.

The Microsoft Azure SGX Attestation only works with Microsoft Azure Confidential Computing VM instances.

# Build and Run

Like other sample codes,

```
$ make
$ cd bin
$ ./maa
```

If the remote attestation is successful, you should be able to see the print of all validated JWT clams.
