apt-get update && apt-get install -y dkms gnupg2 apt-transport-https software-properties-common && \
    curl -fsSL  https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | apt-key add - && \
    add-apt-repository "deb https://download.01.org/intel-sgx/sgx_repo/ubuntu $CODENAME main" && \
    apt-get update && \
    apt-get install -y \
        libsgx-aesm-launch-plugin=$VERSION \
        libsgx-enclave-common=$VERSION \
        libsgx-enclave-common-dev=$VERSION \
        libsgx-epid=$VERSION \
        libsgx-epid-dev=$VERSION \
        libsgx-launch=$VERSION \
        libsgx-launch-dev=$VERSION \
        libsgx-quote-ex=$VERSION \
        libsgx-quote-ex-dev=$VERSION \
        libsgx-uae-service=$VERSION \
        libsgx-urts=$VERSION && \
    mkdir /var/run/aesmd && \
    rm -rf /var/lib/apt/lists/* && \
    rm -rf /var/cache/apt/archives/*
