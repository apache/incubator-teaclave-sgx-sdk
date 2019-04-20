# Protected Code Launch Sample

This code sample shows how to use PCL in Rust-SGX.

`pcl-user` contains logic of the user side, who wants to load encrypted enclave on a remote SGX-enabled machine.

`encrypted-hello` is the code user wants to protect. It is encrypted during building process. The encryption key is dynamically generated.

`pcl-seal` is a remote SGX app. It contains a sealing enclave which is in charge of storing the encryption key and provide it to the PCL API.

## Build and Run

To build, just type `make` and everything should be fine.

To run, please put your IAS registration files (client.key, client.crt and spid.txt) in under `pcl_seal/bin/`. Then

```
$ cd pcl-seal/bin
$ ./app (add --unlink if your spid's type is unlinkable)
```

In another terminal, start the pcl-user app:

```
$ cd pcl-user
$ cargo run
```

Next you'll see `pcl-seal` starts getting a report from Intel and establishes a RA-based TLS channel with `pcl-user` and gets sealed key provisioned and stored in `SgxFile`. At last, `pcl-seal` will launch the `encrypted-hello` enclave and finally print the hello message.

```
Entering get_sealed_pcl_key
SgxFs read success: 68F6DEF27C33F248864A74D9607EA6B3
get_sealed_pcl_key 040002000000...(suppressed)
[+] Home dir is /root
[-] Open token file /root/payload.token error! Will create one.
[+] Init Enclave Successful 3!
This is a normal world string passed into Enclave!
This is a in-Enclave Rust string!
[+] Done!
```
