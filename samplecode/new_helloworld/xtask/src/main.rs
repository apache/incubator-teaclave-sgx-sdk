use std::env;
use std::fs;
use std::path::Path;
use std::process::exit;
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: xtask <command> [options]");
        eprintln!("Commands:");
        eprintln!("  build          Build the project");
        eprintln!("  sign           Sign the enclave");
        eprintln!("  clean          Clean build artifacts");
        exit(1);
    }

    match args[1].as_str() {
        "build" => build_all(),
        "sign" => sign_enclave(),
        "clean" => clean(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            exit(1);
        }
    }
}

pub fn build_all() {
    build_edl();
    build_app();
    build_enclave();
    // link_enclave();
    sign_enclave();
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
            &format!("-Wl,--version-script={}", version_script_path.to_str().unwrap()),
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
