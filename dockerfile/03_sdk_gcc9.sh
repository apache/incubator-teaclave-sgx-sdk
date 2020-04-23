cd /root && \
git clone --recursive https://github.com/intel/linux-sgx && \
cd linux-sgx && \
git checkout sgx_2.9.1 && \
./download_prebuilt.sh && \
git apply /root/gcc9_patch && \
make -j "$(nproc)" sdk_install_pkg && \
echo -e 'no\n/opt' | ./linux/installer/bin/sgx_linux_x64_sdk_2.9.101.2.bin && \
echo 'source /opt/sgxsdk/environment' >> /root/.bashrc && \
cd /root && \
rm -rf /root/linux-sgx
