#cd /root && \
#wget https://download.01.org/intel-sgx/sgx-linux/2.9/as.ld.objdump.gold.r1.tar.gz && \
#tar xzf as.ld.objdump.gold.r1.tar.gz && \
#cp external/toolset/* /usr/bin/

cd /root && \
git clone git://sourceware.org/git/binutils-gdb.git && \
cd binutils-gdb && \
git checkout fe26d3a34a223a86fddb59ed70a621a13940a088 && \
mkdir build && \
cd build && \
../configure --prefix=$BINUTILS_PREFIX --enable-gold --enable-ld=default --enable-plugins --enable-shared --disable-werror --enable-64-bit-bfd --with-system-zlib && \
make -j "$(nproc)" && \
make install && \
cd /root && \
rm -rf binutils-gdb && \
echo 'export LD_LIBRARY_PATH=/usr/lib:$BINUTILS_PREFIX' >> /root/.bashrc && \
echo 'export LD_RUN_PATH=/usr/lib:$BINUTILS_PREFIX'
