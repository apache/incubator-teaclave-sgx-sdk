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
Rust_Enclave_Name := libenclave.a
Rust_Enclave_Files := $(wildcard src/*.rs)
Rust_Target_Path := $(CURDIR)/../../../xargo

ifeq ($(MITIGATION-CVE-2020-0551), LOAD)
export MITIGATION_CVE_2020_0551=LOAD
else ifeq ($(MITIGATION-CVE-2020-0551), CF)
export MITIGATION_CVE_2020_0551=CF
endif

ifneq ($(COV),) # Debug build + coverage collection
	SGX_ENCLAVE_FEATURES = -Z package-features --features "cov"
	COV_FLAGS = CARGO_INCREMENTAL=0 \
	    RUSTFLAGS="-D warnings -Zprofile -Ccodegen-units=1 \
	    -Cllvm_args=-inline-threshold=0 -Clink-dead-code \
	    -Coverflow-checks=off -Cpanic=abort"

	WRAPPER = "./enclave-cov-rustc"
endif

.PHONY: all

all: $(Rust_Enclave_Name)

$(Rust_Enclave_Name): $(Rust_Enclave_Files)
ifeq ($(XARGO_SGX), 1)
	CARGO_INCREMENTAL=0 RUSTC_WRAPPER=${WRAPPER} RUST_TARGET_PATH=$(Rust_Target_Path) xargo build --target x86_64-unknown-linux-sgx ${SGX_ENCLAVE_FEATURES}
	cp ./target/x86_64-unknown-linux-sgx/debug/libsgxcovenclave.a ../lib/libenclave.a
else
	CARGO_INCREMENTAL=0 RUSTC_WRAPPER=${WRAPPER} cargo build ${SGX_ENCLAVE_FEATURES}
	cp ./target/debug/libsgxcovenclave.a ../lib/libenclave.a
endif
