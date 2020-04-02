// Copyright 2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
extern crate sgx_build_helper as build_helper;

use std::env;
use std::process::Command;
use build_helper::{run, native_lib_boilerplate};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let target = env::var("TARGET").expect("TARGET was not set");
    let host = env::var("HOST").expect("HOST was not set");

    if target.contains("linux") {
        if target.contains("musl") {
            // musl is handled in lib.rs
        } else if !target.contains("android") {
            println!("cargo:rustc-link-lib=gcc_s");
        }
    } else if target.contains("freebsd") {
        println!("cargo:rustc-link-lib=gcc_s");
    } else if target.contains("rumprun") {
        println!("cargo:rustc-link-lib=unwind");
    } else if target.contains("netbsd") {
        println!("cargo:rustc-link-lib=gcc_s");
    } else if target.contains("openbsd") {
        println!("cargo:rustc-link-lib=c++abi");
    } else if target.contains("solaris") {
        println!("cargo:rustc-link-lib=gcc_s");
    } else if target.contains("bitrig") {
        println!("cargo:rustc-link-lib=c++abi");
    } else if target.contains("dragonfly") {
        println!("cargo:rustc-link-lib=gcc_pic");
    } else if target.contains("windows-gnu") {
        println!("cargo:rustc-link-lib=static-nobundle=gcc_eh");
        println!("cargo:rustc-link-lib=static-nobundle=pthread");
    } else if target.contains("fuchsia") {
        println!("cargo:rustc-link-lib=unwind");
    } else if target.contains("haiku") {
        println!("cargo:rustc-link-lib=gcc_s");
    } else if target.contains("redox") {
        println!("cargo:rustc-link-lib=gcc");
    } else if target.contains("cloudabi") {
        println!("cargo:rustc-link-lib=unwind");
    }

    let _ = build_libunwind(&host, &target);
}

fn build_libunwind(host: &str, target: &str) -> Result<(), ()> {
    let filter = vec![
        "config",
        "autom4te.cache",
        "Makefile.in",
        "config.h.in",
        "configure",
        "aclocal.m4",
        "INSTALL"];
    let native = native_lib_boilerplate(
                    "sgx_unwind-1.1.1/libunwind",
                    "libunwind",
                    "unwind",
                    "src/.libs",
                    &filter)?;

    let mut cflags = String::new();
    cflags += " -fstack-protector -ffreestanding -nostdinc -fvisibility=hidden -fpie -fno-strict-overflow -fno-delete-null-pointer-checks";
    cflags += " -O2";

    let mitigation_cflags = " -mindirect-branch-register -mfunction-return=thunk-extern";
    let mitigation_asflags = " -fno-plt";
    let mitigation_loadflags = " -Wa,-mlfence-after-load=yes -Wa,-mlfence-before-ret=not";
    let mitigation_cfflags = " -Wa,-mlfence-before-indirect-branch=register -Wa,-mlfence-before-ret=not";
    let mitigation = env::var("MITIGATION_CVE_2020_0551").unwrap_or_default();
    match mitigation.as_ref() {
        "LOAD" => {
            cflags += mitigation_cflags;
            cflags += mitigation_asflags;
            cflags += mitigation_loadflags;
        },
        "CF" => {
            cflags += mitigation_cflags;
            cflags += mitigation_asflags;
            cflags += mitigation_cfflags;
        },
        _  => {},
    }

    run(Command::new("sh")
                .current_dir(&native.out_dir)
                .arg(native.src_dir.join("autogen-linux.sh").to_str().unwrap())
                .arg(format!("--host={}", build_helper::gnu_target(target)))
                .arg(format!("--build={}", build_helper::gnu_target(host)))
                .env("CFLAGS", cflags));

    run(Command::new(build_helper::make(host))
                .current_dir(&native.out_dir)
                .arg(format!("INCDIR={}", native.src_dir.display()))
                .arg("-j5"));
    Ok(())
}
