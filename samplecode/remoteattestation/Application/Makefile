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
SGX_MODE = HW
SGX_ARCH = x64
SGX_PRERELEASE=1

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

ifeq ($(SGX_DEBUG), 1)
	SGX_COMMON_CFLAGS += -O0 -g
else
	SGX_COMMON_CFLAGS += -O2
endif

ifeq ($(SUPPLIED_KEY_DERIVATION), 1)
	SGX_COMMON_CFLAGS += -DSUPPLIED_KEY_DERIVATION
endif

SGX_COMMON_CFLAGS += -fstack-protector

######## CUSTOM Settings ########

CUSTOM_LIBRARY_PATH := ./lib
CUSTOM_BIN_PATH := ./bin
CUSTOM_EDL_PATH := ../../../edl
CUSTOM_COMMON_PATH := ../../../common

######## App Settings ########

ifneq ($(SGX_MODE), HW)
	Urts_Library_Name := sgx_urts_sim
else
	Urts_Library_Name := sgx_urts
endif

App_Cpp_Files := isv_app/isv_app.cpp ../Util/LogBase.cpp ../Networking/NetworkManager.cpp ../Networking/Session.cpp ../Networking/Server.cpp \
../Networking/Client.cpp ../Networking/NetworkManagerServer.cpp ../GoogleMessages/Messages.pb.cpp ../Networking/AbstractNetworkOps.cpp \
../Util/UtilityFunctions.cpp ../Enclave/Enclave.cpp ../MessageHandler/MessageHandler.cpp ../Util/Base64.cpp

App_Include_Paths := -I../Util -Iservice_provider -I$(SGX_SDK)/include -I$(CUSTOM_EDL_PATH) -Iheaders  -I../Networking -Iisv_app -I../GoogleMessages -I/usr/local/include -I../Enclave \
-I../MessageHandler

App_C_Flags := $(SGX_COMMON_CFLAGS) -fPIC -Wno-attributes -D_EDL_STAT_H $(App_Include_Paths)

# Three configuration modes - Debug, prerelease, release
#   Debug - Macro DEBUG enabled.
#   Prerelease - Macro NDEBUG and EDEBUG enabled.
#   Release - Macro NDEBUG enabled.
ifeq ($(SGX_DEBUG), 1)
	App_C_Flags += -DDEBUG -UNDEBUG -UEDEBUG
else ifeq ($(SGX_PRERELEASE), 1)
	App_C_Flags += -DNDEBUG -DEDEBUG -UDEBUG
else
	App_C_Flags += -DNDEBUG -UEDEBUG -UDEBUG
endif

App_Cpp_Flags := $(App_C_Flags) -std=c++11 -DEnableServer
App_Link_Flags := -L$(SGX_LIBRARY_PATH) -l$(Urts_Library_Name) -L. -L./lib -lsgx_ukey_exchange -lpthread -Wl,-rpath=$(CURDIR)/../sample_libcrypto -Wl,-rpath=$(CURDIR) -llog4cpp -lboost_system -lssl -lcrypto -lboost_thread -lprotobuf -L /usr/local/lib -ljsoncpp -lsgx_ustdc

ifneq ($(SGX_MODE), HW)
	App_Link_Flags += -lsgx_uae_service_sim
else
	App_Link_Flags += -lsgx_uae_service
endif

App_Cpp_Objects := $(App_Cpp_Files:.cpp=.o)

App_Name := app


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

#Enclave_Cpp_Files := isv_enclave/isv_enclave.cpp
Enclave_Cpp_Files :=
Enclave_Include_Paths := -I$(CUSTOM_COMMON_PATH)/inc -I$(CUSTOM_EDL_PATH) -I$(SGX_SDK)/include -I$(SGX_SDK)/include/tlibc -I$(SGX_SDK)/include/stlport -I$(SGX_SDK)/include/crypto_px/include -I../Enclave/

Enclave_C_Flags := $(SGX_COMMON_CFLAGS) $(ENCLAVE_CFLAGS) $(Enclave_Include_Paths)
Enclave_Cpp_Flags := $(Enclave_C_Flags) -std=c++11 -nostdinc++

# To generate a proper enclave, it is recommended to follow below guideline to link the trusted libraries:
#    1. Link sgx_trts with the `--whole-archive' and `--no-whole-archive' options,
#       so that the whole content of trts is included in the enclave.
#    2. For other libraries, you just need to pull the required symbols.
#       Use `--start-group' and `--end-group' to link these libraries.
# Do NOT move the libraries linked with `--start-group' and `--end-group' within `--whole-archive' and `--no-whole-archive' options.
# Otherwise, you may get some undesirable errors.
Enclave_Link_Flags := -Wl,--no-undefined -nostdlib -nodefaultlibs -nostartfiles -L$(SGX_LIBRARY_PATH) \
	-Wl,--whole-archive -l$(Trts_Library_Name) -Wl,--no-whole-archive \
	-Wl,--start-group -lsgx_tcxx -lsgx_tstdc -l$(KeyExchange_Library_Name) -l$(Crypto_Library_Name) -l$(Service_Library_Name) -L./lib -lraenclave -Wl,--end-group \
	-Wl,--version-script=enclave/enclave.lds \
	$(ENCLAVE_LDFLAGS)

