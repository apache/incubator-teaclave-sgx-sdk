fn main() {
    // 获取当前工作目录
    // let current_dir = std::env::current_dir().unwrap();
    // 设置链接器脚本路径
    // let version_script_path = current_dir.join("enclave.lds");
    println!("cargo:rerun-if-env-changed=SGX_MODE");
    println!("cargo:rerun-if-changed=build.rs");
    // println!("cargo:rerun-if-changed=enclave.lds");
    // println!(
    //     "cargo:rustc-flags=-Wl --version-script={}",
    //     version_script_path.display()
    // );
    // println!(
    //     "cargo:rustc-link-arg=-Wl,--version-script={}",
    //     version_script_path.display()
    // );
}
