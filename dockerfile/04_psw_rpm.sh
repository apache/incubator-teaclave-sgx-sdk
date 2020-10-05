cd /root && \
curl --output /root/repo.tgz $PSW_REPO && \
cd /root && \
tar xzf repo.tgz && \
cd sgx_rpm_local_repo && \
rpm -ivh ./*.rpm && \
cd /root && \
mkdir /var/run/aesmd && \
rm -rf sgx_rpm_local_repo repo.tgz
