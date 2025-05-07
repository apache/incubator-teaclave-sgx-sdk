use std::env;

fn main() {
    let pwd = env::current_dir().unwrap();
    println!("cargo:rustc-link-search=native={}/../lib", pwd.display());
    println!("cargo:rustc-link-lib=static=enclave_u");
}
