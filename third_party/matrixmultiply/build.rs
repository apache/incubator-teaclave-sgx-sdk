
//!

use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if let Ok(features) = env::var("CARGO_CFG_TARGET_FEATURE") {
        if features.split(",").map(|s| s.trim()).any(|feat| feat == "avx") {
            println!("cargo:rustc-cfg=sgemm_8x8");
        }
    }
}
