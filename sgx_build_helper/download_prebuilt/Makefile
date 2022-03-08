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
#$(shell ./download_prebuilt.sh || echo "Fail to download optimized libs.")

TOP_DIR := ../..
include $(TOP_DIR)/buildenv.mk

EXTERNAL_DIR 	:= $(OPT_LIBS_PATH)/optimized_libs/external
IPPCP_DIR       := $(EXTERNAL_DIR)/ippcp_internal

export OPT_LIBS_PATH ?= $(OPT_LIBS_PATH)

.PHONY: all
all:
ifeq ("$(wildcard $(IPPCP_DIR)/lib/linux/intel64)", "")
	@./download_prebuilt.sh || exit 1
endif

.PHONY: clean
clean:
	@$(RM) -rf $(OPT_LIBS_PATH)/optimized_libs
