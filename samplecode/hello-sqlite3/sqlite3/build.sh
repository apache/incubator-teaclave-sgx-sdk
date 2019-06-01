#!/usr/bin/env bash
rm -rf sqlite3.o
rm -rf sqlite3.i
rm -rf ocall_interface.i
rm -rf ocall_interface.o
echo "build sqlite3"

cc -I/opt/sgxsdk/include -DSQLITE_THREADSAFE=0 -E sqlite3.c -o sqlite3.i
cc -m64 -O2  -nostdinc -fvisibility=hidden -fpie -ffunction-sections -fdata-sections -fstack-protector-strong -IEnclave -I/opt/sgxsdk/include -I/opt/sgxsdk/include/tlibc -I/opt/sgxsdk/include/libcxx -DSQLITE_THREADSAFE=0 -c sqlite3.i -o sqlite3.o

echo "build ocall_interface"

cc -I/opt/sgxsdk/include -E ocall_interface.c -o ocall_interface.i
cc -m64 -O2  -nostdinc -fvisibility=hidden -fpie -ffunction-sections -fdata-sections -fstack-protector-strong -IEnclave -I/opt/sgxsdk/include -I/opt/sgxsdk/include/tlibc -I/opt/sgxsdk/include/libcxx -c ocall_interface.i -o ocall_interface.o
