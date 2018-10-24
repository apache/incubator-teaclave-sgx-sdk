![fancy garbling logo](logo.png)

# fancy-garbling
Implementation of the [BMR16](https://eprint.iacr.org/2016/969) arithmetic garbling scheme.

# compiling
Requires a recentish version of Rust

* `cargo test` run the tests
* `cargo bench` run the benchmarks

We include an optimization that speeds up base conversion 10x. To enable this, you must
generate base conversion truth tables by invoking `./scripts/make_lookup_tables.py` This
overwrites the stub C source file `base_conversion/cbits/lookup_tables.c`.
