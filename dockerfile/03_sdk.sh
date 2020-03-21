apt-get update && apt-get install -y cmake ocaml && \
cd /root && \
git clone --recursive https://github.com/intel/linux-sgx && \
cd linux-sgx && \
git checkout sgx_2.9 && \
./download_prebuilt.sh && \
make -j "$(nproc)" sdk_install_pkg && \
echo -e 'no\n/opt' | ./linux/installer/bin/sgx_linux_x64_sdk_2.9.100.2.bin && \
echo 'source /opt/sgxsdk/environment' >> /root/.bashrc && \
cd /root && \
rm -rf /root/linux-sgx && \
rm -rf /var/lib/apt/lists/* && \
rm -rf /var/cache/apt/archives/*
