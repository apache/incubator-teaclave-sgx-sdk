# BSD License
#
# Copyright (c) 2017 Baidu, Inc. All Rights Reserved.
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

RUN apt-get update
RUN apt-get install -y build-essential ocaml automake autoconf libtool wget python libssl-dev libcurl4-openssl-dev protobuf-compiler libprotobuf-dev sudo kmod vim curl git-core libprotobuf-c0-dev libboost-thread-dev libboost-system-dev liblog4cpp5-dev libjsoncpp-dev

RUN mkdir /root/sgx
RUN wget -O /root/sgx/optimized_libs-1.9.100.39124.tar https://download.01.org/intel-sgx/linux-1.9/optimized_libs-1.9.100.39124.tar
RUN wget -O /root/sgx/prebuilt-ae-1.9.100.39124.tar  https://download.01.org/intel-sgx/linux-1.9/prebuilt-ae-1.9.100.39124.tar
RUN wget -O /root/sgx/sgx_linux_ubuntu16.04.1_x64_psw_1.9.100.39124.bin https://download.01.org/intel-sgx/linux-1.9/sgx_linux_ubuntu16.04.1_x64_psw_1.9.100.39124.bin
RUN wget -O /root/sgx/sgx_linux_ubuntu16.04.1_x64_sdk_1.9.100.39124.bin https://download.01.org/intel-sgx/linux-1.9/sgx_linux_ubuntu16.04.1_x64_sdk_1.9.100.39124.bin

RUN chmod a+x /root/sgx/sgx_linux_ubuntu16.04.1_x64_psw_1.9.100.39124.bin
RUN /root/sgx/sgx_linux_ubuntu16.04.1_x64_psw_1.9.100.39124.bin
RUN chmod a+x /root/sgx/sgx_linux_ubuntu16.04.1_x64_sdk_1.9.100.39124.bin
RUN echo -e 'no\n/opt' | /root/sgx/sgx_linux_ubuntu16.04.1_x64_sdk_1.9.100.39124.bin
RUN echo 'source /opt/sgxsdk/environment' >> /root/.bashrc

RUN git clone https://github.com/01org/linux-sgx.git /root/linux-sgx
ADD patch /root/
RUN cd /root/linux-sgx && git apply ../patch
RUN /root/linux-sgx/download_prebuilt.sh
RUN cd /root/linux-sgx && make
RUN cp /root/linux-sgx/build/linux/libsgx_tstdc.a /opt/sgxsdk/lib64/libsgx_tstdc.a

RUN wget https://static.rust-lang.org/rustup.sh
RUN sh rustup.sh --channel=nightly --date=2017-07-06
