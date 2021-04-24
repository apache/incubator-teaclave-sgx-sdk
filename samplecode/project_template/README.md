# Rust SGX - Template project
==================================

### This is a template project to start developing with the Teaenclave Rust SGX SDK (https://github.com/apache/incubator-teaclave-sgx-sdk/) easily.

You will find in its template:
- Makefiles to build your project easily, and link the ```SGX EDL C``` generated files to your Rust SGX projects
- The file ```buildenv.mk``` that contains compilation rules when building enclave. No need to specify anymore where this file is located.
- The file ```build.rs``` already configured to build the app/host part properly.
- The file rust-toolchain, so we can force the use of one specific toolchain (```nightly-2020-10-25``` in this case)
- ```Cargo/Xargo.toml``` files to set up your project easily. All the dependencies you might need has been added.

You can find those files in this template: 

```
|-- app/
|   |-- src/
|       |-- main.rs
|   |-- Cargo.toml
|   |-- Makefile
|   |-- build.rs
|   +-- rust-toolchain
|-- enclave/
|   |-- src/
|       |-- lib.rs
|   |-- Cargo.toml
|   |-- Enclave.config.xml
|   |-- Enclave.edl
|   |-- Enclave.lds
|   |-- Makefile
|   |-- Xargo.toml
|   +-- rust-toolchain
|-- Makefile
+-- buildenv.mk
```

## Setting up your project

You need to follow a few steps to use this template properly:
- Add your ```.rs``` files to the ```src/``` folders (```lib.rs``` / your enclave source code goes in ```enclave/src```, your host/app source code goes in ```app/src```), or modify the ```.rs``` files already included with the project
- Add your own ```Enclave.edl``` file, or modify the one joined in the project.
- Change the ```Cargo.toml (or/and Xargo.toml if you want to use Xargo)``` files depending of your needs (adding/removing dependences). 
    - Be careful if you want to change the library name on the ```Cargo.toml``` file (enclave part), you will need to reflect this change on the enclave ```Makefile```, more specifically on the ```ENCLAVE_CARGO_LIB``` variable, and on the ```lib.rs``` file.
    - If you need to change the app/host name, please make sure to edit the host ```Makefile```, and change the variable ```APP_U```.

## Build your project

### Before starting the building process, please make sure you downloaded the Rust SGX SDK repository, we're going to need the EDL and headers files joined in the SDK.

Once you downloaded the Rust SGX SDK, you have multiple ways to start the building process: 
- Run this command: ```CUSTOM_EDL_PATH=~/teaenclave/edl CUSTOM_COMMON_PATH=~/teaenclave/common make``` (replace ```~/teaenclave``` by the actual SDK location)
- You can also run the command export (```export CUSTOM_EDL_PATH=~/teaenclave/edl```), and specify the variables before calling make. It is adviced to add this command on your ```.bashrc``` file (if you use bash), or your favorite shell configuration file.

### By default, your project will be compiled in hardware mode. If you wish to compile your project in software/simulation mode, you will need to specify it, either by adding ```SGX_MODE=SW``` before make, or by setting the SGX_MODE variable environment to SW.

### Cargo is used by default when compiling, but you can also use Xargo either by adding ```XARGO_SGX=1``` before make, or by setting the XARGO_SGX variable environment to 1. You will also need to specify Xargo library path with XARGO_PATH.

### The makefile has those commands available: 
- make (will compile everything)
- make host (will only compile the host part)
- make enclave (will only compile the enclave part)
- make clean (will clean the objects/C edl files generated)
- make clean_host (will clean the objects/C edl files generated for the host only)
- make clean_enclave (will clean the objects/C edl files generated for the enclave only)
- make fclean (will clean objects/C edl files and the binairies, plus calling cargo clean for everything)
- make fclean_host (will clean objects/C edl files and the binairies, plus calling cargo clean for the host only)
- make fclean_enclave (will clean objects/C edl files and the binairies, plus calling cargo clean for the enclave only)
- make re (re as relink, will clean everything then compile everything again)
- make re_host (re as relink, will clean the host part then compile it again)
- make re_enclave (re as relink, will clean the enclave part then compile it again)