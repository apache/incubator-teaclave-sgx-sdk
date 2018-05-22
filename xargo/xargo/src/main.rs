#![deny(warnings)]

#[macro_use]
extern crate error_chain;
extern crate fs2;
#[cfg(any(all(target_os = "linux", not(target_env = "musl")), target_os = "macos"))]
extern crate libc;
extern crate rustc_version;
extern crate serde_json;
extern crate tempdir;
extern crate toml;
extern crate walkdir;

use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::{env, io, process};

use rustc_version::Channel;

use errors::*;
use rustc::Target;

mod cargo;
mod cli;
mod errors;
mod extensions;
mod flock;
mod rustc;
mod sysroot;
mod util;
mod xargo;

// We use a different sysroot for Native compilation to avoid file locking
//
// Cross compilation requires `lib/rustlib/$HOST` to match `rustc`'s sysroot,
// whereas Native compilation wants to use a custom `lib/rustlib/$HOST`. If each
// mode has its own sysroot then we avoid sharing that directory and thus file
// locking it.
pub enum CompilationMode {
    Cross(Target),
    Native(String),
}

impl CompilationMode {
    fn hash<H>(&self, hasher: &mut H) -> Result<()>
    where
        H: Hasher,
    {
        match *self {
            CompilationMode::Cross(ref target) => target.hash(hasher)?,
            CompilationMode::Native(ref triple) => triple.hash(hasher),
        }

        Ok(())
    }

    fn triple(&self) -> &str {
        match *self {
            CompilationMode::Cross(ref target) => target.triple(),
            CompilationMode::Native(ref triple) => triple,
        }
    }

    fn is_native(&self) -> bool {
        match *self {
            CompilationMode::Native(_) => true,
            _ => false,
        }
    }
}

pub fn main() {
    fn show_backtrace() -> bool {
        env::var("RUST_BACKTRACE").as_ref().map(|s| &s[..]) == Ok("1")
    }

    match run() {
        Err(e) => {
            let stderr = io::stderr();
            let mut stderr = stderr.lock();

            writeln!(stderr, "error: {}", e).ok();

            for e in e.iter().skip(1) {
                writeln!(stderr, "caused by: {}", e).ok();
            }

            if show_backtrace() {
                if let Some(backtrace) = e.backtrace() {
                    writeln!(stderr, "{:?}", backtrace).ok();
                }
            } else {
                writeln!(stderr, "note: run with `RUST_BACKTRACE=1` for a backtrace").ok();
            }

            process::exit(1)
        }
        Ok(status) => if !status.success() {
            process::exit(status.code().unwrap_or(1))
        },
    }
}

fn run() -> Result<ExitStatus> {
    let args = cli::args();
    let verbose = args.verbose();

    let meta = rustc::version();

    if let Some(sc) = args.subcommand() {
        if !sc.needs_sysroot() {
            return cargo::run(&args, verbose);
        }
    } else if args.version() {
        writeln!(
            io::stderr(),
            concat!("xargo ", env!("CARGO_PKG_VERSION"), "{}"),
            include_str!(concat!(env!("OUT_DIR"), "/commit-info.txt"))
        ).ok();

        return cargo::run(&args, verbose);
    }

    let cd = CurrentDirectory::get()?;

    let config = cargo::config()?;
    if let Some(root) = cargo::root()? {
        // We can't build sysroot with stable or beta due to unstable features
        let sysroot = rustc::sysroot(verbose)?;
        let src = match meta.channel {
            Channel::Dev => rustc::Src::from_env().ok_or(
                "The XARGO_RUST_SRC env variable must be set and point to the \
                 Rust source directory when working with the 'dev' channel",
            )?,
            Channel::Nightly => if let Some(src) = rustc::Src::from_env() {
                src
            } else {
                sysroot.src()?
            },
            Channel::Stable | Channel::Beta => {
                if env::var("RUSTC_BOOTSTRAP").is_ok() {
                    if let Some(src) = rustc::Src::from_env() {
                        src
                    } else {
                        sysroot.src()?
                    }
                } else {
                    writeln!(
                        io::stderr(),
                        "WARNING: the sysroot can't be built for the {:?} channel. \
                        Switch to nightly.",
                        meta.channel
                    ).ok();
                    return cargo::run(&args, verbose);
                }
            }
        };

        let cmode = if let Some(triple) = args.target() {
            if Path::new(triple).is_file() {
                bail!(
                    "Xargo doesn't support files as an argument to --target. \
                     Use `--target foo` instead of `--target foo.json`."
                )
            } else if triple == meta.host {
                Some(CompilationMode::Native(meta.host.clone()))
            } else {
                Target::new(triple, &cd, verbose)?.map(CompilationMode::Cross)
            }
        } else {
            if let Some(ref config) = config {
                if let Some(triple) = config.target()? {
                    Target::new(triple, &cd, verbose)?.map(CompilationMode::Cross)
                } else {
                    Some(CompilationMode::Native(meta.host.clone()))
                }
            } else {
                Some(CompilationMode::Native(meta.host.clone()))
            }
        };

        if let Some(cmode) = cmode {
            let home = xargo::home(&cmode)?;
            let rustflags = cargo::rustflags(config.as_ref(), cmode.triple())?;

            sysroot::update(
                &cmode,
                &home,
                &root,
                &rustflags,
                &meta,
                &src,
                &sysroot,
                verbose,
            )?;
            return xargo::run(
                &args,
                &cmode,
                rustflags,
                &home,
                &meta,
                config.as_ref(),
                verbose,
            );
        }
    }

    cargo::run(&args, verbose)
}

pub struct CurrentDirectory {
    path: PathBuf,
}

impl CurrentDirectory {
    fn get() -> Result<CurrentDirectory> {
        env::current_dir()
            .chain_err(|| "couldn't get the current directory")
            .map(|cd| CurrentDirectory { path: cd })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}
