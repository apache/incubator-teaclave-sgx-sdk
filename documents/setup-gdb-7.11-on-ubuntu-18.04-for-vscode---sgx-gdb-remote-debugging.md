---
permalink: /sgx-sdk-docs/setup-gdb-ubuntu18
---
This instruction is provided by @akoskinas. Thanks!
This is an updated version of the process, which contains bug fixes and additional comments.

------------------------------
Before start: In our setup, the debugging works only when building in simulation mode, i.e
```
cd rust-sgx-sdk/samplecode/hello-rust-vscode-debug/
SGX_MODE=SW SGX_DEBUG=1 make
cd bin
sgx-gdb ./app
```
A short description of the process needed in order to use GDB to remotely debug a Rust SGX enclave with sgx gdb in Ubuntu 18.04:

1. Make sure that the package libsgx-enclave-common-dbgsym_${version}-${revision}_amd64.ddeb is installed, as described here: https://github.com/intel/linux-sgx#build-the-intelr-sgx-psw-installer . The  package can also be found here: https://download.01.org/intel-sgx/linux-2.5/ubuntu18.04-server/

2. Make sure to set up the needed environment variables before compiling your code. To do so, run:
```
  $ source ${sgx-sdk-install-path}/environment  
```
3. As documented [here](debugging-a-local-rust-sgx-enclave-in-docker-with-sgx-gdb.md) , an older version of GDB debugger has to be utilized for debugging.  The steps to use gdb-7.11.1 are

- get the source code of version 7.11.1 :
```
wget "http://ftp.gnu.org/gnu/gdb/gdb-7.11.1.tar.gz"
```

- extract
```
tar -xvzf gdb-7.11.1.tar.gz
```
- install the python development headers, needed to configure the GDB python interpreter:
```
sudo apt-get install python3-dev
```

- configure the build: because python scripts will be given as input to the GDB, a python interpreter has to be configured at this step. To do so the option "with-python" shall be used, followed by the path to the desired python version
```
cd gdb-7.11.1
./configure --with-python=/usr/bin/python3
```

- build: in order for building to complete the following changes are required to solve a type conflict - building takes ~3mins :

In file: gdb/amd64-linux-nat.c:248 --> delete word "const"
in file: gdb/gdbserver/linux-x86-low.c:239:1 --> delete word "const"
```
make
```

- Two options are available to complete installation:

If GDB 7.11.1 is desired to be located in /usr/bin/gdb then execute:

```
sudo make install
```
If GDB 7.11.1 is desired to be located in a different location, two symbolic links are needed. In our case,  we chose to place gdb-7.11.1 folder under /opt directory. In that case the respective commands will look like the following:
```
 sudo ln -sf /opt/gdb-7.11.1/gdb/gdb /usr/bin/gdb
 cd /usr/local/share
 # if gdb dir doesn't exist, create it : mkdir -p gdb
 cd gdb
 # if python dir doesn't exist, create it: mkdir -p python
 cd python
 sudo ln -s /opt/gdb-7.11.1/gdb/data-directory/python/gdb/ /usr/local/share/gdb/python/
```
4. Up until this point, local debugging should be successful. The final step is to use VS Code, by following the steps described [here](use-vscode---rls---rust-analysis---sgx-gdb-for-graphic-developing-(not-in-docker).
