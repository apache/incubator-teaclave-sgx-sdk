use std::env;
use std::process::Command;

fn main() {
    // println!("cargo:rerun-if-changed=build.rs");

    // println!("cargo:rustc-link-arg=-Wl,--no-undefined");
    // println!("cargo:rustc-link-search=native=/opt/sgxsdk/lib64");
    // println!("cargo:rustc-link-arg=-nostdlib");
    // // println!("cargo:rustc-link-lib=sgx_tstdc");
    // println!("cargo:rustc-link-lib=static=sgx_trts");
    // println!("cargo:rustc-link-lib=static=sgx_tservice");
    // println!("cargo:rustc-link-arg=-nodefaultlibs");
    // println!("cargo:rustc-link-arg=-nostartfiles");
    // println!("cargo:rustc-link-arg=-Wl,--version-script=enclave/enclave.lds");
    // println!("cargo:rustc-link-arg=-Wl,-z,relro,-z,now,-z,noexecstack");
    // println!("cargo:rustc-link-arg=-Wl,-Bstatic");
    // println!("cargo:rustc-link-arg=-Wl,-Bsymbolic");
    // println!("cargo:rustc-link-arg=-Wl,--no-undefined");
    // println!("cargo:rustc-link-arg=-Wl,-pie");
    // println!("cargo:rustc-link-arg=-Wl,--export-dynamic");
    // println!("cargo:rustc-link-arg=-Wl,--gc-sections");
}

// fn main() {
//     // 获取当前工作目录
//     // let current_dir = std::env::current_dir().unwrap();
//     // 设置链接器脚本路径
//     // let version_script_path = current_dir.join("enclave.lds");

//     // println!("cargo:rerun-if-changed=enclave.lds");
//     // println!(
//     //     "cargo:rustc-flags=-Wl --version-script={}",
//     //     version_script_path.display()
//     // );
//     // println!(
//     //     "cargo:rustc-link-arg=-Wl,--version-script={}",
//     //     version_script_path.display()
//     // );
// }
