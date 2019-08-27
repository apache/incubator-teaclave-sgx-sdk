#!/usr/bin/env bash
git clone https://github.com/gel-sgx/SGX_SQLite.git
mv SGX_SQLite/Enclave/sqlite3.c ../../sgx_db/sqlite3/
mv SGX_SQLite/Enclave/sqlite3.h ../../sgx_db/sqlite3/
rm -r SGX_SQLite


echo "build sqlite3"

cc -I/opt/sgxsdk/include -DSQLITE_THREADSAFE=0 -E ../../sgx_db/sqlite3/sqlite3.c -o ../../sgx_db/sqlite3/sqlite3.i
cc -m64 -O2  -nostdinc -fvisibility=hidden -fpie -ffunction-sections -fdata-sections -fstack-protector-strong -IEnclave -I/opt/sgxsdk/include -I/opt/sgxsdk/include/tlibc -I/opt/sgxsdk/include/libcxx -DSQLITE_THREADSAFE=0 -c ../../sgx_db/sqlite3/sqlite3.i -o ../../sgx_db/sqlite3/sqlite3.o
