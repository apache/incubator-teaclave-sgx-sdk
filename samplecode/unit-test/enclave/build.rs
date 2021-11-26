use std::env;

fn main() {
    let is_sim = env::var("SGX_MODE").unwrap_or_else(|_| "HW".to_string());

    match is_sim.as_ref() {
        "SW" => {}
        _ => {
            // HW by default
            println!("cargo:rustc-cfg=feature=\"hw_test\"");
        }
    }
}
