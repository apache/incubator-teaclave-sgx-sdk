#![cfg(feature = "unstable")]

extern crate compiletest_rs as compiletest;

use std::fs;
use std::result::Result;

use compiletest::common::Mode;

fn run_mode(mode: Mode) {
    let config = compiletest::Config {
        mode: mode,
        src_base: format!("tests/{}", mode).into(),
        target_rustcflags: fs::read_dir("../target/debug/deps")
            .unwrap()
            .map(Result::unwrap)
            .filter(|entry| {
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();
                file_name.starts_with("libbitflags-") && file_name.ends_with(".rlib")
            })
            .max_by_key(|entry| entry.metadata().unwrap().modified().unwrap())
            .map(|entry| format!("--extern bitflags={}", entry.path().to_string_lossy())),
        ..Default::default()
    };

    compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
    run_mode(Mode::CompileFail);
}
