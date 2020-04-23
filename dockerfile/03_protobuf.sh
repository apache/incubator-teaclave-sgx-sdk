cd /root && wget -O /root/v3.11.4.tar.gz https://github.com/google/protobuf/archive/v3.11.4.tar.gz && \
tar xzf v3.11.4.tar.gz && \
cd /root/protobuf-3.11.4 && \
./autogen.sh && ./configure && make -j && make -j install && ldconfig && cd .. && rm -rf protobuf-3.11.4 v3.11.4.tar.gz
