#!/usr/bin/env bash

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

set -e

# change this on every release
optlibs_name=optimized_libs_2.17.tar.gz
checksum_file_name=SHA256SUM_prebuilt_2.17.cfg
server_url_path=https://download.01.org/intel-sgx/sgx-linux/2.17

# unlikely to change unless opt lib structure changes
top_dir=$(dirname "$(realpath "$0")")
if [[ -n $OPT_LIBS_PATH && -d $OPT_LIBS_PATH ]]; then
    out_dir=${OPT_LIBS_PATH%*/}/optimized_libs
else
    out_dir=$top_dir/optimized_libs
fi
optlibs_file=$out_dir/$optlibs_name
optlibs_url=$server_url_path/$optlibs_name
checksum_file=$out_dir/$checksum_file_name
checksum_url=$server_url_path/$checksum_file_name


mkdir -p $out_dir && \
rm -f $optlibs_file $checksum_file

wget -O "$optlibs_file" "$optlibs_url" || \
(echo "Fail to download file $optlibs_url"; exit 1)

wget -O "$checksum_file" "$checksum_url" || \
(echo "Fail to download file $checksum_url"; exit 1)

pushd "$out_dir"

grep $optlibs_name $checksum_file_name | sha256sum -c || \
(echo "Checksum verification failure"; exit 1)

tar -zxf $optlibs_name && \
rm -f $optlibs_name $checksum_file_name

popd
