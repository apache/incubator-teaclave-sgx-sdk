# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.

######## SGX SDK Settings ########

SGX_SDK ?= /opt/intel/sgxsdk
SGX_MODE ?= HW
SGX_ARCH ?= x64

TOP_DIR := ..
include $(TOP_DIR)/buildenv.mk

ifeq ($(shell getconf LONG_BIT), 32)
	SGX_ARCH := x86
else ifeq ($(findstring -m32, $(CXXFLAGS)), -m32)
	SGX_ARCH := x86
endif

ifeq ($(SGX_ARCH), x86)
	SGX_COMMON_CFLAGS := -m32
	SGX_LIBRARY_PATH := $(SGX_SDK)/lib
	SGX_BIN_PATH := $(SGX_SDK)/bin/x86
else
	SGX_COMMON_CFLAGS := -m64
	SGX_LIBRARY_PATH := $(SGX_SDK)/lib64
	SGX_BIN_PATH := $(SGX_SDK)/bin/x64
endif

ifeq ($(SGX_DEBUG), 1)
	SGX_COMMON_CFLAGS += -O0 -g
else
	SGX_COMMON_CFLAGS += -O2
endif

SGX_EDGER8R := $(SGX_BIN_PATH)/sgx_edger8r
ifneq ($(SGX_MODE), HYPER)
	SGX_ENCLAVE_SIGNER := $(SGX_BIN_PATH)/sgx_sign
else
	SGX_ENCLAVE_SIGNER := $(SGX_BIN_PATH)/sgx_sign_hyper
	SGX_EDGER8R_MODE := --sgx-mode $(SGX_MODE)
endif

######## CUSTOM Settings ########

CUSTOM_LIBRARY_PATH := ./lib
CUSTOM_BIN_PATH := ./bin
CUSTOM_EDL_PATH := ../sgx_edl/edl
CUSTOM_COMMON_PATH := ../common

######## EDL Settings ########

Enclave_EDL_Files := enclave/enclave_t.c enclave/enclave_t.h app/enclave_u.c app/enclave_u.h

######## APP Settings ########

App_Rust_Flags := --release
App_SRC_Files := $(shell find app/ -type f -name '*.rs') $(shell find app/ -type f -name 'Cargo.toml')
App_Include_Paths := -I ./app -I$(SGX_SDK)/include -I$(CUSTOM_EDL_PATH)
App_C_Flags := $(CFLAGS) $(SGX_COMMON_CFLAGS) -fPIC -Wno-attributes $(App_Include_Paths)

App_Rust_Path := ./app/target/release
App_Enclave_u_Object := $(CUSTOM_LIBRARY_PATH)/libenclave_u.a
App_Name := $(CUSTOM_BIN_PATH)/test

######## Enclave Settings ########

RustEnclave_Build_Flags := --release
RustEnclave_SRC_Files := $(shell find enclave/ -type f -name '*.rs') $(shell find enclave/ -type f -name 'Cargo.toml')
RustEnclave_Include_Paths := -I$(CUSTOM_COMMON_PATH)/inc -I$(CUSTOM_COMMON_PATH)/inc/tlibc -I$(CUSTOM_EDL_PATH)

RustEnclave_Link_Libs := -L$(CUSTOM_LIBRARY_PATH) -lenclave
RustEnclave_C_Flags := $(CFLAGS) $(ENCLAVE_CFLAGS) $(SGX_COMMON_CFLAGS) $(RustEnclave_Include_Paths)
RustEnclave_Link_Flags := -Wl,--no-undefined -nostdlib -nodefaultlibs -nostartfiles \
	-Wl,--whole-archive $(RustEnclave_Link_Libs) -Wl,--no-whole-archive \
	-Wl,--version-script=enclave/enclave.lds \
	$(ENCLAVE_LDFLAGS)

RustEnclave_Out_Path := ./enclave/target/release
RustEnclave_Lib_Name := $(RustEnclave_Out_Path)/libunittest.a
RustEnclave_Name := $(CUSTOM_BIN_PATH)/enclave.so
RustEnclave_Signed_Name := $(CUSTOM_BIN_PATH)/enclave.signed.so

.PHONY: all
all: $(Enclave_EDL_Files) $(App_Name) $(RustEnclave_Signed_Name)

######## EDL Objects ########

$(Enclave_EDL_Files): $(SGX_EDGER8R) enclave/enclave.edl
	$(SGX_EDGER8R) $(SGX_EDGER8R_MODE) --trusted enclave/enclave.edl --search-path $(CUSTOM_COMMON_PATH)/inc --search-path $(CUSTOM_EDL_PATH) --trusted-dir enclave
	$(SGX_EDGER8R) $(SGX_EDGER8R_MODE) --untrusted enclave/enclave.edl --search-path $(CUSTOM_COMMON_PATH)/inc --search-path $(CUSTOM_EDL_PATH) --untrusted-dir app
	@echo "GEN => $(Enclave_EDL_Files)"

######## App Objects ########

app/enclave_u.o: $(Enclave_EDL_Files)
	@$(CC) $(App_C_Flags) -c app/enclave_u.c -o $@

$(App_Enclave_u_Object): app/enclave_u.o
	@mkdir -p $(CUSTOM_LIBRARY_PATH)
	@$(AR) rcsD $@ $^

$(App_Name): $(App_Enclave_u_Object) app
	@mkdir -p $(CUSTOM_BIN_PATH)
	@cp $(App_Rust_Path)/test $(CUSTOM_BIN_PATH)
	@echo "LINK => $@"

######## Enclave Objects ########

enclave/enclave_t.o: $(Enclave_EDL_Files)
	@$(CC) $(RustEnclave_C_Flags) -c enclave/enclave_t.c -o $@

$(RustEnclave_Name): enclave/enclave_t.o enclave
	@mkdir -p $(CUSTOM_LIBRARY_PATH)
	@mkdir -p $(CUSTOM_BIN_PATH)
	@cp $(RustEnclave_Lib_Name) $(CUSTOM_LIBRARY_PATH)/libenclave.a
	@$(CXX) enclave/enclave_t.o -o $@ $(RustEnclave_Link_Flags)
	@echo "LINK => $@"

$(RustEnclave_Signed_Name): $(RustEnclave_Name) enclave/config.xml
	@$(SGX_ENCLAVE_SIGNER) sign -key enclave/private.pem -enclave $(RustEnclave_Name) -out $@ -config enclave/config.xml
	@echo "SIGN => $@"

######## Build App ########

.PHONY: app
app:
	@cd app && SGX_SDK=$(SGX_SDK) cargo build $(App_Rust_Flags)

######## Build Enclave ########

.PHONY: enclave
enclave:
	@cd enclave && cargo build $(RustEnclave_Build_Flags)

######## Run Unit Test ########

.PHONY: test
test: $(App_Name) $(RustEnclave_Signed_Name)
	@cd bin && ./test test

######## Run Benchmark ########

.PHONY: bench
bench: $(App_Name) $(RustEnclave_Signed_Name)
	@cd bin && ./test bench

.PHONY: clean
clean:
	@rm -f $(App_Name) $(RustEnclave_Name) $(RustEnclave_Signed_Name) enclave/*_t.* app/*_u.*
	@cd enclave && cargo clean
	@cd app && cargo clean
	@rm -rf $(CUSTOM_BIN_PATH) $(CUSTOM_LIBRARY_PATH)
