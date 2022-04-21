# Project Template

Default settings:

- `std-aware-cargo` used by default. Tweak `BUILD_STD ?= cargo` in Makefile to switch between `no_std`, `xargo`, and `std-aware-cargo` mode.
- `sgx_tstd` enables its default feature gate which only contains `stdio`. Tweak `Rust_Std_Features` in Makefile to enable more features. Options are `backtrace`, `stdio`, `env`, `net`, `pipe`, `thread`, `untrusted_fs`, `untrusted_time` `unsupported_process`.
-  StackMaxSize: 0x40000, HeapMaxSize: 0x100000, TCSNum: 1
