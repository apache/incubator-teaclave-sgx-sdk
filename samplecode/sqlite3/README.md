# Welcome to RUST-SGX-SQLITE SampleCode!

Hi! This is a port of **Sqlite3** to **Rust-SGX-SDK**. This port reuses a lot ofcode and contents in https://github.com/yerzhan7/SGX_SQLite. Special thanks to @yerzhan7 for open sourcing his/her work! Since this port borrows a lot of content from SGX_SQLite, so they shares a lot of common properties (This port literally does all the things SGX_SQLite does!). I cite @yerzhan7's intro in the following.

> SQLite database inside a secure Intel SGX enclave (Linux).

> You can execute your SQL statements securely. The official SQLite library ("sqlite.c", "sqlite.h") is entirely loaded into an enclave. However, if you want to save your database (i.e. not in-memory database) it will save *.db file without any encryption yet. (need to implement data sealing in the future)

> SQLite Version - 3.19.3

> The project started from "SampleEnclave" provided by Intel SGX SDK. Later, I've added official SQLite library, and then redirected all system calls to ocalls (untrusted part). You can track the development from scratch by viewing all commits.

> This project may act as guide on how to port C applications inside Intel SGX enclave on Linux.`

In addition, this port may also act as a guide on how to port C applications to Rust SGX SDK on Linux :D


## Docker

I use the docker image baiduxlab/sgx-rust.

You can 

> docker pull baiduxlab/sgx-rust

to pull it and can

> docker run -v /your/path/to/rust-sgx:/root/sgx -ti --device /dev/isgx baiduxlab/sgx-rust

to start it and can

> docker ps

to find it and can

> docker exec -it <your_docker_container_id> /bin/bash

to get an interactive bash shell to it!

## Test

Simply make & cd bin. And ./app test.db (or any name you want!). You are good to go!

I did the following test :)

> CREATE TABLE TEST(ID INT PRIMARY KEY NOT NULL, AGE INT NOT NULL);
> INSERT INTO TEST (ID, AGE) VALUES (123,12);
> select ID from TEST;

You shall see ID = 123 returned by console!
Then when you are bored, type in

> quit

Then the database and enclave would be closed! Easy!

