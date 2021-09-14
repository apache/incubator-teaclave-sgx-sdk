---
permalink: /sgx-sdk-docs/performance-optimization-tips
---
# Performance Optimization Tips

## Enable link time optimization

This could boost CPU-intensive enclaves about 2~5% (on my 9900K)

In `Cargo.toml`:
```toml
[profile.release]
lto = true
```

## Let rustc emit asm

This could boost enclaves on some platforms.

Set an environment variable as

```bash
export RUSTFLAGS="--emit asm"
```

Or create a `.cargo/config` which covers your project as:

```toml
[build]
rustflags = ["--emit","asm"]
```

##  Configure target_cpu for llvm

This could boost enclaves on some platforms (not effective on my 9900K).

Set an environment variable as

```bash
export RUSTFLAGS="-C target-cpu=native"
```

Or create a `.cargo/config` which covers your project as:

```toml
[build]
rustflags = ["-C", "target-cpu=native"]
```

## Enable lto on the final linking step

Add `-flto` to the final linking step using `CXX`.
