This is simple proof of concept for db-proxy.[VeritasDB: High Throughput Key-Value Store with Integrity using SGX](https://eprint.iacr.org/2018/251.pdf).

We are concerd about the following things in the paper:
- Basic Design of db-proxy
- Merkle B-Tree and Operations
- Persistence and Fault Tolerance

### build
we need some go dependency so run
```bash
make install
```
before run server and client.


### run 

```bash
make build
```

start server
```bash
cd bin
./db-proxy-server
```

start client
```bash
cd bin
./db-client -mode=start
```

###  test persist
close server.

restart server
```bash
./db-proxy-server
```

restart client
```bash
./db-client -mode=reload
```