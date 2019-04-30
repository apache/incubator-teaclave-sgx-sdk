# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## [v0.3.13] - 2018-12-18

### Fixed

- Xargo now works again with recent nightlies.

### Added

- When the `XARGO_KEEP_TEMP` env variable is set Xargo will keep the temporary
  directory used to build the sysroot. This is useful for debugging problems in
  Xargo.

## [v0.3.12] - 2018-04-08

### Changed

- The `core` and `compiler_builtins` crates are built when no Xargo.toml is present.

## [v0.3.11] - 2018-03-09

### Added

- Xargo now copies the `bin` directory from the original sysroot, the host sysroot, into its own.
  This lets you use binaries shipped with the Rust toolchain, like LLD.

## [v0.3.10] - 2017-12-28

### Added

- Print a warning when the stable or beta toolchain, which are not supported, is used.

### Changed

- Set RUST_TARGET_PATH when building the sysroot. This fixes builds when using custom targets with a
  recent toolchain.

### Removed

- The lock file included in the rust-src component is no longer used when building the sysroot. This
  fixes building a sysroot that contains the compiler-builtins crate.

## [v0.3.9] - 2017-09-06

### Added

- Use Cargo.lock from the `rust-src` component if available. With this change
  the Xargo sysroot will be built using the exact same set of dependencies that
  the official sysroot distributed via rustup uses.

- The `RUSTFLAGS` variable internally used by Xargo is now printed when verbose
  (`-v`) mode is enabled.

### Changed

- Updated the documentation about building `std` with recent nightlies.

## [v0.3.8] - 2017-05-30

### Changed

- Improved the error message when `--target foo.json` is used.

## [v0.3.7] - 2017-05-13

### Changed

- Improved the error message when the `rust-src` component is missing.

## [v0.3.6] - 2017-04-07

### Fixed

- Xargo on Windows. The layout of the default / rustc sysroot recently changed
  on Windows on broke the code that copied the host part of the rustc sysroot
  into the Xargo sysroot.

## [v0.3.5] - 2017-01-20

### Fixed

- Relative paths in `dependencies.{}.path` were not being correctly handled.

## [v0.3.4] - 2017-01-18

### Added

- A `[dependencies.{}.stage]` (or `[target.{}.dependencies.{}.stage]`) entry in
  Xargo.toml. This lets you build a sysroot in "stages". This is required, for
  instance, to build the `test` crate whose dependency on the `std` crate is not
  explicitly listed in its Cargo.toml. Example:

To make `xargo test` work

``` toml
# Xargo.toml
[dependencies.std]
features = ["panic_unwind"]  # `test` depends on this `std` feature
# stage = 0  # implicit

[dependencies.test]
stage = 1
```

- Support for `[dependencies.{}.git]` or `[dependencies.{}.path]` (and their
  `target.{}.dependencies` variants) in Xargo.toml. With this feature you can
  inject foreign crates (crates which are not part of the `rust-src` component)
  into the sysroot. The main use case is replacing the `std` crate with a drop
  in replacement. Example:

