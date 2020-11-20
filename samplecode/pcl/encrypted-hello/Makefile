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

SGX_SDK ?= /opt/sgxsdk
SGX_MODE ?= HW
SGX_ARCH ?= x64

TOP_DIR := ../../..
include $(TOP_DIR)/buildenv.mk

ifeq ($(shell getconf LONG_BIT), 32)
	SGX_ARCH := x86
else ifeq ($(findstring -m32, $(CXXFLAGS)), -m32)
	SGX_ARCH := x86
endif

ifeq ($(SGX_ARCH), x86)
	SGX_COMMON_CFLAGS := -m32
	SGX_LIBRARY_PATH := $(SGX_SDK)/lib
	SGX_ENCLAVE_SIGNER := $(SGX_SDK)/bin/x86/sgx_sign
	SGX_EDGER8R := $(SGX_SDK)/bin/x86/sgx_edger8r
else
	SGX_COMMON_CFLAGS := -m64
	SGX_LIBRARY_PATH := $(SGX_SDK)/lib64
	SGX_ENCLAVE_SIGNER := $(SGX_SDK)/bin/x64/sgx_sign
	SGX_EDGER8R := $(SGX_SDK)/bin/x64/sgx_edger8r
endif

ifeq ($(SGX_DEBUG), 1)
ifeq ($(SGX_PRERELEASE), 1)
$(error Cannot set SGX_DEBUG and SGX_PRERELEASE at the same time!!)
endif
endif

Pcl_Encryption_Tool := $(SGX_SDK)/bin/x64/sgx_encrypt
PCL_KEY := ../key.bin
PCL_KEY_SIZE := 16

ifneq ($(SGX_MODE), HW)
PCL_LIB_NAME := sgx_pclsim
else  # ifneq ($(SGX_MODE), HW)
PCL_LIB_NAME := sgx_pcl
endif # ifneq ($(SGX_MODE), HW)

PCL_LINK_FLAGS := -Wl,--whole-archive -l$(PCL_LIB_NAME) -Wl,--no-whole-archive

ifeq ($(SGX_DEBUG), 1)
	SGX_COMMON_CFLAGS += -O0 -g
	Encryption_Tool_Flags := -d
else
	SGX_COMMON_CFLAGS += -O2
endif

SGX_COMMON_CFLAGS += -fstack-protector

######## CUSTOM Settings ########

CUSTOM_LIBRARY_PATH := ./lib
CUSTOM_BIN_PATH := ./bin
CUSTOM_EDL_PATH := ../../../edl
CUSTOM_COMMON_PATH := ../../../common

######## EDL Settings ########

Enclave_EDL_Files := enclave/Enclave_t.c enclave/Enclave_t.h app/Enclave_u.c app/Enclave_u.h

######## Enclave Settings ########

ifneq ($(SGX_MODE), HW)
	Trts_Library_Name := sgx_trts_sim
	Service_Library_Name := sgx_tservice_sim
else
	Trts_Library_Name := sgx_trts
	Service_Library_Name := sgx_tservice
endif
Crypto_Library_Name := sgx_tcrypto
KeyExchange_Library_Name := sgx_tkey_exchange
ProtectedFs_Library_Name := sgx_tprotected_fs

RustEnclave_C_Files := $(wildcard ./enclave/*.c)
RustEnclave_C_Objects := $(RustEnclave_C_Files:.c=.o)
RustEnclave_Include_Paths := -I$(CUSTOM_COMMON_PATH)/inc -I$(CUSTOM_EDL_PATH) -I$(SGX_SDK)/include -I$(SGX_SDK)/include/tlibc -I$(SGX_SDK)/include/stlport -I$(SGX_SDK)/include/epid -I ./enclave -I./include

RustEnclave_Link_Libs := -L$(CUSTOM_LIBRARY_PATH) -lenclave
RustEnclave_Compile_Flags := $(SGX_COMMON_CFLAGS) $(ENCLAVE_CFLAGS) $(RustEnclave_Include_Paths)
RustEnclave_Link_Flags := -Wl,--no-undefined -nostdlib -nodefaultlibs -nostartfiles -L$(SGX_LIBRARY_PATH) \
	-Wl,--whole-archive -l$(Trts_Library_Name) -Wl,--no-whole-archive \
	-Wl,--start-group -lsgx_tstdc -l$(Service_Library_Name) -l$(Crypto_Library_Name) $(RustEnclave_Link_Libs) -Wl,--end-group \
	-Wl,--version-script=enclave/Enclave.lds \
	$(ENCLAVE_LDFLAGS)

RustEnclave_Name := enclave/enclave.so
Encrypted_RustEnclave_Name := $(RustEnclave_Name).enc
Signed_RustEnclave_Name := bin/enclave.signed.so

.PHONY: all
all: $(Signed_RustEnclave_Name)

######## PCL KEY ########
../key.bin:
	dd if=/dev/random of=$@ count=1 bs=$(PCL_KEY_SIZE)

######## EDL Objects ########

$(Enclave_EDL_Files): $(SGX_EDGER8R) enclave/Enclave.edl
	$(SGX_EDGER8R) --trusted enclave/Enclave.edl --search-path $(SGX_SDK)/include --search-path $(CUSTOM_EDL_PATH) --trusted-dir enclave
	$(SGX_EDGER8R) --untrusted enclave/Enclave.edl --search-path $(SGX_SDK)/include --search-path $(CUSTOM_EDL_PATH) --untrusted-dir app
	@echo "GEN  =>  $(Enclave_EDL_Files)"

######## Enclave Objects ########

enclave/Enclave_t.o: $(Enclave_EDL_Files)
	@$(CC) $(RustEnclave_Compile_Flags) -c enclave/Enclave_t.c -o $@
	@echo "CC   <=  $<"

$(RustEnclave_Name): enclave enclave/Enclave_t.o
	@$(CXX) enclave/Enclave_t.o -o $@ $(RustEnclave_Link_Flags) $(PCL_LINK_FLAGS)
	@echo "LINK =>  $@"

$(Signed_RustEnclave_Name): $(RustEnclave_Name) $(Pcl_Encryption_Tool) $(PCL_KEY)
	@echo -n "Encrypt key: "
	@hexdump -v -e '/1 "%02X"' $(PCL_KEY)
	@echo ""
	$(Pcl_Encryption_Tool) -i $(RustEnclave_Name) -o $(Encrypted_RustEnclave_Name) -k $(PCL_KEY) $(Encryption_Tool_Flags)
	@echo "Encrypted => $(Encrypted_RustEnclave_Name)"
	mkdir -p bin
	@$(SGX_ENCLAVE_SIGNER) sign -key enclave/Enclave_private.pem -enclave $(Encrypted_RustEnclave_Name) -out $@ -config enclave/Enclave.config.xml
	@echo "SIGN =>  $@"

.PHONY: enclave
enclave:
	$(MAKE) -C ./enclave/


.PHONY: clean
clean:
	@rm -f $(RustEnclave_Name) $(Encrypted_RustEnclave_Name) $(Signed_RustEnclave_Name) enclave/*_t.* app/*_u.* lib/*.a
	@cd enclave && cargo clean && rm -f Cargo.lock
