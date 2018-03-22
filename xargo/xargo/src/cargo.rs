use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::{env, fmt};

use toml::Value;

use cli::Args;
use errors::*;
use extensions::CommandExt;
use util;
use xargo::Home;

pub struct Rustflags {
    flags: Vec<String>,
}

impl Rustflags {
    pub fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        let mut flags = self.flags.iter();

        while let Some(flag) = flags.next() {
            if flag == "-C" {
                if let Some(next) = flags.next() {
                    if next.starts_with("link-arg=") || next.starts_with("link-args=") {
                        // don't hash linker arguments
                    } else {
                        flag.hash(hasher);
                        next.hash(hasher);
                    }
                } else {
                    flag.hash(hasher);
                }
            } else {
                flag.hash(hasher);
            }
        }
    }

    /// Stringifies these flags for Xargo consumption
    pub fn for_xargo(&self, home: &Home) -> String {
        let mut flags = self.flags.clone();
        flags.push("--sysroot".to_owned());
        flags.push(home.display().to_string());
        flags.join(" ")
    }
}

impl fmt::Display for Rustflags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.flags.join(" "), f)
    }
}

pub fn rustflags(config: Option<&Config>, target: &str) -> Result<Rustflags> {
    flags(config, target, "rustflags").map(|fs| Rustflags { flags: fs })
}

pub struct Rustdocflags {
    flags: Vec<String>,
}

impl Rustdocflags {
    /// Stringifies these flags for Xargo consumption
    pub fn for_xargo(mut self, home: &Home) -> String {
        self.flags.push("--sysroot".to_owned());
        self.flags.push(home.display().to_string());
        self.flags.join(" ")
    }
}

pub fn rustdocflags(config: Option<&Config>, target: &str) -> Result<Rustdocflags> {
    flags(config, target, "rustdocflags").map(|fs| Rustdocflags { flags: fs })
}


/// Returns the flags for `tool` (e.g. rustflags)
///
/// This looks into the environment and into `.cargo/config`
fn flags(config: Option<&Config>, target: &str, tool: &str) -> Result<Vec<String>> {
    if let Some(t) = env::var_os(tool.to_uppercase()) {
        return Ok(
            t.to_string_lossy()
                .split_whitespace()
                .map(|w| w.to_owned())
                .collect(),
        );
    }

    if let Some(config) = config.as_ref() {
        let mut build = false;
        if let Some(array) = config
            .table
            .lookup(&format!("target.{}.{}", target, tool))
            .or_else(|| {
                build = true;
                config.table.lookup(&format!("build.{}", tool))
            }) {
            let mut flags = vec![];

            let mut error = false;
            if let Some(array) = array.as_slice() {
                for value in array {
                    if let Some(flag) = value.as_str() {
                        flags.push(flag.to_owned());
                    } else {
                        error = true;
                        break;
                    }
                }
            } else {
                error = true;
            }

            if error {
                if build {
                    Err(format!(
                        ".cargo/config: build.{} must be an array \
                         of strings",
                        tool
                    ))?
                } else {
                    Err(format!(
                        ".cargo/config: target.{}.{} must be an \
                         array of strings",
                        target,
                        tool
                    ))?
                }
            } else {
                Ok(flags)
            }
        } else {
            Ok(vec![])
        }
    } else {
        Ok(vec![])
    }
}

pub fn run(args: &Args, verbose: bool) -> Result<ExitStatus> {
    Command::new("cargo")
        .args(args.all())
        .run_and_get_status(verbose)
}

pub struct Config {
    table: Value,
}

impl Config {
    pub fn target(&self) -> Result<Option<&str>> {
        if let Some(v) = self.table.lookup("build.target") {
            Ok(Some(v.as_str()
                .ok_or_else(|| format!(".cargo/config: build.target must be a string"))?))
        } else {
            Ok(None)
        }
    }
}

pub fn config() -> Result<Option<Config>> {
    let cd = env::current_dir().chain_err(|| "couldn't get the current directory")?;

    if let Some(p) = util::search(&cd, ".cargo/config") {
        Ok(Some(Config {
            table: util::parse(&p.join(".cargo/config"))?,
        }))
    } else {
        Ok(None)
    }
}

pub struct Profile<'t> {
    table: &'t Value,
}

impl<'t> Profile<'t> {
    pub fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        let mut v = self.table.clone();

        // Don't include `lto` in the hash because it doesn't affect compilation
        // of `.rlib`s
        if let Value::Table(ref mut table) = v {
            table.remove("lto");

            // don't hash an empty map
            if table.is_empty() {
                return;
            }
        }

        v.to_string().hash(hasher);
    }
}

impl<'t> fmt::Display for Profile<'t> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut map = BTreeMap::new();
        map.insert("profile".to_owned(), {
            let mut map = BTreeMap::new();
            map.insert("release".to_owned(), self.table.clone());
            Value::Table(map)
        });

        fmt::Display::fmt(&Value::Table(map), f)
    }
}

pub struct Toml {
    table: Value,
}

impl Toml {
    /// `profile.release` part of `Cargo.toml`
    pub fn profile(&self) -> Option<Profile> {
        self.table
            .lookup("profile.release")
            .map(|t| Profile { table: t })
    }
}

pub fn toml(root: &Root) -> Result<Toml> {
    util::parse(&root.path().join("Cargo.toml")).map(|t| Toml { table: t })
}

pub struct Root {
    path: PathBuf,
}

impl Root {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

pub fn root() -> Result<Option<Root>> {
    let cd = env::current_dir().chain_err(|| "couldn't get the current directory")?;

    Ok(util::search(&cd, "Cargo.toml").map(|p| Root { path: p.to_owned() }))
}

#[derive(Clone, Copy, PartialEq)]
pub enum Subcommand {
    Clean,
    Doc,
    Init,
    New,
    Other,
    Search,
    Update,
}

impl Subcommand {
    pub fn needs_sysroot(&self) -> bool {
        use self::Subcommand::*;

        match *self {
            Clean | Init | New | Search | Update => false,
            _ => true,
        }
    }
}

impl<'a> From<&'a str> for Subcommand {
    fn from(s: &str) -> Subcommand {
        match s {
            "clean" => Subcommand::Clean,
            "doc" => Subcommand::Doc,
            "init" => Subcommand::Init,
            "new" => Subcommand::New,
            "search" => Subcommand::Search,
            "update" => Subcommand::Update,
            _ => Subcommand::Other,
        }
    }
}