Replace `std` with [`steed`](https://github.com/japaric/steed)

``` toml
[dependencies.collections]  # `steed` depends on `collections`

[dependencies.std]
git = "https://github.com/japaric/steed"
stage = 1
```

## [v0.3.3] - 2017-01-09

### Added

- Support for building a custom sysroot when compiling natively.
- Support for specifying dependencies as `[dependencies]` in Xargo.toml.

## [v0.3.2] - 2017-01-03

### Changed

- `XARGO_RUST_SRC` is now used when working with nightly Rust and it has
  precedence over the `rust-src` component.

## [v0.3.1] - 2016-12-30

### Added

- You can now specify the location where Xargo stores the sysroots via the
  `XARGO_HOME` environment variable. If unspecified, the sysroots will be stored
  in `$HOME/.xargo`

## [v0.3.0] - 2016-12-28

### Changed

- [breaking-change] By default, Xargo now only compiles the `core` crate. To
  build more crates, use a `Xargo.toml` file

- [breaking-change] Xargo will now build a sysroot for any target that's not the
  host.

- The verbose flag, `-v`, makes Xargo print all the shell commands it invokes
  to stderr.

## [v0.2.3] - 2016-12-19

### Added

- Support for the 'dev' channel. When using the dev channel, you must specify
  the path to the Rust source directory via the XARGO_RUST_SRC environment
  variable.

### Changed

- The rust-src search logic to account for recent changes in the Rust
  distribution.

## [v0.2.2] - 2016-12-12

### Changed

- Xargo will now try to build every crate "below" `std`, i.e. all its
  dependencies, in topological order. This makes Xargo robust against changes in
  the `std` facade as it no longer depends on hard coded crate names like
  `rustc_unicode`.

- Xargo won't rebuild the sysroot if the only thing that changed in Cargo.toml
  is profile.*.lto. Enabling/disabling LTO doesn't change how dependencies are
  compiled.

- Xargo won't rebuild the sysroot if the linker flags (`-C link-arg`) have
  changed. Those don't affect how the dependencies are compiled.

## [v0.2.1] - 2016-10-22

### Changed

- No weird `()` output in `xargo -V` if Xargo was built via `cargo install`
- Better formatted error messages. Mention the RUST_BACKTRACE env variable which
  is used to get backtraces on errors.

## [v0.2.0] - 2016-10-16

### Added

- Statically linked binary releases for Linux (x86 musl targets)
- `xargo -V` output now includes the commit hash and date

### Changed

- Xargo now depends on the `rust-src` component being installed. Install it with
  `rustup component add rust-src`.
- Xargo no longer depends on libcurl, libssh or libssl and, therefore, it's now
  much easier to build.
- Xargo now respects the existing rustdocflags (RUSTDOCFLAGS env var,
  build.rustdocflags, etc) when passing --sysroot to rustdoc.
- File locking logic has been revised/simplied and now lock periods are shorter

## [v0.1.14] - 2016-10-09

### Added

- `xargo -V` and `xargo --version` now report Xargo's version as well as
  Cargo's.

## [v0.1.13] - 2016-10-06

### Added

- Xargo now builds a sysroot for the new built-in `thumbv*-none-eabi*` targets
  which don't ship with a binary release of the standard crates.

## [v0.1.12] - 2016-10-04

### Added

- Xargo now supports per-target rustflags:
  `target.thumbv7em-none-eabihf.rustflags` in .cargo/config.

## [v0.1.11] - 2016-09-30

### Fixed

- `xargo clean` and other commands not associated to building stuff no longer
  trigger a sysroot rebuild.

## [v0.1.10] - 2016-09-28

### Fixed

- `xargo doc`, which wasn't working because we didn't pass --sysroot to rustdoc.
  Note that rustdoc gained support for '--sysroot' as of nightly-2016-06-28, so
  that version or newer is required to use `xargo doc`.

## [v0.1.9] - 2016-09-27

### Fixed

- "error: Invalid cross-device link (os error 18)" which occurred when
  `$CARGO_HOME` was mounted in a different device than "`$XARGO_HOME`"
  (~/.xargo). The solution was to stop using hard links to place the host
  libraries in the Xargo sysroot and instead just copy them. This is a
  regression in disk usage but this problem was coming up in common Docker usage
  patterns (-v A:B).

## [v0.1.8] - 2016-09-04

### Changed

- All the status messages are now printed to stderr instead of to stdout. Cargo
  did the same change (from stdout to stderr) a while ago. Let's follow suit.

### Fixed

- When compiling crate `foo` with Xargo, the profile section of `foo`'s
  Cargo.toml is also "taken into account" when compiling the sysroot. For
  example, if `foo` has set `panic = "abort"` for all its profiles, then the
  sysroot will also be compiled with `-C panic=abort`. Previously, this wasn't
  the case.

## [v0.1.7] - 2016-09-03

### Fixed

- The sysroot now gets rebuilt when rust-src changes.

## [v0.1.6] - 2016-08-29

### Added

- Xargo can now use the source code installed by rustup. When available, this is
  the preferred way to fetch the source code and saves network bandwidth by not
  having to fetch the source tarball.

## [v0.1.5] - 2016-08-11

### Fixed

- Xargo now works properly when called from a `rustup override`n directory.

## [v0.1.4] - 2016-08-06

### Added

- Support targets that don't support atomics (`"max-atomic-width": 0`). For
  these targets, Xargo only compiles the `core` and `rustc_unicode` crates as
  the other crates depend on atomics (e.g. `alloc::Arc`).

## [v0.1.3] - 2016-04-24

### Added

- `xargo (..) --verbose` passes `--verbose` to the `cargo` call that builds the
  sysroot.
- the sysroot now gets rebuilt when RUSTFLAGS or build.rustflags is modified.

### Fixed

- Xargo now respects the build.rustflags value set in .cargo/config.
- A bug where the hash/date file didn't get properly truncated before updating
  it leading to Xargo to *always* trigger a sysroot rebuild.

## [v0.1.2] - 2016-04-24 [YANKED]

### Added

- Xargo now uses file locking and can be executed concurrently.
- Xargo now print its current status to the console while building a sysroot.
- Xargo now reports errors to the console instead of panicking.

### Removed

- Logging via `RUST_LOG` has been removed now that Xargo prints its status to
  the console.

## v0.1.1 - 2016-04-10

- Initial release

[Unreleased]: https://github.com/japaric/xargo/compare/v0.3.13...HEAD
[v0.3.13]: https://github.com/japaric/xargo/compare/v0.3.12...v0.3.13
[v0.3.12]: https://github.com/japaric/xargo/compare/v0.3.11...v0.3.12
[v0.3.11]: https://github.com/japaric/xargo/compare/v0.3.10...v0.3.11
[v0.3.10]: https://github.com/japaric/xargo/compare/v0.3.9...v0.3.10
[v0.3.9]: https://github.com/japaric/xargo/compare/v0.3.8...v0.3.9
[v0.3.8]: https://github.com/japaric/xargo/compare/v0.3.7...v0.3.8
[v0.3.7]: https://github.com/japaric/xargo/compare/v0.3.6...v0.3.7
[v0.3.6]: https://github.com/japaric/xargo/compare/v0.3.5...v0.3.6
[v0.3.5]: https://github.com/japaric/xargo/compare/v0.3.4...v0.3.5
[v0.3.4]: https://github.com/japaric/xargo/compare/v0.3.3...v0.3.4
[v0.3.3]: https://github.com/japaric/xargo/compare/v0.3.2...v0.3.3
[v0.3.2]: https://github.com/japaric/xargo/compare/v0.3.1...v0.3.2
[v0.3.1]: https://github.com/japaric/xargo/compare/v0.3.0...v0.3.1
[v0.3.0]: https://github.com/japaric/xargo/compare/v0.2.3...v0.3.0
[v0.2.3]: https://github.com/japaric/xargo/compare/v0.2.2...v0.2.3
[v0.2.2]: https://github.com/japaric/xargo/compare/v0.2.1...v0.2.2
[v0.2.1]: https://github.com/japaric/xargo/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/japaric/xargo/compare/v0.1.14...v0.2.0
[v0.1.14]: https://github.com/japaric/xargo/compare/v0.1.13...v0.1.14
[v0.1.13]: https://github.com/japaric/xargo/compare/v0.1.12...v0.1.13
[v0.1.12]: https://github.com/japaric/xargo/compare/v0.1.11...v0.1.12
[v0.1.11]: https://github.com/japaric/xargo/compare/v0.1.10...v0.1.11
[v0.1.10]: https://github.com/japaric/xargo/compare/v0.1.9...v0.1.10
[v0.1.9]: https://github.com/japaric/xargo/compare/v0.1.8...v0.1.9
[v0.1.8]: https://github.com/japaric/xargo/compare/v0.1.7...v0.1.8
[v0.1.7]: https://github.com/japaric/xargo/compare/v0.1.6...v0.1.7
[v0.1.6]: https://github.com/japaric/xargo/compare/v0.1.5...v0.1.6
[v0.1.5]: https://github.com/japaric/xargo/compare/v0.1.4...v0.1.5
[v0.1.4]: https://github.com/japaric/xargo/compare/v0.1.3...v0.1.4
[v0.1.3]: https://github.com/japaric/xargo/compare/v0.1.2...v0.1.3
[v0.1.2]: https://github.com/japaric/xargo/compare/v0.1.1...v0.1.2
