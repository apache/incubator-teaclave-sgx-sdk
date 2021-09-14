---
permalink: /sgx-sdk-docs/debugging-a-local-rust-sgx-enclave
---
# Before start

As of today (03/19/2019), `sgx-gdb` cannot work well with gdb > 7.12. In this wiki page, I'm using the gdb 7.11.1. Please check if you have the correct version of gdb.

# Requirement

* Linux
* docker installed
* Intel SGX driver installed and `/dev/isgx` works.
* docker image baiduxlab/sgx-rust:1604

# Prepare the code

Let's use [hello-rust](https://github.com/apache/incubator-teaclave-sgx-sdk/tree/master/samplecode/hello-rust) as debuggee. We need to add debug info for all of the Rust/C codes.

First, switch to debug build for the Rust codes. In the root Makefile of hello-rust, remove the `--release` Rust flag and fix the path:
```diff
-App_Rust_Flags := --release
+App_Rust_Flags :=
 App_SRC_Files := $(shell find app/ -type f -name '*.rs') $(shell find app/ -type f -name 'Cargo.toml')
 App_Include_Paths := -I ./app -I./include -I$(SGX_SDK)/include -I$(CUSTOM_EDL_PATH)
 App_C_Flags := $(SGX_COMMON_CFLAGS) -fPIC -Wno-attributes $(App_Include_Paths)

-App_Rust_Path := ./app/target/release
+App_Rust_Path := ./app/target/debug
 App_Enclave_u_Object :=app/libEnclave_u.a
 App_Name := bin/app
```
And do the same thing in enclave/Makefile:
```diff
-       RUSTC_BOOTSTRAP=1 cargo build --release
-       cp ./target/release/libhelloworldsampleenclave.a ../lib/libenclave.a
-endif
+       RUSTC_BOOTSTRAP=1 cargo build
+       cp ./target/debug/libhelloworldsampleenclave.a ../lib/libenclave.a
+endif
```

And we also need to add debug symbol to for Enclave_t.c and Enclave_u.c. In the root Makefile:
```diff
        SGX_ENCLAVE_SIGNER := $(SGX_SDK)/bin/x86/sgx_sign
        SGX_EDGER8R := $(SGX_SDK)/bin/x86/sgx_edger8r
 else
-       SGX_COMMON_CFLAGS := -m64
+       SGX_COMMON_CFLAGS := -m64 -ggdb
        SGX_LIBRARY_PATH := $(SGX_SDK)/lib64
        SGX_ENCLAVE_SIGNER := $(SGX_SDK)/bin/x64/sgx_sign
        SGX_EDGER8R := $(SGX_SDK)/bin/x64/sgx_edger8r
```

# Start the docker container

Next, use the following command to start a docker container. Please **fix the path** before running it.

```bash
$ docker run -ti \
             --rm \
             --privileged \
             -v /home/ding/rust-sgx-sdk:/root/rust-sgx-sdk \
             --device /dev/isgx \
             baiduxlab/sgx-rust:1604 bash
root@ef40bc98b273:~#
```

Then, set up gdb using `apt-get`:

```
root@ef40bc98b273:~# apt-get update && apt-get install -y gdb
```

Then check if the version is correct:

```
root@ef40bc98b273:~# gdb --version
GNU gdb (Ubuntu 7.11.1-0ubuntu1~16.5) 7.11.1
Copyright (C) 2016 Free Software Foundation, Inc.
License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.  Type "show copying"
and "show warranty" for details.
This GDB was configured as "x86_64-linux-gnu".
Type "show configuration" for configuration details.
For bug reporting instructions, please see:
<http://www.gnu.org/software/gdb/bugs/>.
Find the GDB manual and other documentation resources online at:
<http://www.gnu.org/software/gdb/documentation/>.
For help, type "help".
Type "apropos word" to search for commands related to "word".
```

As of today (03-19-2019), the installed version is 7.11.1 and it's ok.

Then we need to start the aesm service daemon:

```
root@ef40bc98b273:~# /opt/intel/libsgx-enclave-common/aesm/aesm_service
aesm_service[878]: The server sock is 0x55ed65a9a560
aesm_service[878]: [ADMIN]White List update requested
aesm_service[878]: [ADMIN]Platform Services initializing
aesm_service[878]: [ADMIN]Platform Services initialization failed due to DAL error
aesm_service[878]: [ADMIN]White list update request successful for Version: 49
```

Just ignore the `DAL error`.

# Debug the enclave

```
root@ef40bc98b273:~# cd rust-sgx-sdk/samplecode/hello-rust/
root@ef40bc98b273:~/rust-sgx-sdk/samplecode/hello-rust# make
info: syncing channel updates for 'stable-2019-01-17-x86_64-unknown-linux-gnu'
info: latest update on 2019-01-17, rust version 1.32.0 (9fda7c223 2019-01-16)
info: downloading component 'rustc'
..........(suppressed output)..........
LINK =>  enclave/enclave.so
<!-- Please refer to User's Guide for the explanation of each field -->
<EnclaveConfiguration>
    <ProdID>0</ProdID>
    <ISVSVN>0</ISVSVN>
    <StackMaxSize>0x40000</StackMaxSize>
    <HeapMaxSize>0x100000</HeapMaxSize>
    <TCSNum>1</TCSNum>
    <TCSPolicy>1</TCSPolicy>
    <DisableDebug>0</DisableDebug>
    <MiscSelect>0</MiscSelect>
    <MiscMask>0xFFFFFFFF</MiscMask>
</EnclaveConfiguration>
tcs_num 1, tcs_max_num 1, tcs_min_pool 1
The required memory is 1798144B.
Succeed.
SIGN =>  bin/enclave.signed.so
```

