cd /root && \
curl --output gcc.tar.gz http://ftp.mirrorservice.org/sites/sourceware.org/pub/gcc/releases/gcc-8.4.0/gcc-8.4.0.tar.gz && \
tar xzf gcc.tar.gz && \
cd gcc-8.4.0 && \
./contrib/download_prerequisites && \
mkdir build && \
cd build && \
../configure --disable-multilib --enable-languages=c,c++,fortran,go && \
make -j $(nproc) && \
make install && \
cd /root && \
rm -rf gcc-8.4.0