Enclave_Cpp_Objects := $(Enclave_Cpp_Files:.cpp=.o)

Enclave_Name := enclave.so
Signed_Enclave_Name := enclave.signed.so
Enclave_Config_File := enclave/enclave.config.xml

ifeq ($(SGX_MODE), HW)
ifneq ($(SGX_DEBUG), 1)
ifneq ($(SGX_PRERELEASE), 1)
Build_Mode = HW_RELEASE
endif
endif
endif

.PHONY: all run

ifeq ($(Build_Mode), HW_RELEASE)
all: $(App_Name) $(Enclave_Name)
	@echo "The project has been built in release hardware mode."
	@echo "Please sign the $(Enclave_Name) first with your signing key before you run the $(App_Name) to launch and access the enclave."
	@echo "To sign the enclave use the command:"
	@echo "   $(SGX_ENCLAVE_SIGNER) sign -key <your key> -enclave $(Enclave_Name) -out <$(Signed_Enclave_Name)> -config $(Enclave_Config_File)"
	@echo "You can also sign the enclave using an external signing tool."
	@echo "To build the project in simulation mode set SGX_MODE=SIM. To build the project in prerelease mode set SGX_PRERELEASE=1 and SGX_MODE=HW."
else
all: $(App_Name) $(Signed_Enclave_Name)
endif

run: all
ifneq ($(Build_Mode), HW_RELEASE)
	@$(CURDIR)/$(App_Name)
	@echo "RUN  =>  $(App_Name) [$(SGX_MODE)|$(SGX_ARCH), OK]"
endif


######## App Objects ########

isv_app/enclave_u.c: $(SGX_EDGER8R) enclave/enclave.edl
	@$(SGX_EDGER8R) --untrusted enclave/enclave.edl --search-path $(CUSTOM_EDL_PATH) --search-path $(SGX_SDK)/include --untrusted-dir isv_app
	@echo "GEN  =>  $@"

isv_app/enclave_u.o: isv_app/enclave_u.c
	@$(CC) $(App_C_Flags) -c $< -o $@
	@echo "CC   <=  $<"

isv_app/%.o: isv_app/%.cpp
	@$(CXX) $(App_Cpp_Flags) -c $< -o $@
	@echo "CXX  <=  $<"

../MessageHandler/%.o: ../MessageHandler/%.cpp ../GeneralSettings.h
	@$(CXX) $(App_Cpp_Flags) -c $< -o $@
	@echo "CXX  <=  $<"

../Util/%.o: ../Util/%.cpp
	@$(CXX) $(App_Cpp_Flags) -c $< -o $@
	@echo "CXX  <=  $<"

../Networking/%.o: ../Networking/%.cpp ../GeneralSettings.h
	@$(CXX) $(App_Cpp_Flags) -c $< -o $@
	@echo "CXX  <=  $<"

../Enclave/%.o: ../Enclave/%.cpp
	@$(CXX) $(App_Cpp_Flags) -c $< -o $@
	@echo "CXX  <=  $<"

$(App_Name): isv_app/enclave_u.o $(App_Cpp_Objects) sgx_ustdc
	cp ../../../sgx_ustdc/libsgx_ustdc.a ./lib
	@$(CXX) isv_app/enclave_u.o $(App_Cpp_Objects) -o $@ $(App_Link_Flags)
	@echo "LINK =>  $@"

######## Enclave Objects ########

enclave/enclave_t.c: $(SGX_EDGER8R) enclave/enclave.edl
	$(SGX_EDGER8R) --trusted enclave/enclave.edl --search-path $(CUSTOM_EDL_PATH) --search-path $(SGX_SDK)/include --trusted-dir enclave
	@echo "GEN  =>  $@"

enclave/enclave_t.o: enclave/enclave_t.c
	@$(CC) $(Enclave_C_Flags) -c $< -o $@
	@echo "CC   <=  $<"

enclave/%.o: enclave/%.cpp enclave/enclave_t.c
	@$(CXX) $(Enclave_Cpp_Flags) -c $< -o $@
	@echo "CXX  <=  $<"

$(Enclave_Name): raenclave enclave/enclave_t.o $(Enclave_Cpp_Objects)
	$(CXX) enclave/enclave_t.o -o $@ $(Enclave_Link_Flags)
	@echo "LINK =>  $@"

$(Signed_Enclave_Name): $(Enclave_Name)
	mkdir -p bin
	@$(SGX_ENCLAVE_SIGNER) sign -key enclave/enclave_private.pem -enclave $(Enclave_Name) -out $@ -config $(Enclave_Config_File)
	@echo "SIGN =>  $@"


.PHONY: raenclave
raenclave:
	$(MAKE) -C ./enclave/

.PHONY: sgx_ustdc
sgx_ustdc:
	$(MAKE) -C ../../../sgx_ustdc/ 2> /dev/null

.PHONY: clean
clean:
	$(MAKE) -C ./enclave clean
	rm -f $(App_Name) $(Enclave_Name) $(Signed_Enclave_Name) $(App_Cpp_Objects) isv_app/enclave_u.* $(Enclave_Cpp_Objects) enclave/enclave_t.* libservice_provider.* $(ServiceProvider_Cpp_Objects) lib/*.a
