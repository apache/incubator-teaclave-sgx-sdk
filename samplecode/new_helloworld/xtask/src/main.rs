use clap;
use clap::Parser;
use once_cell::sync::Lazy;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

static RUST_TARGET_PATH: Lazy<String> = Lazy::new(|| {
    let root = std::env::var("ROOT_DIR").unwrap();
    format!("{root}/rustlib")
});
static RUST_BUILD_TARGET: Lazy<String> = Lazy::new(|| "x86_64-unknown-linux-sgx".to_string());
static RUST_BUILD_STD: Lazy<String> = Lazy::new(|| format!("-Zbuild-std=core,alloc"));
static RUST_TARGET_FLAGS: Lazy<String> = Lazy::new(|| {
    format!(
        "--target {}/{}.json",
        RUST_TARGET_PATH.as_str(),
        RUST_BUILD_TARGET.as_str()
    )
});
static RUST_STD_FEATURES: Lazy<String> = Lazy::new(|| format!(""));
static RUST_SYSROOT_PATH: Lazy<String> =
    Lazy::new(|| format!("{}/sysroot", std::env::var("CURDIR").unwrap()));
static RUST_SYSROOT_FLAGS: Lazy<String> =
    Lazy::new(|| format!("RUSTFLAGS=\"--sysroot {}\"", RUST_SYSROOT_PATH.as_str()));

trait CommandDisplay {
    fn display(&mut self, s: &mut String) -> &mut Self;
}

impl CommandDisplay for Command {
    fn display(&mut self, s: &mut String) -> &mut Self {
        std::mem::replace(s, format_command(self));
        self
    }
}

fn format_command(cmd: &Command) -> String {
    let mut s = cmd.get_program().to_str().unwrap();
    let args = cmd.get_args();
    let args = [s]
        .into_iter()
        .chain(args.into_iter().map(|arg| arg.to_str().unwrap()))
        .collect::<Vec<_>>()
        .join(" ");

    args
}

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// SGX_MODE:
    /// - SIM/SW
    #[arg(long, short, default_value_t = String::from("SIM"))]
    mode: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    Build(BuildArg),
}

#[derive(clap::Args)]
pub struct BuildArg {
    #[arg(long, short, default_value_t = String::from("all"))]
    target: String,
}

fn main() {
    let cli = Cli::parse();

    set_mode(&cli.mode);

    match cli.command {
        Commands::Build(arg) => build(&arg.target),
    }

    // let args: Vec<String> = std::env::args().collect();
    // if args.len() < 2 {
    //     eprintln!("Usage: xtask <command> [options]");
    //     eprintln!("Commands:");
    //     eprintln!("  build          Build the project");
    //     eprintln!("  sign           Sign the enclave");
    //     eprintln!("  clean          Clean build artifacts");
    //     exit(1);
    // }

    // match args[1].as_str() {
    //     "build" => build_all(),
    //     "sign" => sign_enclave(),
    //     "clean" => clean(),
    //     _ => {
    //         eprintln!("Unknown command: {}", args[1]);
    //         exit(1);
    //     }
    // }
}

pub fn set_mode(mode: &str) {
    std::env::set_var("SGX_MODE", mode);
}

pub fn build(target: &str) {
    match target {
        "app" => build_app(),
        "enclave" => build_enclave(),
        "all" => build_all(),
        _ => panic!(),
    }
}

fn build_std(
    path: &str,
    std: &str,
    flags: &str,
    features: &str,
) -> Result<std::process::Output, std::io::Error> {
    let mut s = String::new();
    let output = Command::new("cargo")
        .current_dir(path)
        .arg(std)
        .arg(flags)
        .arg(features)
        .stdout(std::io::stdout())
        .display(&mut s)
        .output();
    println!("{s}");
    output
}

pub fn build_all() {
    // build_edl();
    build_app();
    // build_enclave();
    // link_enclave();
    // sign_enclave();
}

pub fn build_edl() {
    println!("Building edl...");
    if !Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir("edl")
        .status()
        .expect("Failed to build edl")
        .success()
    {
        panic!("Failed to build edl");
    }
    println!("edl built successfully.");
}

pub fn build_app() {
    println!("Building app...");
    if !Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir("app")
        .status()
        .expect("Failed to build app")
        .success()
    {
        panic!("Failed to build app");
    }
    println!("App built successfully.");
}

pub fn build_enclave() {
    println!("Building enclave...");
    println!("Building std...");
    let output = build_std(
        RUST_TARGET_PATH.as_str(),
        RUST_BUILD_STD.as_str(),
        RUST_TARGET_FLAGS.as_str(),
        RUST_STD_FEATURES.as_str(),
    )
    .unwrap();
    if !Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir("enclave")
        .status()
        .expect("Failed to build enclave")
        .success()
    {
        panic!("Failed to build enclave");
    }
    println!("Enclave built successfully.");
}

pub fn link_enclave() {
    println!("Linking enclave...");
    let cxx = env::var("CXX").unwrap_or_else(|_| "g++".to_string());

    let input_path = Path::new("target/release/libenclave.a");
    let output_path = Path::new("target/release/enclave.so");
    let version_script_path = Path::new("enclave/enclave.lds");

    let status = Command::new(&cxx)
        .args(&[
            input_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
            "-Wl,--no-undefined",
            "-nostdlib",
            "-nodefaultlibs",
            "-nostartfiles",
            "-Wl,--start-group",
            "-L",
            "-lenclave",
            "-Wl,--end-group",
            &format!(
                "-Wl,--version-script={}",
                version_script_path.to_str().unwrap()
            ),
            "-Wl,-z,relro,-z,now,-z,noexecstack",
            "-Wl,-Bstatic",
            "-Wl,-Bsymbolic",
            "-Wl,--no-undefined",
            "-Wl,-pie",
            "-Wl,--export-dynamic",
            "-Wl,--gc-sections",
        ])
        .status()
        .expect("Failed to execute g++ command");

    if !status.success() {
        eprintln!("g++ command failed with status: {}", status);
        std::process::exit(1);
    }
    println!("Enclave linked successfully.");
}

pub fn sign_enclave() {
    let enclave_path = Path::new("target/release/enclave.so");
    let signed_path = Path::new("target/release/enclave.signed.so");
    let config_path = Path::new("enclave/config.xml");
    let key_path = Path::new("enclave/private.pem");

    if !enclave_path.exists() {
        panic!("libenclave.so not found. Please build the project first.");
    }

    println!("Signing enclave...");
    if !Command::new("sgx_sign")
        .arg("sign")
        .arg("-key")
        .arg(key_path)
        .arg("-enclave")
        .arg(enclave_path)
        .arg("-out")
        .arg(signed_path)
        .arg("-config")
        .arg(config_path)
        .status()
        .expect("Failed to sign enclave")
        .success()
    {
        panic!("Failed to sign enclave");
    }

    println!("Enclave signed successfully.");
}

pub fn clean() {
    let paths = vec!["app/target", "enclave/target", "xtask/target", "target"];

    for path in paths {
        let dir = Path::new(path);
        if dir.exists() {
            println!("Cleaning {}", path);
            fs::remove_dir_all(dir).expect("Failed to clean directory");
        }
    }

    println!("Clean completed.");
}
