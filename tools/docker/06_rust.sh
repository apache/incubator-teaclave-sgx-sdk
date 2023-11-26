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
curl 'https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init' --output /root/rustup-init && \
chmod +x /root/rustup-init && \
echo '1' | /root/rustup-init --default-toolchain "$RUST_TOOLCHAIN" && \
echo 'source /root/.cargo/env' >> /root/.bashrc && \
/root/.cargo/bin/rustup component add rust-src rust-analysis && \
/root/.cargo/bin/cargo install xargo && \
rm /root/rustup-init && rm -rf /root/.cargo/registry && rm -rf /root/.cargo/git
