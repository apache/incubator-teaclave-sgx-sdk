Rust_Enclave_Name := libraenclave.a
Rust_Enclave_Files := $(wildcard src/*.rs)

.PHONY: all clean

all: $(Rust_Enclave_Name)

$(Rust_Enclave_Name): $(Rust_Enclave_Files)
	cargo build --release

clean:
	cargo clean && rm -f Cargo.lock
