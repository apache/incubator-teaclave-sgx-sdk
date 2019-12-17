# Rust SGX remote attestation
This example is derived from [linux-sgx-remoteattestation](https://github.com/svartkanin/linux-sgx-remoteattestation). Only the enclave is rewrote into Rust.
Please refer to the following Readme for further detail.

For users' convenience, we simply add `SGX_MODE=HW SGX_PRERELEASE=1` into Makefile and just run `make` in both `ServiceProvider` and `Application`.

btw. Dependencies include `libjsoncpp-dev` which is missing from below.

# Certificate configuration in GeneralSettings.h

There are **two** set of crt/keys in GeneralSettings.h.

* `server_crt` and `server_key` is for the connection between Service Provider and Application. This is **not** used in IAS connection.

* `static const char *ias_crt = "";` This is the path of the client cert generated for IAS registration. During the IAS registration, one always generates two files: `client.crt` and `client.key`. Use the following commands to combine them into one PEM file and place it here:

```
$ openssl pkcs12 -export -in ./client.crt -inkey client.key > client.p12
Enter Export Password:
Verifying - Enter Export Password:

$ openssl pkcs12 -in client.p12 -out client.pem -clcerts
Enter Import Password:
MAC verified OK
Enter PEM pass phrase:
Verifying - Enter PEM pass phrase:
```

Then the settings should be:

```
static const char *ias_crt = "client.pem"
```

During Remote attestation, the PEM pass phrase would be required.

As the most simple setup, one can just use IAS client crt/key for `server_crt` and `server_key`. It works.

# Signature policy definition in ServiceProvider.cpp

Please check your [signature policy](https://software.intel.com/en-us/articles/signature-policy) and set it up [here](https://github.com/apache/teaclave-sgx-sdk/blob/3ac5a21c3720bd819c938d28df11cbae499f3bc5/samplecode/remoteattestation/ServiceProvider/service_provider/ServiceProvider.cpp#L222) and [here](https://github.com/apache/teaclave-sgx-sdk/blob/c1bf3775e4abbd79a26450f91655d3f67f9e0083/samplecode/remoteattestation/ServiceProvider/service_provider/ServiceProvider.cpp#L291). Wrong signature policy would trigger IAS HTTP error code 400 in MSG3.

# Linux SGX remote attestation (Original Readme below)
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


After the installation of those dependencies, the code can be compiled with the following commands:<br/>
```cd ServiceProvider```<br />
```make```<br />
```cd ../Application```<br />
```make SGX_MODE=HW SGX_PRERELEASE=1```
