// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

extern crate sgx_build_helper as build_helper;
extern crate cc;

use build_helper::native_lib_boilerplate;
use std::env;
use std::fs::File;

fn main() {
    let target = env::var("TARGET").expect("TARGET was not set");
    if cfg!(feature = "backtrace") {
        let _ = build_libbacktrace(&target);

        println!("cargo:rustc-cfg=RUST_BACKTRACE=\"1\"");
        //println!("cargo:rustc-cfg=RUST_BACKTRACE=\"full\"");
    }
}

fn build_libbacktrace(_target: &str) -> Result<(), ()> {
    let native = native_lib_boilerplate("sgx_tstd/libbacktrace", "libbacktrace", "backtrace", "")?;

    let mut build = cc::Build::new();
    build
        .flag("-fvisibility=hidden")
        .include("./libbacktrace")
        .include(&native.out_dir)
        .out_dir(&native.out_dir)
        .warnings(false)
        .file("./libbacktrace/mmap.c")
        .file("./libbacktrace/mmapio.c")
        .file("./libbacktrace/backtrace.c")
        .file("./libbacktrace/dwarf.c")
        .file("./libbacktrace/fileline.c")
        .file("./libbacktrace/posix.c")
        .file("./libbacktrace/sort.c")
        .file("./libbacktrace/state.c");

    let any_debug = env::var("RUSTC_DEBUGINFO").unwrap_or_default() == "true" ||
        env::var("RUSTC_DEBUGINFO_LINES").unwrap_or_default() == "true";
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
    Ok(())
}
