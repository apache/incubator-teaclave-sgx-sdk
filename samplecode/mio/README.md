## Run

Start server

```
cd server
make
cd bin
./app (add '--maxconn 32' if you want to set the max_conn of tlsserver to 32)
```

Start client 

```
cd client
cargo run
```

Start client-go (golang should be installed)
```
cd client-go
make
./bin/app
```

Start client-java (Java:1.8+, mvn)
```
cd client-java
mvn install
java -jar target/client-java-0.0.1-SNAPSHOT.jar
```
