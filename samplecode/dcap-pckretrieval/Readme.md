# DCAP PCK RetrievalTool

**This is a demo of using Teaclave Rust SGX with Intel SGX DCAP suite. More examples are coming up.**

Re-write most of Intel's [PCKRetrieval](https://github.com/intel/SGXDataCenterAttestationPrimitives/tree/master/tools/PCKRetrievalTool) tool in Rust:

- `app` behaves like [`App`](https://github.com/intel/SGXDataCenterAttestationPrimitives/tree/master/tools/PCKRetrievalTool/App)
- `enclave` is like [`Enclave`](https://github.com/intel/SGXDataCenterAttestationPrimitives/tree/master/tools/PCKRetrievalTool/Enclave)
- `qpl` is like [`Qpl`](https://github.com/intel/SGXDataCenterAttestationPrimitives/tree/master/tools/PCKRetrievalTool/Qpl)

`enclave` is configured to be a release mode enclave, and only supports DCAP on FLC enabled platform.

# Usage

`libsgx_dcap_ql.so` is required for building the app. With the default setup of Intel DCAP package, only `libsg_dcap_ql.so.1` presented at `/usr/lib/x86_64-linux-gnu`. You may probably need to create a symlink for it by

```
cd /usr/lib/x86_64-linux-gnu
ln -s libsgx_dcap_ql.so.1 libsgx_dcap_ql.so
```

Then the project could be build smoothly:

```
$ make
$ cd bin
$ ./PCKIDRetrievalTool
```

# Development tips

## Hardware

AFAIK, i7-9700k, i9-9900k, i9-9900ks, Celeron J5005 supports FLC. My platform is i9-9900ks + Gigabyte AORUS Z390 Master. DCAP suite v1.6 works fine. Also Xeon E-2100/E-2200 works.

## Software

Regular Intel SGX SDK + DCAP driver + DCAP libraries are enough. I use the following Dockerfile:

```
FROM ubuntu:18.04
MAINTAINER Yu Ding

ENV DEBIAN_FRONTEND=noninteractive
ENV rust_toolchain  nightly-2020-04-07
ENV sdk_bin         https://download.01.org/intel-sgx/sgx-linux/2.9.1/distro/ubuntu18.04-server/sgx_linux_x64_sdk_2.9.101.2.bin

RUN apt-get update && \
    apt-get install -y gnupg2 apt-transport-https ca-certificates curl software-properties-common build-essential automake autoconf libtool protobuf-compiler libprotobuf-dev git-core libprotobuf-c0-dev cmake pkg-config expect gdb

RUN curl -fsSL https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | apt-key add - && \
    add-apt-repository "deb [arch=amd64] https://download.01.org/intel-sgx/sgx_repo/ubuntu bionic main" && \
    apt-get update  && \
    apt-get install -y  libsgx-urts libsgx-dcap-ql libsgx-dcap-default-qpl sgx-dcap-pccs \
        libsgx-enclave-common-dbgsym libsgx-dcap-ql-dbgsym libsgx-dcap-default-qpl-dbgsym && \
    rm -rf /var/lib/apt/lists/* && \
    rm -rf /var/cache/apt/archives/* && \
    mkdir /var/run/aesmd && \
    mkdir /etc/init

RUN curl 'https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init' --output /root/rustup-init && \
    chmod +x /root/rustup-init && \
    echo '1' | /root/rustup-init --default-toolchain ${rust_toolchain} && \
    echo 'source /root/.cargo/env' >> /root/.bashrc && \
    /root/.cargo/bin/rustup component add rust-src rls rust-analysis clippy rustfmt && \
    /root/.cargo/bin/cargo install xargo && \
    rm /root/rustup-init && rm -rf /root/.cargo/registry && rm -rf /root/.cargo/git

RUN mkdir /root/sgx && \
    curl --output /root/sgx/sdk.bin ${sdk_bin} && \
    cd /root/sgx && \
    chmod +x /root/sgx/sdk.bin && \
    echo -e 'no\n/opt' | /root/sgx/sdk.bin && \
    echo 'source /opt/sgxsdk/environment' >> /root/.bashrc && \
    echo 'alias start-aesm="LD_LIBRARY_PATH=/opt/intel/sgx-aesm-service/aesm /opt/intel/sgx-aesm-service/aesm/aesm_service"' >> /root/.bashrc && \
    rm -rf /root/sgx*

RUN cd /usr/lib/x86_64-linux-gnu && \
    ln -s libsgx_dcap_ql.so.1 libsgx_dcap_ql.so

WORKDIR /root
```
