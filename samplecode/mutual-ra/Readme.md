# Mutual Remote Attestation code sample

This code sample contains an implementation of [Integrating Remote Attestation with Transport Layer Security](https://github.com/cloud-security-research/sgx-ra-tls/blob/master/whitepaper.pdf).

## Requirements

To use this code sample, one needs to register at Intel website for dev IAS service access. Once the registration is finished, the following stuff should be ready:

1. An SPID assigned by Intel
2. IAS client certificate such as `client.crt`
3. IAS client private key such as `client.key`

To check whether your IAS registration is complete, please perform the following query with your IAS client certificate and private key:

```
curl -1 --tlsv1.2 -v --key client.key --cert client.crt https://test-as.sgx.trustedservices.intel.com:443/attestation/sgx/v3/sigrl/00000ABC
```

Here `00000ABC` is a fake group id which is only used here for testing connection. If this http request can successfully obtain an HTTP status code (no matter which code it is), your IAS service registration should be fine.

## Embedding

`enclave/src/lib.rs` contains three funcs `load_certs` `load_private_key` and `load_spid`. These three functions are configured to load cert/key/spid from `client.crt` `client.key` `spid.txt` from `bin` directory respectively. One can either adjust the file paths/names or copy the cert/key/spid to `bin`. `spid.txt` should only contain one line of 32 chars such as `DEADBEAFDEADBEAFDEADBEAFDEADBEAF`.

## Run

Start server

```
make
cd bin
./app --server (add --unlink if your spid's type is unlinkable)
```

Start client 

```
make
cd bin
./app --client (add --unlink if your spid's type is unlinkable)
```
