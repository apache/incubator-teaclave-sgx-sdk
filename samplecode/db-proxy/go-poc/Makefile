default: build

install:
	go get github.com/bradyjoestar/merkle-btree-test
	go get github.com/syndtr/goleveldb/leveldb


build:
	rm -rf db
	go build -o bin/db-client db-client/main.go
	go build -o bin/db-proxy-server db-proxy-server/main.go


