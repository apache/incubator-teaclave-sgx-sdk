// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use sgx_build_helper as build_helper;

use build_helper::native_lib_boilerplate;
use std::env;
use std::fs::File;

fn main() {
    println!("cargo:rerun-if-env-changed=SGX_MODE");
    println!("cargo:rerun-if-changed=build.rs");

    let mode = env::var("SGX_MODE").unwrap_or_else(|_| "HW".to_string());
    let is_use_read = if mode.as_str() == "HYPER" {
        println!("cargo:rustc-cfg=feature=\"use_read\"");
        true
    } else {
        cfg!(feature = "use_read")
    };

    let target = env::var("TARGET").expect("TARGET was not set");
    let _ = build_libbacktrace(&target, is_use_read);
}

fn build_libbacktrace(_target: &str, is_use_read: bool) -> Result<(), ()> {
    native_lib_boilerplate(
        "sgx_backtrace_sys/libbacktrace",
        "libbacktrace",
        "backtrace",
        "",
        &[],
    )
    .map(|native| {
        let mut build = cc::Build::new();
        build
            .opt_level(2)
            .flag("-fstack-protector")
            .flag("-ffreestanding")
            .flag("-fpie")
            .flag("-fno-strict-overflow")
            .flag("-fno-delete-null-pointer-checks")
            .flag("-fvisibility=hidden")
            .flag("-ffunction-sections")
            .flag("-fdata-sections")
            .include("./libbacktrace")
            .include("../../common/inc")
            .include(&native.out_dir)
            .out_dir(&native.out_dir)
            .warnings(false)
            .file("./libbacktrace/backtrace.c")
            .file("./libbacktrace/dwarf.c")
            .file("./libbacktrace/fileline.c")
            .file("./libbacktrace/posix.c")
            .file("./libbacktrace/sort.c")
            .file("./libbacktrace/state.c");

        if is_use_read {
            build
                .file("./libbacktrace/alloc.c")
                .file("./libbacktrace/read.c");
        } else {
            build
                .file("./libbacktrace/mmap.c")
                .file("./libbacktrace/mmapio.c");
        }

        let mitigation_cflags1 = "-mindirect-branch-register";
        let mitigation_cflags2 = "-mfunction-return=thunk-extern";
        let mitigation_asflags = "-fno-plt";
        let mitigation_loadflags1 = "-Wa,-mlfence-after-load=yes";
        let mitigation_loadflags2 = "-Wa,-mlfence-before-ret=not";
        let mitigation_cfflags1 = "-Wa,-mlfence-before-indirect-branch=register";
        let mitigation_cfflags2 = "-Wa,-mlfence-before-ret=not";
        let mitigation = env::var("MITIGATION_CVE_2020_0551").unwrap_or_default();
        match mitigation.as_ref() {
            "LOAD" => {
                build
                    .flag(mitigation_cflags1)
                    .flag(mitigation_cflags2)
                    .flag(mitigation_asflags)
                    .flag(mitigation_loadflags1)
                    .flag(mitigation_loadflags2);
            }
            "CF" => {
                build
                    .flag(mitigation_cflags1)
                    .flag(mitigation_cflags2)
                    .flag(mitigation_asflags)
                    .flag(mitigation_cfflags1)
                    .flag(mitigation_cfflags2);
            }
            _ => {}
        }

        let any_debug = env::var("RUSTC_DEBUGINFO").unwrap_or_default() == "true"
            || env::var("RUSTC_DEBUGINFO_LINES").unwrap_or_default() == "true";
        build.debug(any_debug);

        build.file("./libbacktrace/elf.c");

        let pointer_width = env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap();
        if pointer_width == "64" {
            build.define("BACKTRACE_ELF_SIZE", "64");
        } else {
            build.define("BACKTRACE_ELF_SIZE", "32");
        }

        File::create(native.out_dir.join("backtrace-supported.h")).unwrap();
        build.define("BACKTRACE_SUPPORTED", "1");
        build.define("BACKTRACE_USES_MALLOC", "0");
        build.define("BACKTRACE_SUPPORTS_THREADS", "0");
        build.define("BACKTRACE_SUPPORTS_DATA", "0");

        File::create(native.out_dir.join("config.h")).unwrap();
        build.define("HAVE_DL_ITERATE_PHDR", "1");
        build.define("_GNU_SOURCE", "1");
        build.define("_LARGE_FILES", "1");

        build.compile("backtrace");
    })
    .or(Ok(()))
}
