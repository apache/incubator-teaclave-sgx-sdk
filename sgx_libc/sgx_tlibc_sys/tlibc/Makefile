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

USE_OPTLIBS ?= $(USE_OPT_LIBS)
OUT_DIR 	?= ./out

SGX_COMMON_FLAGS := -O2 -D_FORTIFY_SOURCE=2 -UDEBUG -DNDEBUG
CFLAGS   += $(SGX_COMMON_FLAGS) $(ENCLAVE_CFLAGS) -D_TLIBC_GNU_
ASFLAGS  := $(CFLAGS)
CFLAGS   += -std=c99
CXXFLAGS += $(SGX_COMMON_FLAGS) $(ENCLAVE_CXXFLAGS) -fno-exceptions -fno-rtti

CPPFLAGS += -I.                          \
            -I$(COMMON_DIR)/inc          \
            -I$(COMMON_DIR)/inc/internal \
            -I$(COMMON_DIR)/inc/tlibc    \
            -I./string/asm               \
            -I./gdtoa

EXTERNAL_DIR := $(OPT_LIBS_PATH)/optimized_libs/external
LIBM_PATH    := $(EXTERNAL_DIR)/libm/Libm_ct_Linux
LIBIRC_PATH  := $(EXTERNAL_DIR)/libirc/efi2
SVML_PATH    := $(EXTERNAL_DIR)/libm/SVML_Intel64_Linux

ifeq ($(USE_OPTLIBS), 1)
CFLAGS += -D_TLIBC_USE_INTEL_FAST_STRING_
CXXFLAGS += -D_TLIBC_USE_INTEL_FAST_STRING_
endif

ifeq ($(USE_OPTLIBS), 0)
LIBC_C_SRCS := $(wildcard gen/*.c gdtoa/*.c locale/*.c math/*.c stdlib/*.c string/*.c stdio/*.c time/*.c)
LIBC_CPP_SRCS := $(wildcard gen/*.cpp)
LIBC_ASM_SRCS := $(wildcard gen/*.S math/*.S string/asm/*.S)
else
C_SRCS := $(wildcard gen/*.c gdtoa/*.c locale/*.c math/opt/*.c stdlib/*.c string/*.c stdio/*.c time/*.c)
LIBC_C_SRCS := $(filter-out $(wildcard gen/fpclassify*.c gen/isfinite*.c gen/isinf*.c gen/isnan*.c gen/isnormal*.c gen/signbit*.c), $(C_SRCS))
LIBC_CPP_SRCS := $(wildcard gen/*.cpp)
LIBC_ASM_SRCS := $(wildcard gen/*.S string/asm/*.S)
endif

LIBC_OBJS := $(LIBC_C_SRCS:.c=.o)
LIBC_OBJS += $(LIBC_CPP_SRCS:.cpp=.o)
LIBC_OBJS += $(LIBC_ASM_SRCS:.S=.o)
LIBC_OBJS := $(sort $(LIBC_OBJS))
LIBC_OBJS := $(addprefix $(OUT_DIR)/, $(LIBC_OBJS))

TLIBC_NAME := libtlibc.a
BUILD_DIR := build_dir
LIBC_NAME := $(OUT_DIR)/$(TLIBC_NAME)

# SDK requires prebuilt opt libraries.
CHECK_OPT :=
ifeq ($(USE_OPTLIBS), 1)
ifeq ("$(wildcard $(EXTERNAL_DIR)/libm)", "")
CHECK_OPT := opt_check_failed
endif
ifeq ("$(wildcard $(EXTERNAL_DIR)/libirc)", "")
CHECK_OPT := opt_check_failed
endif
endif

.PHONY: all
all: $(CHECK_OPT) $(BUILD_DIR) $(LIBC_NAME)

$(LIBC_NAME): $(LIBC_OBJS)
	$(AR) rcs $@ $^
ifeq ($(USE_OPTLIBS), 1)
	$(RM) -f $(OUT_DIR)/libirc/* && cd $(OUT_DIR)/libirc && $(AR) x $(LIBIRC_PATH)/libirc.a
	$(RM) -f $(OUT_DIR)/libimf/* && cd $(OUT_DIR)/libimf && $(AR) x $(LIBM_PATH)/libimf.a
	$(RM) -f $(OUT_DIR)/libsvml/* && cd $(OUT_DIR)/libsvml && $(AR) x $(SVML_PATH)/libsvml.a
	@cd $(OUT_DIR) && $(AR) qD $(TLIBC_NAME) libirc/*.o
	@cd $(OUT_DIR) && $(AR) qD $(TLIBC_NAME) libimf/*.o
	@cd $(OUT_DIR) && $(AR) qD $(TLIBC_NAME) libsvml/*.o
endif

$(OUT_DIR)/string/consttime_memequal.o: string/consttime_memequal.c
	$(CC)  $(filter-out -O2,$(CFLAGS)) -O1 $(CPPFLAGS) -c $< -o $@
	
$(OUT_DIR)/%.o: %.c
	$(CC)  $(CFLAGS)   $(CPPFLAGS) -c $< -o $@

$(OUT_DIR)/%.o: %.cpp
	$(CXX) $(CXXFLAGS) $(CPPFLAGS) -c $< -o $@

$(OUT_DIR)/%.o: %.S
	$(CC)  $(ASFLAGS)  $(CPPFLAGS) -c $< -o $@

$(BUILD_DIR):
	@$(MKDIR) $(OUT_DIR)
	@$(MKDIR) $(OUT_DIR)/gen
	@$(MKDIR) $(OUT_DIR)/gdtoa
	@$(MKDIR) $(OUT_DIR)/locale
	@$(MKDIR) $(OUT_DIR)/math
	@$(MKDIR) $(OUT_DIR)/setjmp
	@$(MKDIR) $(OUT_DIR)/stdlib
	@$(MKDIR) $(OUT_DIR)/string
	@$(MKDIR) $(OUT_DIR)/string/asm
	@$(MKDIR) $(OUT_DIR)/stdio
	@$(MKDIR) $(OUT_DIR)/time
ifeq ($(USE_OPTLIBS), 1)
	@$(MKDIR) $(OUT_DIR)/math/opt
	@$(MKDIR) $(OUT_DIR)/libirc
	@$(MKDIR) $(OUT_DIR)/libimf
	@$(MKDIR) $(OUT_DIR)/libsvml
endif

.PHONY: opt_check_failed
opt_check_failed:
	@echo "ERROR: Please run 'download_prebuilt.sh' to download the prebuilt optimized libraries before compiling."
	@echo "Exiting......"
	@exit 1

.PHONY: clean
clean:
	@$(RM) -rf $(OUT_DIR)

