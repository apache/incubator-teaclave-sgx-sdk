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
// under the License..

#![allow(unused_imports)]

use sgx_build_helper as build_helper;
use sgx_download_prebuilt as download_prebuilt;

use build_helper::{native_lib_boilerplate, run};
use std::env;
use std::process::Command;

fn main() -> Result<(), &'static str> {
    println!("cargo:rerun-if-changed=build.rs");
    let target = env::var("TARGET").expect("TARGET was not set");
    let host = env::var("HOST").expect("HOST was not set");

    build_libtlibc(&host, &target).map_err(|_| "Faile to build tlibc library.")?;

    build_libtsafecrt(&host, &target).map_err(|_| "Faile to build tsafecrt library.")
}

fn build_libtlibc(host: &str, _target: &str) -> Result<(), ()> {
    println!("cargo:rerun-if-env-changed=USE_OPT_LIBS");

    let use_optlibs = is_use_optlibs();

    let native = native_lib_boilerplate("sgx_tlibc_sys/tlibc", "libtlibc", "tlibc", "", &[])?;

    let mut command = Command::new(build_helper::make(host));
    command
        .current_dir(&native.src_dir)
        .arg(format!("OUT_DIR={}", native.out_dir.display()));

    if use_optlibs {
        command.arg(format!("USE_OPTLIBS={}", 1));
    }
    run(&mut command);
    Ok(())
}

fn build_libtsafecrt(host: &str, _target: &str) -> Result<(), ()> {
    let native =
        native_lib_boilerplate("sgx_tlibc_sys/tsafecrt", "libtsafecrt", "tsafecrt", "", &[])?;

    run(Command::new(build_helper::make(host))
        .current_dir(&native.src_dir)
        .arg(format!("OUT_DIR={}", native.out_dir.display())));
    Ok(())
}

fn is_use_optlibs() -> bool {
    let use_optlibs = env::var("USE_OPT_LIBS").unwrap_or_else(|_| "0".to_string());
    if <String as AsRef<str>>::as_ref(&use_optlibs) == "1" {
        println!("cargo:rustc-cfg=feature=\"opt_lib\"");
        true
    } else {
        cfg!(feature = "use_optlibs")
    }
}
