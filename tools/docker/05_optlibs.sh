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

optlibs_name=optimized_libs_${SGX_SDK_RELEASE_VERSION}.tar.gz
checksum_file_name=SHA256SUM_prebuilt_${SGX_SDK_RELEASE_VERSION}.cfg

cd /root && \
wget https://download.01.org/intel-sgx/sgx-linux/${SGX_SDK_RELEASE_VERSION}/${optlibs_name} && \
wget https://download.01.org/intel-sgx/sgx-linux/${SGX_SDK_RELEASE_VERSION}/${checksum_file_name} && \
grep $optlibs_name $checksum_file_name | sha256sum -c && \
mkdir -p /opt/intel/optimized_libs && \
tar -zxf $optlibs_name -C /opt/intel/optimized_libs && \
rm -f $optlibs_name $checksum_file_name && \
echo "export OPT_LIBS_PATH=opt/intel/optimized_libs" >> /root/.bashrc
