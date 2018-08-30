# BSD License
#
# Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
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
#   * Neither the name of Baidu, Inc., nor the names of its
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


FROM ubuntu:16.04
MAINTAINER Yu Ding

RUN apt-get update && \ 
    apt-get install -y --no-install-recommends ca-certificates build-essential ocaml automake autoconf libtool wget python libssl-dev libcurl4-openssl-dev protobuf-compiler libprotobuf-dev sudo kmod vim curl git-core libprotobuf-c0-dev libboost-thread-dev libboost-system-dev liblog4cpp5-dev libjsoncpp-dev alien uuid-dev libxml2-dev cmake pkg-config expect && \
    rm -rf /var/lib/apt/lists/* &&  rm -rf /var/cache/apt/archives/*


# Uncomment the following lines for setup iCls
# Make sure you have put the installer file in this directory with filename iclsClient-1.45.449.12-1.x86_64.rpm
#RUN mkdir /root/icls
#ADD iclsClient-1.45.449.12-1.x86_64.rpm /root/icls/
#RUN cd /root/icls && alien --scripts iclsClient-1.45.449.12-1.x86_64.rpm && dpkg -i iclsclient_1.45.449.12-2_amd64.deb && ldconfig && \
#    cd /tmp && \
#    git clone https://github.com/01org/dynamic-application-loader-host-interface.git jhi && \
#    cd jhi && mkdir build && cd build && cmake .. && make && make install && \
#    systemctl enable jhi

RUN mkdir /root/sgx && \
    wget -O /root/sgx/psw.bin https://download.01.org/intel-sgx/linux-2.2/ubuntu64-desktop/sgx_linux_x64_psw_2.2.100.45311.bin && \
    wget -O /root/sgx/sdk.bin https://download.01.org/intel-sgx/linux-2.2/ubuntu64-desktop/sgx_linux_x64_sdk_2.2.100.45311.bin && \
    cd /root/sgx && \
    chmod +x /root/sgx/psw.bin && \
    /root/sgx/psw.bin && \
    chmod +x /root/sgx/sdk.bin && \
    echo -e 'no\n/opt' | /root/sgx/sdk.bin && \
    echo 'source /opt/sgxsdk/environment' >> /root/.bashrc && \
    rm -rf /root/sgx/*

ADD patch /root/

RUN wget -O /root/src.tar.gz https://github.com/intel/linux-sgx/archive/sgx_2.2.tar.gz && \
    cd /root && tar xzf src.tar.gz && \
    cd /root/linux-sgx-sgx_2.2 && git apply ../patch && \
    /root/linux-sgx-sgx_2.2/download_prebuilt.sh && \
    cd /root/linux-sgx-sgx_2.2 && make -j && \
    cp /root/linux-sgx-sgx_2.2/build/linux/libsgx_tstdc.a /opt/sgxsdk/lib64/libsgx_tstdc.a && \
    rm /root/src.tar.gz && rm -rf /root/linux-sgx-sgx_2.2

RUN wget 'https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init' -O /root/rustup-init && \
    chmod +x /root/rustup-init && \
    echo '1' | /root/rustup-init --default-toolchain nightly-2018-07-16 && \
    echo 'source /root/.cargo/env' >> /root/.bashrc && \
    /root/.cargo/bin/rustup component add rust-src && \
    /root/.cargo/bin/cargo install xargo && \
    rm -rf /root/.cargo/registry && rm -rf /root/.cargo/git
WORKDIR /root
