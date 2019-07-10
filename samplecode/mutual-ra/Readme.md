# Mutual Remote Attestation code sample

This code sample contains an implementation of [Integrating Remote Attestation with Transport Layer Security](https://github.com/cloud-security-research/sgx-ra-tls/blob/master/whitepaper.pdf).

## Requirements

To use this code sample, one needs to register at [Intel website](https://api.portal.trustedservices.intel.com/EPID-attestation) for dev IAS service access. Once the registration is finished, the following stuff should be ready:

1. An SPID assigned by Intel
2. IAS API Key assigned by Intel

Both of these information could be found in the new [Intel Trusted Services API Management Portal](https://api.portal.trustedservices.intel.com/developer). Please log into this portal and switch to "Manage subscriptions" page on the top right corner to see your SPID and API keys. Either primary key or secondary key works.

Save them to `bin/spid.txt` and `bin/key.txt` respectively. Size of these two files should be 32 or 33.

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
