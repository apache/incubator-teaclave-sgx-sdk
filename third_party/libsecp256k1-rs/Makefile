.PHONY: gen

gen:
	cd gen/ecmult && cargo run > ../../src/ecmult/const.rs.new
	mv src/ecmult/const.rs.new src/ecmult/const.rs
	cd gen/genmult && cargo run > ../../src/ecmult/const_gen.rs.new
	mv src/ecmult/const_gen.rs.new src/ecmult/const_gen.rs
