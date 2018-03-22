use std::env;

use cargo::Subcommand;

pub struct Args {
    all: Vec<String>,
    subcommand: Option<Subcommand>,
    target: Option<String>,
}

impl Args {
    pub fn all(&self) -> &[String] {
        &self.all
    }

    pub fn subcommand(&self) -> Option<Subcommand> {
        self.subcommand
    }

    pub fn target(&self) -> Option<&str> {
        self.target.as_ref().map(|s| &**s)
    }

    pub fn verbose(&self) -> bool {
        self.all
            .iter()
            .any(|a| a == "--verbose" || a == "-v" || a == "-vv")
    }

    pub fn version(&self) -> bool {
        self.all.iter().any(|a| a == "--version" || a == "-V")
    }
}

pub fn args() -> Args {
    let all = env::args().skip(1).collect::<Vec<_>>();

    let mut sc = None;
    let mut target = None;
    {
        let mut args = all.iter();
        while let Some(arg) = args.next() {
            if !arg.starts_with("-") {
                sc = sc.or_else(|| Some(Subcommand::from(&**arg)));
            }

            if arg == "--target" {
                target = args.next().map(|s| s.to_owned());
            } else if arg.starts_with("--target=") {
                target = arg.splitn(2, '=').nth(1).map(|s| s.to_owned());
            }
        }
    }

    Args {
        all: all,
        subcommand: sc,
        target: target,
    }
}
