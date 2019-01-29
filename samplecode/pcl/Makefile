PCL_KEY_SIZE = 16
.PHONY: payload user launcher seal clean user-debug user-release

all: payload user launcher seal
	@echo -n "Encrypt key: " && hexdump -v -e '/1 "%02X"' ./key.bin && echo ""

key.bin:
	dd if=/dev/random of=$@ count=1 bs=$(PCL_KEY_SIZE)

payload: key.bin
	make -C encrypted-hello

user: key.bin user-debug user-release

user-debug: key.bin
	cd pcl-user && cargo b && cd ..

user-release: key.bin
	cd pcl-user && cargo b --release && cd ..

seal: payload
	make -C pcl-seal
	cp encrypted-hello/bin/enclave.signed.so pcl-seal/bin/payload.signed.so

clean:
	rm -f ./key.bin
	make -C encrypted-hello clean
	make -C pcl-seal clean
	rm -f pcl-seal/bin/payload.signed.so
