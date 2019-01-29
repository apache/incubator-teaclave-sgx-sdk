# [PSA: Xargo is in maintenance mode](https://github.com/japaric/xargo/issues/193)

[![crates.io](https://img.shields.io/crates/v/xargo.svg)](https://crates.io/crates/xargo)
[![crates.io](https://img.shields.io/crates/d/xargo.svg)](https://crates.io/crates/xargo)

# `xargo`

> The sysroot manager that lets you build and customize `std`

<p align="center">
<img
  alt="Cross compiling `std` for i686-unknown-linux-gnu"
  src="assets/xargo.png"
  title="Cross compiling `std` for i686-unknown-linux-gnu"
>
<br>
<em>Cross compiling `std` for i686-unknown-linux-gnu</em>
</p>

Xargo builds and manages "sysroots" (cf. `rustc --print sysroot`). Making it
easy to cross compile Rust crates for targets that *don't* have binary
releases of the standard crates, like the `thumbv*m-none-eabi*` targets. And
it also lets you build a customized `std` crate, e.g. compiled with `-C
panic=abort`, for your target.

## Dependencies

- The `rust-src` component, which you can install with `rustup component add
  rust-src`.

- Rust and Cargo.

## Installation

```
$ cargo install xargo
```

But we also have [binary releases] for the three major OSes.

[binary releases]: https://github.com/japaric/xargo/releases

## Usage

### `no_std`

`xargo` has the exact same CLI as `cargo`.

```
# This Just Works
$ xargo build --target thumbv6m-none-eabi
   Compiling core v0.0.0 (file://$SYSROOT/lib/rustlib/src/rust/src/libcore)
    Finished release [optimized] target(s) in 11.61 secs
   Compiling lib v0.1.0 (file://$PWD)
    Finished debug [unoptimized + debuginfo] target(s) in 0.5 secs
```

`xargo` will cache the sysroot, in this case the `core` crate, so the next
`build` command will be (very) fast.

```
$ xargo build --target thumbv6m-none-eabi
    Finished debug [unoptimized + debuginfo] target(s) in 0.0 secs
```

By default, `xargo` will only compile the `core` crate for the target. If you
need a bigger subset of the standard crates, specify the dependencies in a
`Xargo.toml` at the root of your Cargo project (right next to `Cargo.toml`).

```
$ cat Xargo.toml
# Alternatively you can use [build.dependencies]
# the syntax is the same as Cargo.toml's; you don't need to specify path or git
[target.thumbv6m-none-eabi.dependencies]
collections = {}

$ xargo build --target thumbv6m-none-eabi
   Compiling core v0.0.0 (file://$SYSROOT/lib/rustlib/src/rust/src/libcore)
   Compiling alloc v0.0.0 (file://$SYSROOT/lib/rustlib/src/rust/src/liballoc)
   Compiling std_unicode v0.0.0 (file://$SYSROOT/lib/rustlib/src/rust/src/libstd_unicode)
   Compiling collections v0.0.0 (file://$SYSROOT/lib/rustlib/src/rust/src/libcollections)
    Finished release [optimized] target(s) in 15.26 secs
   Compiling lib v0.1.0 (file://$PWD)
    Finished debug [unoptimized + debuginfo] target(s) in 0.5 secs
```

### `std`

You can compile a customized `std` crate as well, just specify which Cargo
features to enable.

```
# Build `std` with `-C panic=abort` (default) and with jemalloc as the default
# allocator
$ cat Xargo.toml
[target.i686-unknown-linux-gnu.dependencies.std]
features = ["jemalloc"]

# Needed to compile `std` with `-C panic=abort`
$ tail -n2 Cargo.toml
[profile.release]
panic = "abort"

$ xargo run --target i686-unknown-linux-gnu --release
    Updating registry `https://github.com/rust-lang/crates.io-index`
   Compiling libc v0.0.0 (file://$SYSROOT/lib/rustlib/src/rust/src/rustc/libc_shim)
   Compiling core v0.0.0 (file://$SYSROOT/lib/rustlib/src/rust/src/libcore)
   Compiling build_helper v0.1.0 (file://$SYSROOT/lib/rustlib/src/rust/src/build_helper)
   Compiling gcc v0.3.41
   Compiling unwind v0.0.0 (file://$SYSROOT/lib/rustlib/src/rust/src/libunwind)
   Compiling std v0.0.0 (file://$SYSROOT/lib/rustlib/src/rust/src/libstd)
   Compiling compiler_builtins v0.0.0 (file://$SYSROOT/lib/rustlib/src/rust/src/libcompiler_builtins)
   Compiling alloc_jemalloc v0.0.0 (file://$SYSROOT/lib/rustlib/src/rust/src/liballoc_jemalloc)
   Compiling alloc v0.0.0 (file://$SYSROOT/lib/rustlib/src/rust/src/liballoc)
   Compiling rand v0.0.0 (file://$SYSROOT/lib/rustlib/src/rust/src/librand)
   Compiling std_unicode v0.0.0 (file://$SYSROOT/lib/rustlib/src/rust/src/libstd_unicode)
   Compiling alloc_system v0.0.0 (file://$SYSROOT/lib/rustlib/src/rust/src/liballoc_system)
   Compiling panic_abort v0.0.0 (file://$SYSROOT/lib/rustlib/src/rust/src/libpanic_abort)
   Compiling collections v0.0.0 (file://$SYSROOT/lib/rustlib/src/rust/src/libcollections)
    Finished release [optimized] target(s) in 33.49 secs
   Compiling hello v0.1.0 (file://$PWD)
    Finished release [optimized] target(s) in 0.28 secs
     Running `target/i686-unknown-linux-gnu/release/hello`
Hello, world!
```

If you'd like to know what `xargo` is doing under the hood, pass the verbose,
`-v`, flag to it.

```
$ xargo build --target thumbv6m-none-eabi -v
+ "rustc" "--print" "target-list"
+ "rustc" "--print" "sysroot"
+ "cargo" "build" "--release" "--manifest-path" "/tmp/xargo.lTBXKnaUGicV/Cargo.toml" "--target" "thumbv6m-none-eabi" "-v" "-p" "core"
   Compiling core v0.0.0 (file://$SYSROOT/lib/rustlib/src/rust/src/libcore)
     Running `rustc --crate-name core $SYSROOT/lib/rustlib/src/rust/src/libcore/lib.rs --crate-type lib -C opt-level=3 -C metadata=a5c596f87f7d486b -C extra-filename=-a5c596f87f7d486b --out-dir /tmp/xargo.lTBXKnaUGicV/target/thumbv6m-none-eabi/release/deps --emit=dep-info,link --target thumbv6m-none-eabi -L dependency=/tmp/xargo.lTBXKnaUGicV/target/thumbv6m-none-eabi/release/deps -L dependency=/tmp/xargo.lTBXKnaUGicV/target/release/deps`
    Finished release [optimized] target(s) in 11.50 secs
+ "cargo" "build" "--target" "thumbv6m-none-eabi" "-v"
   Compiling lib v0.1.0 (file://$PWD)
     Running `rustc --crate-name lib src/lib.rs --crate-type lib -g -C metadata=461fd0b398821543 -C extra-filename=-461fd0b398821543 --out-dir $PWD/target/thumbv6m-none-eabi/debug/deps --emit=dep-info,link --target thumbv6m-none-eabi -L dependency=$PWD/target/thumbv6m-none-eabi/debug/deps -L dependency=$PWD/lib/target/debug/deps --sysroot $HOME/.xargo`
    Finished debug [unoptimized + debuginfo] target(s) in 0.5 secs
```

### Dev channel

Oh, and if you want to use `xargo` to compile `std` using a "dev" `rustc`, a
rust compiled from source, you can use the `XARGO_RUST_SRC` environment variable
to tell `xargo` where the Rust source is.

```
# The source of the `core` crate must be in `$XARGO_RUST_SRC/libcore`
$ export XARGO_RUST_SRC=/path/to/rust/src

$ xargo build --target msp430-none-elf
```

**NOTE** This also works with the nightly channel but it's not recommended as
the Rust source may diverge from what your compiler is able to compile as it may
make use of newer features that your compiler doesn't understand.

### Compiling the sysroot with custom rustc flags

Xargo uses the same custom rustc flags that apply to the target Cargo project.
So you can use either the `RUSTFLAGS` env variable or a `.cargo/config`
configuration file to specify custom rustc flags.

```
# build the sysroot with debug information
$ RUSTFLAGS='-g' xargo build --target x86_64-unknown-linux-gnu

# Alternatively
$ edit .cargo/config && cat $_
[build]
rustflags = ["-g"]

# Then you can omit RUSTFLAGS
$ xargo build --target x86_64-unknown-linux-gnu
```

### Compiling the sysroot for a custom target

At some point you may want to develop a program for a target that's not
officially supported by rustc. Xargo's got your back! It supports custom targets
via target specifications files, which are not really documented anywhere other
than in the [compiler source code][spec-docs]. Luckily you don't need to write
a specification file from scratch; you can start from an existing one.

[spec-docs]: https://github.com/rust-lang/rust/blob/256e497fe63bf4b13f7c0b58fa17360ca849c54d/src/librustc_back/target/mod.rs#L228-L409

For example, let's say that you want to cross compile a program for a PowerPC
Linux systems that uses uclibc instead of glibc. There's a similarly looking
target in the list of targets supported by the compiler -- see `rustc --print
target-list` -- and that is `powerpc-unknown-linux-gnu`. So you can start by
dumping the specification of that target into a file:

``` console
$ rustc -Z unstable-options --print target-spec-json --target powerpc-unknown-linux-gnu | tee powerpc-unknown-linux-uclibc.json
```

``` js
{
  "arch": "powerpc",
  "data-layout": "E-m:e-p:32:32-i64:64-n32",
  "dynamic-linking": true,
  "env": "gnu",
  "executables": true,
  "has-elf-tls": true,
  "has-rpath": true,
  "is-builtin": true,
  "linker-flavor": "gcc",
  "linker-is-gnu": true,
  "llvm-target": "powerpc-unknown-linux-gnu",
  "max-atomic-width": 32,
  "os": "linux",
  "position-independent-executables": true,
  "pre-link-args": {
    "gcc": [
      "-Wl,--as-needed",
      "-Wl,-z,noexecstack",
      "-m32"
    ]
  },
  "target-endian": "big",
  "target-family": "unix",
  "target-pointer-width": "32",
  "vendor": "mesalock"
}
```

One of the things you'll definitively want to do is drop the `is-builtin` field
as that's reserved for targets that are defined in the compiler itself. Apart
from that the only modification you would have to in this case is change the
`env` field from `gnu` (glibc) to `uclibc`.

``` diff
   "arch": "powerpc",
   "data-layout": "E-m:e-p:32:32-i64:64-n32",
   "dynamic-linking": true,
-  "env": "gnu",
+  "env": "uclibc",
   "executables": true,
   "has-elf-tls": true,
   "has-rpath": true,
-  "is-builtin": true,
   "linker-flavor": "gcc",
   "linker-is-gnu": true,
   "llvm-target": "powerpc-unknown-linux-gnu",
```

Once you have your target specification file you only have to call Xargo with
the right target triple; make sure that the specification file is the same
folder from where you invoke Xargo because that's where rustc expects it to be.

``` console
$ ls powerpc-unknown-linux-uclibc.json
powerpc-unknown-linux-uclibc.json

$ xargo build --target powerpc-unknown-linux-uclibc
```

Your build may fail because if rustc doesn't support your target then it's
likely that the standard library doesn't support it either. In that case you
will have to modify the source of the standard library. Xargo helps with that
too because you can make a copy of the original source -- see `rustc --print
sysroot`, modify it and then point Xargo to it using the `XARGO_RUST_SRC` env
variable.

### Multi-stage builds

Some standard crates have implicit dependencies between them. For example, the
`test` crate implicitly depends on the `std`. Implicit here means that the test
crate Cargo.toml [doesn't list std as its dependency][test]. To compile a
sysroot that contains such crates you can perform the build in stages by
specifying which crates belong to each stage in the Xargo.toml file:

[test]: https://github.com/rust-lang/rust/blob/1.17.0/src/libtest/Cargo.toml

``` toml
[dependencies.std]
stage = 0

[dependencies.test]
stage = 1
```

This will compile an intermediate sysroot, the stage 0 sysroot, containing the
`std` crate, and then it will compile the `test` crate against that intermediate
sysroot. The final sysroot, the stage 1 sysroot, will contain both the `std` and
`test` crates, and their dependencies.

### Creating a sysroot with custom crates

Xargo lets you create a sysroot with custom crates. You can virtually put any
crate in the sysroot. However, this feature is mainly used to create [alternative
`std` facades][steed], and to replace the `test` crate with [one that supports
`no_std` targets][utest]. To specify the contents of the sysroot simply list the
dependencies in the Xargo.toml file as you would do with Cargo.toml:

[steed]: https://github.com/japaric/steed
[utest]: https://github.com/japaric/utest

``` toml
[dependencies]
collections = {}
rand = {}

[dependencies.compiler_builtins]
features = ["mem"]
stage = 1

[dependencies.std]
git = "https://github.com/japaric/steed"
stage = 2
```

## Caveats / gotchas

- Xargo won't build a sysroot when used with stable or beta Rust. This is
  because `std` and other standard crates depend on unstable features so it's
  not possible to build the sysroot with stable or beta.

- `std` is built as rlib *and* dylib. The dylib needs a panic library and an
  allocator.  If you do not specify the `panic-unwind` feature, you have to set
  `panic = "abort"` in `Cargo.toml`.

- To build without the `jemalloc` feature include the following in `Xargo.toml`:

  ``` toml
  [dependencies.std]
  features = ["force_alloc_system"]
  ```

  What this flag means is that every program compiled with this libstd can only use the system
  allocator. If your program tries to set its own allocator, compilation will fail because now two
  allocators are set (one by libstd, one by your program). For some further information on this
  issue, see
  [rust-lang/rust#43637](https://github.com/rust-lang/rust/issues/43637#issuecomment-320463578).

- It's recommended that the `--target` option is always used for `xargo`. This is because it must
  be provided even when compiling for the host platform due to the way cargo handles compiler
  plugins (e.g. `serde_derive`) and build scripts (`build.rs`). This also applies to how all of the
  dependant crates get compiled that use compiler plugins or build scripts. You can determine your
  host's target triple with `rustc -vV`. On *nix, the following rune will extract the triple:
  `rustc -vV | egrep '^host: ' | sed 's/^host: //'`.

- Remember that both `core` and `std` will get implicitly linked to your crate but *all the other
  sysroot crates* will *not*. This means that if your Xargo.toml contains a crate like
  `compiler_builtins` or `alloc` then you will have to add a `extern crate compiler_builtins` or
  `extern crate alloc` *somewhere* in your dependency graph (either in your current crate or in some
  of its dependencies).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
