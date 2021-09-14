---
permalink: /sgx-sdk-docs/environment-setup
---
# Recommended OS to start with

We recommend Ubuntu 16.04/18.04. Desktop or server is the same. It could be your host OS or guest OS (inside docker). Technically, a full compatible list could be found at Intel's download [page](https://download.01.org/intel-sgx/linux-2.4/). As of 04-01-2019 (v 2.4.0), the list contains:

* CentOS 7.5
* Fedora 27 server
* RedHat Enterprise Linux 7.4
* SUSE 12.3 server
* Ubuntu 16.04
* Ubuntu 18.04

# Hardware setup

A good reference for hardware compatibility is [SGX-Hardware](https://github.com/ayeks/SGX-hardware). You can use the script [test-sgx.c](https://github.com/ayeks/SGX-hardware/blob/master/test-sgx.c) there to check if SGX is/could be enabled.

Followings are FAQs I've been always asked:
1. Macbook Pro? No to all on hardware support! Docker-based simulation is OK.
2. Rack Server? Here are my listings:
* SuperServer [5019S-MR](https://www.supermicro.com/products/system/1U/5019/SYS-5019S-MR.cfm)
* Lenovo [SR250](https://www.lenovo.com/us/en/data-center/servers/racks/ThinkSystem-SR250/p/77XX7SRSR25)
* SuperMicro Microblade server with [MBI-6119G-T4](https://www.supermicro.com/products/MicroBlade/module/MBI-6119G-T4.cfm).
3. Laptops? I don't know. If you can find something related to Intel SGX in the BIOS, then Yes.

Another solution is Intel's [VCA 2](https://www.intel.com/content/dam/support/us/en/documents/server-products/server-accessories/VCA2_HW_User_Guide.pdf) card. It should be placed only in 2-socket Xeon E5 systems (or following). Dmitrii of Intel Lab is using it for [Redis-SGX](https://redislabs.com/rlsessions_post_type/redis-sgx-secure-redis-intel-sgx/).

# Software Setup

## Rust toolchain

Please use [rustup](https://rustup.rs/) to install and manage Rust toolchains. **DO NOT** use anything like `apt` or `yum`.

During the installation you'll be asked about 'installation options' as follows:
```
Current installation options:

   default host triple: x86_64-unknown-linux-gnu
     default toolchain: stable
  modify PATH variable: yes
```

The host triple (though quadruple here) is correct. You could just press enter to skip it. When asking about default toolchain, you could enter `nightly-2019-01-28` or similar version number. And we recommend to answer 'Y' to the PATH modification.

rustup is always installed in `~` and does not affect other users.

Then you will have rustup works well. To switch to another toolchain, try
```
$ rustup toolchain default nightly-2019-03-31
```
This would triggers downloading and installation if the desired toolchain is not found on your disk.

To add more rust tools such as `rust-src` (for xargo), `rust-clippy` (for lint):
```
$ rustup component add rust-src
```

## Intel SGX toolchain setup

The toolchain setup strictly follows the following steps:

0. Driver installation ( sgx_linux_x64_driver_??????.bin ). You'll get a misc device '/dev/isgx' after this step.
1. (OPTIONAL, if Intel ME is required) iCls setup (iclsClient-1.45.449.12-1.x86_64.rpm)
2. (OPTIONAL, if Intel ME is required) jhi setup https://github.com/01org/dynamic-application-loader-host-interface
3. Platform Software installation (libsgx-enclave-common, libsgx-enclave-common-dev, libsgx-enclave-common-dbgsym)
4. Intel SGX SDK installation ( sgx_linux_x64_sdk_???????.bin )

And don't forget to source the `environment` file for Intel SGX SDK (such as sgx-sign).

## Docker setup

### Use docker with hardware support, and run aesm inside docker

Firstly, do step 0 to get `/dev/isgx` works. Then start a docker container as follows:

```
$ docker run -ti --rm -v /path/to/sdk:/root/sgx \
             --device /dev/isgx \
             --device /dev/mei0 \  # Optional if you have it and want to use it
             baiduxlab/sgx-rust
root@913e6a00c8d8:~#
```

(Optional) Install iCls and jhi daemon. Steps are [here](https://github.com/apache/incubator-teaclave-sgx-sdk/blob/master/dockerfile/Dockerfile.1604.nightly#L50)

(Optional) Start jhi daemon: `jhid -d`

Start aesm daemon
```
root@913e6a00c8d8:~# aesm_service[18]: The server sock is 0x5636e90be960
aesm_service[18]: [ADMIN]White List update requested
aesm_service[18]: [ADMIN]Platform Services initializing
aesm_service[18]: [ADMIN]Platform Services initialization failed due to DAL error
aesm_service[18]: [ADMIN]White list update request successful for Version: 49

root@913e6a00c8d8:~#
```

And then change directory to `/root/sgx/samplecode/hello-rust` and `make`. Then cd to `bin` and `./app`.

## Use docker without hardware support, only with simulation. Windows/Macbook compatible.

Make sure you have docker installed and working.

Start docker as:

```
$ docker run -ti --rm -v /path/to/sdk:/root/sgx baiduxlab/sgx-rust
root@913e6a00c8d8:~#
```

And then build in simulation mode
```
$ cd /root/sgx/samplecode/hello-rust
$ SGX_MODE=SW make
$ cd bin
$ ./app
```

### Use docker with hardware support, and run aesm outside docker (on the host OS)

![overview](https://github.com/apache/incubator-teaclave-sgx-sdk/raw/master/documents/mesa.png)

Just add another device mapping to the command to have `aesm.socket` works in SGX. This requires step 3 finished on the host OS and `/var/run/aesmd/aesm.socket` exists on the host OS.

```
$ docker run --rm -ti \
             --device /dev/isgx \                               # forward isgx device
             -v /path/to/rust-sgx-sdk:/root/sgx \               # add SDK
             -v /var/run/aesmd:/var/run/aesmd \                 # forward domain socket
             baiduxlab/sgx-rust
```

Then you can skip launching `aesmd` in the docker container.

# CI setup

The only known solution:[drone.io](http://drone.io) is provided by @elichai. We've set it up successfully.
