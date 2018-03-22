use std::env;
use std::ffi::OsStr;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

pub use rustc_version::version_meta as version;

use serde_json::Value;
use serde_json;
use walkdir::WalkDir;

use CurrentDirectory;
use errors::*;
use extensions::CommandExt;
use {rustc, util};

fn command() -> Command {
    env::var_os("RUSTC")
        .map(Command::new)
        .unwrap_or_else(|| Command::new("rustc"))
}

/// `rustc --print target-list`
pub fn targets(verbose: bool) -> Result<Vec<String>> {
    command()
        .args(&["--print", "target-list"])
        .run_and_get_stdout(verbose)
        .map(|t| t.lines().map(|l| l.to_owned()).collect())
}

/// `rustc --print sysroot`
pub fn sysroot(verbose: bool) -> Result<Sysroot> {
    command()
        .args(&["--print", "sysroot"])
        .run_and_get_stdout(verbose)
        .map(|l| {
            Sysroot {
                path: PathBuf::from(l.trim()),
            }
        })
}
/// Path to Rust source
pub struct Src {
    path: PathBuf,
}

impl Src {
    pub fn from_env() -> Option<Self> {
        env::var_os("XARGO_RUST_SRC").map(|s| {
            Src {
                path: PathBuf::from(s),
            }
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Path to `rustc`'s sysroot
pub struct Sysroot {
    path: PathBuf,
}

impl Sysroot {
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the path to Rust source, `$SRC`, where `$SRC/libstd/Carg.toml`
    /// exists
    pub fn src(&self) -> Result<Src> {
        let src = self.path().join("lib/rustlib/src");

        if src.join("rust/src/libstd/Cargo.toml").is_file() {
            return Ok(Src {
                path: src.join("rust/src"),
            });
        }

        if src.exists() {
            for e in WalkDir::new(src) {
                let e = e.chain_err(|| "couldn't walk the sysroot")?;

                // Looking for $SRC/libstd/Cargo.toml
                if e.file_type().is_file() && e.file_name() == "Cargo.toml" {
                    let toml = e.path();

                    if let Some(std) = toml.parent() {
                        if let Some(src) = std.parent() {
                            if std.file_name() == Some(OsStr::new("libstd")) {
                                return Ok(Src {
                                    path: src.to_owned(),
                                });
                            }
                        }
                    }
                }
            }
        }

        Err(
            "`rust-src` component not found. Run `rustup component add \
             rust-src`.",
        )?
    }
}

#[derive(Debug)]
pub enum Target {
    Builtin { triple: String },
    Custom { json: PathBuf, triple: String },
}

impl Target {
    pub fn new(triple: &str, cd: &CurrentDirectory, verbose: bool) -> Result<Option<Target>> {
        let triple = triple.to_owned();

        if rustc::targets(verbose)?.iter().any(|t| t == &triple) {
            Ok(Some(Target::Builtin { triple: triple }))
        } else {
            let mut json = cd.path().join(&triple);
            json.set_extension("json");

            if json.exists() {
                return Ok(Some(Target::Custom {
                    json: json,
                    triple: triple,
                }));
            } else {
                if let Some(p) = env::var_os("RUST_TARGET_PATH") {
                    let mut json = PathBuf::from(p);
                    json.push(&triple);
                    json.set_extension("json");

                    if json.exists() {
                        return Ok(Some(Target::Custom {
                            json: json,
                            triple: triple,
                        }));
                    }
                }
            }

            Ok(None)
        }
    }

    pub fn triple(&self) -> &str {
        match *self {
            Target::Builtin { ref triple } => triple,
            Target::Custom { ref triple, .. } => triple,
        }
    }

    pub fn hash<H>(&self, hasher: &mut H) -> Result<()>
    where
        H: Hasher,
    {
        if let Target::Custom { ref json, .. } = *self {
            // Here we roundtrip to/from JSON to get the same hash when some
            // fields of the JSON file has been shuffled around
            serde_json::from_str::<Value>(&util::read(json)?)
                .chain_err(|| format!("{} is not valid JSON", json.display()))?
                .to_string()
                .hash(hasher);
        }

        Ok(())
    }
}
