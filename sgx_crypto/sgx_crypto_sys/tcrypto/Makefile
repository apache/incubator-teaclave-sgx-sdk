#
# Copyright (C) 2011-2021 Intel Corporation. All rights reserved.
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions
# are met:
#
#   * Redistributions of source code must retain the above copyright
#     notice, this list of conditions and the following disclaimer.
#   * Redistributions in binary form must reproduce the above copyright
#     notice, this list of conditions and the following disclaimer in
#     the documentation and/or other materials provided with the
#     distribution.
#   * Neither the name of Intel Corporation nor the names of its
#     contributors may be used to endorse or promote products derived
#     from this software without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
# "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
# LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
# A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
# OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
# SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
# LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
# DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
# THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
# (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
#
#

include ../../../buildenv.mk

IPP_SUBDIR = no_mitigation
ifeq ($(MITIGATION_CVE_2020_0551), LOAD)
    IPP_SUBDIR = cve_2020_0551_load
else ifeq ($(MITIGATION-CVE-2020-0551), CF)
    IPP_SUBDIR = cve_2020_0551_cf
endif

EXTERNAL_DIR 	:= $(OPT_LIBS_PATH)/optimized_libs/external
IPP_DIR         := ./ipp
IPP_INC         := $(IPP_DIR)/inc
IPPCP_DIR       := $(EXTERNAL_DIR)/ippcp_internal
IPPCP_INC       := $(IPPCP_DIR)/inc
IPP_LIBS_DIR    := $(IPPCP_DIR)/lib/linux/intel64/$(IPP_SUBDIR)
LD_IPP          := -lippcp

BUILD_ARG ?= tcrypto
OUT_DIR ?= ./out
CPPFLAGS := -I$(COMMON_DIR)/inc/internal \
            -I$(COMMON_DIR)/inc          \
            -I$(IPP_INC)                 \
            -I$(IPPCP_INC)

ifeq ($(BUILD_ARG), tcrypto)
	CPPFLAGS += -I$(COMMON_DIR)/inc/tlibc
	CXXFLAGS += $(ENCLAVE_CXXFLAGS)
else
	CPPFLAGS += -D_UCRYPTO_
	CXXFLAGS += -ffreestanding -fvisibility=hidden -fpie -fno-strict-overflow -fno-delete-null-pointer-checks
endif

CXXFLAGS += -fno-exceptions -fno-rtti #-Werror

OBJ = init_tcrypto_lib.o sgx_aes_ctr.o sgx_rsa_encryption.o sgx_aes_gcm.o sgx_cmac128.o sgx_hmac_sha256.o \
	sgx_ecc256.o sgx_ecc256_ecdsa.o sgx_sha256.o sgx_sha1.o sgx_sha256_msg.o sgx_ecc256_internal.o  \
	sgx_rsa2048.o sgx_rsa3072.o sgx_internal.o sgx_sm3_msg.o sgx_sm3.o sgx_sm4_ccm.o sgx_sm4_cbc.o \
	sgx_sm4_ctr.o sgx_hmac_sm3.o sgx_sm2.o sgx_sm2_sign.o sgx_sm2_internal.o sgx_aes_cbc.o sgx_aes_ccm.o
SHARED_OBJ = sgx_common_init_ipp.o

# Build IPP crypto based sgx_tcrypto library
CFLAGS += $(CPPFLAGS) $(ENCLAVE_CFLAGS) $(SGX_COMMON_CFLAGS) -Werror -fno-exceptions -fPIC

OBJ += sgx_tcrypto_common.o sgx_rsa_internal.o
SRCDIR := ipp
DISPDIR := $(SRCDIR)/ipp_disp

LIB_NAME := libsgx_tcrypto_ipp.a

C_Files := $(wildcard $(DISPDIR)/*.c)
C_OBJS  := $(C_Files:.c=.o)
C_OBJS  := $(sort $(C_OBJS))
C_OBJS  := $(addprefix $(OUT_DIR)/, $(C_OBJS))

DISP_LIB_NAME := libsgx_disp.a

OBJ := $(addprefix $(OUT_DIR)/$(SRCDIR)/, $(OBJ))
SHARED_OBJ := $(addprefix $(OUT_DIR)/, $(SHARED_OBJ))
DISP_LIB_NAME := $(addprefix $(OUT_DIR)/$(DISPDIR)/, $(DISP_LIB_NAME))
LIB_NAME := $(addprefix $(OUT_DIR)/$(SRCDIR)/, $(LIB_NAME))
TARGET := $(OUT_DIR)/libsgx_tcrypto.a

BUILD_DIR := build_dir

$(TARGET): $(OBJ) $(SHARED_OBJ) $(C_OBJS)
	$(CP) $(IPP_LIBS_DIR)/libippcp.a $(LIB_NAME)
	$(AR) rsD $(LIB_NAME) $(OBJ) $(SHARED_OBJ) $(C_OBJS)
	$(CP) $(LIB_NAME) $@

$(DISP_LIB_NAME): $(C_OBJS)
	$(AR) rsD $(DISP_LIB_NAME) $(C_OBJS)

$(C_OBJS): $(OUT_DIR)/$(DISPDIR)/%.o: $(DISPDIR)/%.c
	$(CC) -c $(CFLAGS) $< -o $@

$(OUT_DIR)/$(SRCDIR)/%.o: $(SRCDIR)/%.cpp
	$(CXX) -c $(CXXFLAGS) $(CPPFLAGS) $< -o $@

$(SHARED_OBJ): $(OUT_DIR)/%.o: %.cpp
	$(CXX) -c $(CXXFLAGS) $(CPPFLAGS) $< -o $@

$(BUILD_DIR):
	@$(MKDIR) $(OUT_DIR)
	@$(MKDIR) $(OUT_DIR)/$(SRCDIR)
	@$(MKDIR) $(OUT_DIR)/$(DISPDIR)

# SDK requires prebuilt IPP libraries.
CHECK_OPT :=
ifeq ("$(wildcard $(IPPCP_DIR)/lib/linux/intel64)", "")
CHECK_OPT := opt_check_failed
endif

.PHONY: all
all: $(CHECK_OPT) $(BUILD_DIR) $(TARGET) $(DISP_LIB_NAME)

.PHONY: opt_check_failed
opt_check_failed:
	@echo "ERROR: Please run 'download_prebuilt.sh' to download the prebuilt optimized libraries before compiling."
	@echo "Exiting......"
	@exit 1

.PHONY: clean
clean:
	@$(RM) -rf $(OUT_DIR)

.PHONY: rebuild
rebuild:
	$(MAKE) clean
	$(MAKE) all