Let's debug it!

```
root@ef40bc98b273:~/rust-sgx-sdk/samplecode/hello-rust# cd bin/
root@ef40bc98b273:~/rust-sgx-sdk/samplecode/hello-rust/bin# sgx-gdb ./app
GNU gdb (Ubuntu 7.11.1-0ubuntu1~16.5) 7.11.1
Copyright (C) 2016 Free Software Foundation, Inc.
License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.  Type "show copying"
and "show warranty" for details.
This GDB was configured as "x86_64-linux-gnu".
Type "show configuration" for configuration details.
For bug reporting instructions, please see:
<http://www.gnu.org/software/gdb/bugs/>.
Find the GDB manual and other documentation resources online at:
<http://www.gnu.org/software/gdb/documentation/>.
For help, type "help".
Type "apropos word" to search for commands related to "word"...
Source directories searched: /opt/sgxsdk/lib64/gdb-sgx-plugin:$cdir:$cwd
Setting environment variable "LD_PRELOAD" to null value.
Reading symbols from ./app...done.
warning: Missing auto-load script at offset 0 in section .debug_gdb_scripts
of file /root/rust-sgx-sdk/samplecode/hello-rust/bin/app.
Use `info auto-load python-scripts [REGEXP]' to list them.
(gdb) b say_something
Breakpoint 1 at 0x11800: file app/Enclave_u.c, line 731.
(gdb) r
Starting program: /root/rust-sgx-sdk/samplecode/hello-rust/bin/app
detect urts is loaded, initializing
[Thread debugging using libthread_db enabled]
Using host libthread_db library "/lib/x86_64-linux-gnu/libthread_db.so.1".
[+] Home dir is /root
[-] Open token file /root/enclave.token error! Will create one.
add-symbol-file '/root/rust-sgx-sdk/samplecode/hello-rust/bin/enclave.signed.so' 0x7ffff5805340 -readnow -s .interp 0x7ffff5800270  -s .note.gnu.build-id 0x7ffff580028c  -s .gnu.hash 0x7ffff58002b0  -s .dynsym 0x7ffff58002e0  -s .dynstr 0x7ffff5800388  -s .gnu.version 0x7ffff58003c2  -s .gnu.version_d 0x7ffff58003d0  -s .rela.dyn 0x7ffff5800408  -s .plt 0x7ffff5805310  -s .plt.got 0x7ffff5805320  -s .nipx 0x7ffff5845060  -s .rodata 0x7ffff58458a0  -s .eh_frame_hdr 0x7ffff584ace0  -s .eh_frame 0x7ffff584d4b0  -s .gcc_except_table 0x7ffff5857850  -s .tbss 0x7ffff5a597a0  -s .init_array 0x7ffff5a597a0  -s .fini_array 0x7ffff5a597a8  -s .data.rel.ro 0x7ffff5a59800  -s .dynamic 0x7ffff5a5b000  -s .got 0x7ffff5a5b190  -s .got.plt 0x7ffff5a5c000  -s .data 0x7ffff5a5c020  -s .nipd 0x7ffff5a5cd84  -s .niprod 0x7ffff5a5cdc0  -s .bss 0x7ffff5a5d600
[+] Saved updated launch token!
[+] Init Enclave Successful 2!

Breakpoint 1, say_something (eid=2, retval=0x7fffffffe288,
    some_string=0x5555557c0f00 "This is a normal world string passed into Enclave!\n", len=51) at app/Enclave_u.c:731
731	{
(gdb)
```

Look at the automatically triggered `add-symbol-file` command. `sgx-gdb` helps us load the debug sym automatically. If you can't see this line, it means that `sgx-gdb` is not working.

Let's check where it stopped at:

```
(gdb) info reg rip
rip            0x555555565800	0x555555565800 <say_something>
```

It stopped at 0x555555565800, not in the enclave. It the place where `say_something` located in `Enclave_u.c`. But we can set another breakpoint at the one inside enclave:

```
(gdb) info line say_something
Line 731 of "app/Enclave_u.c" starts at address 0x555555565800 <say_something>
   and ends at 0x555555565804 <say_something+4>.
Line 51 of "src/lib.rs" starts at address 0x7ffff5827ab0 <say_something>
   and ends at 0x7ffff5827ac1 <say_something+17>.
(gdb) b "src/lib.rs:say_something"
Note: breakpoint 1 also set at pc 0x7ffff5827ac1.
Breakpoint 2 at 0x7ffff5827ac1: file src/lib.rs, line 52.
```

So now bp #2 is the correct bp inside SGX enclave. Continue:

```
(gdb) c
Continuing.

Breakpoint 1, say_something (
    some_string=0x7ffff5a71040 "This is a normal world string passed into Enclave!\n", some_len=51) at src/lib.rs:52
(gdb) info reg rip
rip            0x7ffff5827ac1	0x7ffff5827ac1 <say_something+17>
```

That's it! It stopped correctly at the first statement in enclave function `say_something`! We can do something more using gdb:

```
(gdb) n
54	    let str_slice = unsafe { slice::from_raw_parts(some_string, some_len) };
(gdb)
55	    let _ = io::stdout().write(str_slice);
(gdb)
This is a normal world string passed into Enclave!
58	    let rust_raw_string = "This is a in-Enclave ";
(gdb) n
60	    let word:[u8;4] = [82, 117, 115, 116];
(gdb) p rust_raw_string
$1 = {data_ptr = 0x7ffff5847150 "This is a in-Enclave Invalid UTF-8\n",
  length = 21}
```

That's it! Try `p` more stuffs!

## Tips

[peda](https://github.com/longld/peda) is helpful with some commands like `vmmap`.
