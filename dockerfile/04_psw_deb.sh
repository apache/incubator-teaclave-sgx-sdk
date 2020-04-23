source /opt/sgxsdk/environment && \
cd /root && \
git clone --recursive https://github.com/intel/linux-sgx && \
cd linux-sgx && \
git checkout sgx_2.9.1 && \
git apply /root/focal_psw_patch && \
./download_prebuilt.sh && \
make deb_local_repo && \
cd linux/installer/deb && \
find . -maxdepth 2 -name '*.deb' | grep -v pccs | xargs dpkg -i
#cd /root && \ rm -rf /root/linux-sgx
