---
permalink: /sgx-sdk-docs/sgxtime
---
# Acquiring Trusted timestamp from Intel ME in SGX enclave

Intel provides Trusted Platform Service via Intel Management Engine. Part of the Intel SGX features depend on the trusted platform service, including the trusted timestamp service. We implemented `sgx_tservice::SgxTime` for this feature. To use it in Linux, the prerequisites are:
1. `/dev/mei0` must exist.
2. [Dynamic Application Loader (DAL) Host Interface (aka JHI)](https://github.com/intel/dynamic-application-loader-host-interface) installed.
3. [iclsClient](https://software.intel.com/en-us/sgx-sdk/download) installed.

The Ubuntu linux kernel would initiate Intel ME device during boot. Almost every Intel platform has Intel ME, but it may not be initiated properly. For [example](https://github.com/ayeks/SGX-hardware/issues/24), a server of SuperServer 5019S-MR with v2.0b BIOS + v4.0.3.96 SPS could be initiated properly, while the other server with the same mother board and same BIOS version but v4.1.3.22 SPS couldn't.

The JHI service usually runs as a daemon service process. Its [readme](https://github.com/intel/dynamic-application-loader-host-interface/blob/master/readme.md) is short and easy to read.

The iclsClient could be downloaded from the [Intel SGX's website](https://software.intel.com/en-us/sgx-sdk/download). Tozd's [docker-sgx](https://github.com/tozd/docker-sgx) provides a sample Dockerfile including the setup of iclsClient.

One important thing is that the Intel SGX PSW has to be installed **after** the above three prerequisites has been satisfied. During the first time of PSW installation, the installer would try to do the provisioning for this platform, and this takes about one minute.

We provide [sgxtime](../samplecode/sgxtime) code sample for demonstrating how to acquire trusted timestamp from Intel ME.

## Run in docker

The dockerfile we provide contains an optional setup of icls. Due to the limitation of iclsClient, you need to acquire the installer from Intel and follow the instruction in [Dockerfile](../docker/Dockerfile).

After downloaded the icls installer, please uncomment the icls related lines in Dockerfile and build the docker image by yourself.

To run sgxtime in this docker image, first launch it using the following command:

```bash
$ docker run -ti -v /path/to/sdk:/root/sgx \
             --device /dev/isgx \
             --device /dev/mei0 \
             rust-sgx-docker    # This name is identified during docker build
root@913e6a00c8d8:~#
```

Then start the `jhid` and `aesm_service`

```
root@913e6a00c8d8:~# jhid -d
root@913e6a00c8d8:~# jhi[16]: --> jhi start
jhi[16]: <-- jhi start

root@913e6a00c8d8:~# /opt/intel/sgxpsw/aesm/aesm_service
root@913e6a00c8d8:~# aesm_service[18]: [ADMIN]White List update requested
aesm_service[18]: The server sock is 0x55d3d2893940
jhi[16]: JHI service release prints are enabled

jhi[16]: Applet repository dir path: /var/lib/intel/dal/applet_repository
jhi[16]: Applets dir path: /var/lib/intel/dal/applets
aesm_service[18]: [ADMIN]White list update request successful for Version: 25

root@913e6a00c8d8:~#
```

Then build the code sample

```
root@913e6a00c8d8:~# cd sgx/samplecode/sgxtime/
root@913e6a00c8d8:~/sgx/samplecode/sgxtime# XARGO_SGX=1 make
make -C ./enclave/
make[1]: Entering directory '/root/sgx/samplecode/sgxtime/enclave'
cargo build --release
    Updating registry `https://github.com/rust-lang/crates.io-index`
.........
</EnclaveConfiguration>
tcs_num 1, tcs_max_num 1, tcs_min_pool 1
The required memory is 2437120B.
Succeed.
SIGN =>  bin/enclave.signed.so
```

Then run it. The first time would probably fail, but it only fails once.

```
root@913e6a00c8d8:~/sgx/samplecode/sgxtime/bin# ./app
[+] Home dir is /root
[-] Open token file /root/enclave.token error! Will create one.
[+] Saved updated launch token!
[+] Init Enclave Successful 2!
aesm_service[18]: [ADMIN]Platform Services initializing
aesm_service[18]: [ADMIN]EPID Provisioning initiated
aesm_service[18]: [ADMIN]EPID Provisioning successful
aesm_service[18]: PCH EPID RL retrieval failure
Cannot create PSE session
Err with SGX_ERROR_AE_SESSION_INVALID
close PSE session done
Hello world
[+] sgx_time_sample success...
root@913e6a00c8d8:~/sgx/samplecode/sgxtime/bin# ./app
[+] Home dir is /root
[+] Open token file success!
[+] Token file valid!
[+] Init Enclave Successful 2!
aesm_service[18]: [ADMIN]Platform Services initializing
aesm_service[18]: [ADMIN]Platform Services initialized successfully
Create PSE session done
Ok with SgxTime { timestamp: 1420259903, source_nonce: [17, 101, 46, 174, 115, 133, 196, 251, 170, 218, 3, 21, 81, 92, 144, 241, 66, 38, 230, 186, 251, 193, 41, 246, 148, 131, 111, 126, 191, 105, 17, 33] }
close PSE session done
Hello world
[+] sgx_time_sample success...
root@913e6a00c8d8:~/sgx/samplecode/sgxtime/bin# ./app
[+] Home dir is /root
[+] Open token file success!
[+] Token file valid!
[+] Init Enclave Successful 2!
Create PSE session done
Ok with SgxTime { timestamp: 1420259905, source_nonce: [17, 101, 46, 174, 115, 133, 196, 251, 170, 218, 3, 21, 81, 92, 144, 241, 66, 38, 230, 186, 251, 193, 41, 246, 148, 131, 111, 126, 191, 105, 17, 33] }
close PSE session done
Hello world
[+] sgx_time_sample success...
```

## Run without docker

Follow the instruction of [JHI](https://github.com/intel/dynamic-application-loader-host-interface) first. Be sure to run `systemctl enable jhi` to enable the service and then start it.

Install iclsClient following Intel's setup [instruction](https://download.01.org/intel-sgx/linux-2.0/docs/Intel_SGX_Installation_Guide_Linux_2.0_Open_Source.pdf). `sudo ldconfig` may probably be needed after the installation.

Next, uninstall the current PSW (if installed) and reinstall it.

Now, `sgxtime` should work.
