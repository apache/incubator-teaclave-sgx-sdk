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

extern crate build_helper;

use std::env;
use std::process::Command;
use build_helper::{run, native_lib_boilerplate};

fn main() {
    let target = env::var("TARGET").expect("TARGET was not set");
    let host = env::var("HOST").expect("HOST was not set");
    if cfg!(feature = "backtrace") {
        let _ = build_libbacktrace(&host, &target);

        println!("cargo:rustc-cfg=RUST_BACKTRACE=\"1\"");
        //println!("cargo:rustc-cfg=RUST_BACKTRACE=\"full\"");
    }
}

fn build_libbacktrace(host: &str, target: &str) -> Result<(), ()> {
    let native = native_lib_boilerplate("libbacktrace", "libbacktrace", "backtrace", ".libs")?;

    run(Command::new("sh")
                .current_dir(&native.out_dir)
                .arg(native.src_dir.join("configure").to_str().unwrap()
                                   .replace("C:\\", "/c/")
                                   .replace("\\", "/"))
                .arg("--with-pic")
                .arg("--disable-multilib")
                .arg("--disable-shared")
                .arg("--disable-host-shared")
                .arg(format!("--host={}", build_helper::gnu_target(target)))
                .arg(format!("--build={}", build_helper::gnu_target(host)))
                .env("CFLAGS", env::var("CFLAGS").unwrap_or_default() + " -fvisibility=hidden"));

    run(Command::new(build_helper::make(host))
                .current_dir(&native.out_dir)
                .arg(format!("INCDIR={}", native.src_dir.display()))
                .arg("-j").arg(env::var("NUM_JOBS").expect("NUM_JOBS was not set")));

    Ok(())
}