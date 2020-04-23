source /opt/sgxsdk/environment && \
cd /root && \
git clone --recursive https://github.com/intel/linux-sgx && \
cd linux-sgx && \
git checkout sgx_2.9.1 && \
git apply /root/centos_patch && \
./download_prebuilt.sh && \
cd external/dcap_source && \
git apply /root/centos_dcap_patch && \
cd /root/linux-sgx && \
echo 'source /opt/sgxsdk/environment' >> /root/.bashrc && \
make rpm_local_repo
#cd linux/installer/rpm && \
#find . -maxdepth 2 -name '*.deb' | grep -v pccs | xargs dpkg -i
#cd /root && \ rm -rf /root/linux-sgx
