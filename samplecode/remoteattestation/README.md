# Rust SGX remote attestation
This example is derived from [linux-sgx-remoteattestation](https://github.com/svartkanin/linux-sgx-remoteattestation). Only the enclave is rewrote into Rust.
Please refer to the following Readme for further detail.

For users' convenience, we simply add `SGX_MODE=HW SGX_PRERELEASE=1` into Makefile and just run `make` in both `ServiceProvider` and `Application`.

btw. Dependencies include `libjsoncpp-dev` which is missing from below.

#Linux SGX remote attestation (Original Readme below)
Example of a remote attestation with Intel's SGX including the communication with IAS.

The code requires the installation of Intel SGX [here](https://github.com/01org/linux-sgx) and
the SGX driver [here](https://github.com/01org/linux-sgx-driver). Furthermore, also a developer account
for the usage of IAS has be registered [Deverloper account](https://software.intel.com/en-us/sgx).
After the registration with a certificate (can be self-signed for development purposes), Intel will
respond with a SPID which is needed to communicate with IAS.

The code consists of two separate programs, the ServiceProvider and the Application.
The message exchange over the network is performed using Google Protocol Buffers.

## Installation

Before running the code, some settings have to be set in the ```GeneralSettings.h``` file:
* The application port and IP
* A server certificate and private key are required for the SSL communication between the SP and the Application (which can be self-signed)<br />
e.g. ```openssl req -x509 -nodes -newkey rsa:4096 -keyout server.key -out sever.crt -days 365```
* The SPID provided by Intel when registering for the developer account
* The certificate sent to Intel when registering for the developer account
* IAS Rest API url (should stay the same)

To be able to run the above code some external libraries are needed:

* Google Protocol Buffers (should already be installed with the SGX SDK package) otherwise install ```libprotobuf-dev```, ```libprotobuf-c0-dev``` and ```protobuf-compiler```
* ```libboost-thread-dev```, ```libboost-system-dev```
* ```curl```, ```libcurl4-openssl-dev```
* ```libssl```
* ```liblog4cpp5-dev```

```
$ apt-get update && apt-get install -y libprotobuf-c0-dev protobuf-compiler libboost-thread-dev libboost-system-dev curl libcurl4-openssl-dev libjsoncpp-dev liblog4cpp5-dev libprotobuf-dev

# Recompile the protobuf
$ cd GoogleMessages/
$ protoc Messages.proto --cpp_out=./
```


After the installation of those dependencies, the code can be compiled with the following commands:<br/>
```cd ServiceProvider```<br />
```make```<br />
```cd ../Application```<br />
```make SGX_MODE=HW SGX_PRERELEASE=1```
