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
RUN apt-get install --allow-unauthenticated -y build-essential ocaml automake \
            autoconf libtool wget python libssl-dev libcurl4-openssl-dev sudo kmod vim curl \
            git-core liblog4cpp5-dev libjsoncpp-dev autoconf make g++ unzip python-dev

RUN mkdir /root/sgx
WORKDIR /root/sgx

# Download essential files
#RUN wget -O /root/sgx/optimized_libs-2.0.100.40950.tar https://download.01.org/intel-sgx/linux-1.9/optimized_libs-2.0.100.40950.tar
#RUN wget -O /root/sgx/prebuilt-ae-2.0.100.40950.tar  https://download.01.org/intel-sgx/linux-1.9/prebuilt-ae-2.0.100.40950.tar
RUN wget -O /root/sgx/sgx_linux_ubuntu16.04.1_x64_psw_2.0.100.40950.bin https://download.01.org/intel-sgx/linux-2.0/sgx_linux_ubuntu16.04.1_x64_psw_2.0.100.40950.bin
RUN wget -O /root/sgx/sgx_linux_ubuntu16.04.1_x64_sdk_2.0.100.40950.bin https://download.01.org/intel-sgx/linux-2.0/sgx_linux_ubuntu16.04.1_x64_sdk_2.0.100.40950.bin

# Setup protobuf v3.4.1
RUN wget -O /root/sgx/v3.4.1.tar.gz https://github.com/google/protobuf/archive/v3.4.1.tar.gz
RUN tar xzf v3.4.1.tar.gz
WORKDIR /root/sgx/protobuf-3.4.1
RUN ./autogen.sh && ./configure && make -j8 && make -j8 check && make -j8 install && ldconfig && cd .. && rm -rf protobuf-3.4.1 v3.4.1.tar.gz
WORKDIR /root/sgx

#RUN wget -O /root/sgx/boost_1_65_1.tar.gz https://dl.bintray.com/boostorg/release/1.65.1/source/boost_1_65_1.tar.gz
#RUN tar xzf boost_1_65_1.tar.gz
#WORKDIR /root/sgx/boost_1_65_1
#RUN ./bootstrap.sh --prefix=/usr/local --with-libraries=all && ./b2 -j8 install && cd .. && rm -rf boost_1_65_1 boost_1_65_1.tar.gz
#RUN echo "/usr/local/lib" > /etc/ld.so.conf.d/boost.conf && ldconfig
#WORKDIR /root/sgx

RUN chmod a+x /root/sgx/sgx_linux_ubuntu16.04.1_x64_psw_2.0.100.40950.bin
RUN /root/sgx/sgx_linux_ubuntu16.04.1_x64_psw_2.0.100.40950.bin
RUN chmod a+x /root/sgx/sgx_linux_ubuntu16.04.1_x64_sdk_2.0.100.40950.bin
RUN echo -e 'no\n/opt' | /root/sgx/sgx_linux_ubuntu16.04.1_x64_sdk_2.0.100.40950.bin
RUN echo 'source /opt/sgxsdk/environment' >> /root/.bashrc

RUN wget -O /root/sgx/sgx_2.0.tar.gz https://github.com/01org/linux-sgx/archive/sgx_2.0.tar.gz
RUN tar xzf sgx_2.0.tar.gz
ADD all.patch /root/sgx
RUN patch -t -p0 < all.patch
WORKDIR linux-sgx-sgx_2.0
RUN /root/sgx/linux-sgx-sgx_2.0/download_prebuilt.sh
RUN make -j8
RUN cp /root/sgx/linux-sgx-sgx_2.0/build/linux/libsgx_tstdc.a /opt/sgxsdk/lib64/libsgx_tstdc.a
RUN cp /root/sgx/linux-sgx-sgx_2.0/build/linux/aesm_service /opt/intel/sgxpsw/aesm/aesm_service
RUN cp /root/sgx/linux-sgx-sgx_2.0/build/linux/libsgx_uae_service.so /usr/lib/libsgx_uae_service.so
RUN apt-get autoclean && apt-get autoremove && rm -rf /var/cache/apt/archives/*
#RUN git clone https://github.com/01org/linux-sgx.git /root/linux-sgx
#ADD patch /root/
#RUN cd /root/linux-sgx && git apply ../patch
#RUN /root/linux-sgx/download_prebuilt.sh
#RUN cd /root/linux-sgx && make
#RUN cp /root/linux-sgx/build/linux/libsgx_tstdc.a /opt/sgxsdk/lib64/libsgx_tstdc.a

RUN wget 'https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init' -O /root/rustup-init
RUN chmod +x /root/rustup-init
RUN echo '1' | /root/rustup-init --default-toolchain nightly-2017-11-29
RUN echo 'source /root/.cargo/env' >> /root/.bashrc

# Remove useless .h file in intel sgx sdk, which always cause linking problem.
# RUN ar d /opt/sgxsdk/lib64/libsgx_uprotected_fs.a sgx_tprotected_fs_u.h
# RUN ar d /opt/sgxsdk/lib64/libsgx_tprotected_fs.a sgx_tprotected_fs_t.h

## A hack of linking Rust's libc (https://github.com/rust-lang/libc) in no_std environment.
RUN rm /root/.rustup/toolchains/nightly-2017-11-29-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/lib/liblibc-7f54f8f6cd32e0c8.rlib

WORKDIR /root
