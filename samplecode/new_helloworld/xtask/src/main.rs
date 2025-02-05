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
    link_enclave();
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
    let enclave_obj = Path::new("enclave/enclave_t.o");
    let enclave_lib = Path::new("enclave/target/release/libenclave.a");
    let output_so = Path::new("enclave/target/release/enclave.so");

    if !enclave_obj.exists() || !enclave_lib.exists() {
        panic!("Enclave object or library not found. Please build first.");
    }

    println!("Linking enclave...");
    if !Command::new("ld")
        .arg("-nostdlib")
        .arg("-nodefaultlibs")
        .arg("-nostartfiles")
        .arg("-o")
        .arg(output_so)
        .arg(enclave_obj)
        .arg(enclave_lib)
        .arg("-T")
        .arg("enclave/enclave.lds")
        .status()
        .expect("Failed to link enclave")
        .success()
    {
        panic!("Failed to link enclave");
    }
    println!("Enclave linked successfully.");
}

pub fn sign_enclave() {
    let enclave_path = Path::new("enclave/target/release/enclave.so");
    let signed_path = Path::new("enclave/target/release/enclave.signed.so");
    let config_path = Path::new("enclave/config.xml");
    let key_path = Path::new("enclave/private.pem");

    if !enclave_path.exists() {
        panic!("enclave.so not found. Please build the project first.");
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
