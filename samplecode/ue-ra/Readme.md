# Untrusted-Enclave Remote Attestation code sample

This code sample contains an implementation of [Integrating Remote Attestation with Transport Layer Security](https://github.com/cloud-security-research/sgx-ra-tls/blob/master/whitepaper.pdf), with the modification of the untrusted side.

## Requirements

To use this code sample, one needs to register at Intel website for dev IAS service access. Once the registration is finished, the following stuff should be ready:

1. An SPID assigned by Intel
2. IAS client certificate such as `client.crt`
3. IAS client private key such as `client.key`

To check whether your IAS registration is complete, please perform the following query with your client certificate and private key:

```
curl -1 --tlsv1.2 -v --key client.key --cert client.crt https://test-as.sgx.trustedservices.intel.com:443/attestation/sgx/v3/sigrl/00000ABC
```

Here `00000ABC` is a fake group id which is only used here for testing connection. If this http request can successfully obtain a HTTP status code (no matter which code it is), the IAS service registration should be fine.

## Custom CA/client setup

To establish a TLS channel, we need a CA and generates a client cert for mutual authentication. We store them at `cert`.

1. Generate CA private key
openssl ecparam -genkey -name prime256v1 -out ca.key

2. Generate CA cert
openssl req -x509 -new -SHA256 -nodes -key ca.key -days 3650 -out ca.crt

3. Generate Client private key
openssl ecparam -genkey -name prime256v1 -out client.key

4. Export the keys to pkcs8 unencrypted format
openssl pkcs8 -topk8 -nocrypt -in client.key -out client.pkcs8

5. Generate Client CSR
openssl req -new -SHA256 -key client.key -nodes -out client.csr

6. Generate Client Cert
openssl x509 -req -extfile <(printf "subjectAltName=DNS:localhost,DNS:www.example.com") -days 3650 -in client.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out client.crt

7. Intel CA report signing pem. Download and uncompress:
https://software.intel.com/sites/default/files/managed/7b/de/RK_PUB.zip

## Embedding IAS credentials to ue-ra-server

`enclave/src/lib.rs` contains three funcs `load_certs` `load_private_key` and `load_spid`. These three functions are configured to load cert/key/spid from `client.crt` `client.key` `spid.txt` from `bin` directory respectively. One can either adjust the file paths/names or copy the cert/key/spid to `bin`. `spid.txt` should only contain one line of 32 chars such as `DEADBEAFDEADBEAFDEADBEAFDEADBEAF`.

## Run

Start server

```
cd ue-ra-server
make
cd bin
./app (add --unlink if your spid's type is unlinkable)
```

Start client 

```
cd ue-ra-client
cargo run
```
