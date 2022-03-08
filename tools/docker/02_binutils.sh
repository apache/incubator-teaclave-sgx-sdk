#!/bin/bash

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

cd /root && \
wget https://download.01.org/intel-sgx/sgx-linux/$SGX_SDK_RELEASE_VERSION/as.ld.objdump.r4.tar.gz && \
tar xzf as.ld.objdump.r4.tar.gz && \
cp -r external/toolset/$BINUTILS_DIST/* /usr/bin/ && \
rm -rf ./external ./as.ld.objdump.r4.tar.gz
