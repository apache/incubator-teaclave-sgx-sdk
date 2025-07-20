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

use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs};

/// A helper macro to `unwrap` a result except also print out details like:
///
/// * The file/line of the panic
/// * The expression that failed
/// * The error itself
///
/// This is currently used judiciously throughout the build system rather than
/// using a `Result` with `try!`, but this may change one day...
#[macro_export]
macro_rules! t {
    ($e:expr) => {
        match $e {
            Ok(e) => e,
            Err(e) => panic!("{} failed with {}", stringify!($e), e),
        }
    };
    // it can show extra info in the second parameter
    ($e:expr, $extra:expr) => {
        match $e {
            Ok(e) => e,
            Err(e) => panic!("{} failed with {} ({:?})", stringify!($e), e, $extra),
        }
    };
}

// Because Cargo adds the compiler's dylib path to our library search path, llvm-config may
// break: the dylib path for the compiler, as of this writing, contains a copy of the LLVM
// shared library, which means that when our freshly built llvm-config goes to load it's
// associated LLVM, it actually loads the compiler's LLVM. In particular when building the first
// compiler (i.e., in stage 0) that's a problem, as the compiler's LLVM is likely different from
// the one we want to use. As such, we restore the environment to what bootstrap saw. This isn't
// perfect -- we might actually want to see something from Cargo's added library paths -- but
// for now it works.
pub fn restore_library_path() {
    println!("cargo:rerun-if-env-changed=REAL_LIBRARY_PATH_VAR");
    println!("cargo:rerun-if-env-changed=REAL_LIBRARY_PATH");
    let key = env::var_os("REAL_LIBRARY_PATH_VAR").expect("REAL_LIBRARY_PATH_VAR");
    if let Some(env) = env::var_os("REAL_LIBRARY_PATH") {
        env::set_var(&key, env);
    } else {
        env::remove_var(&key);
    }
}

/// Run the command, printing what we are running.
pub fn run_verbose(cmd: &mut Command) {
    println!("running: {:?}", cmd);
    run_silent(cmd);
}

pub fn run(cmd: &mut Command) {
    println!("running: {:?}", cmd);
    run_silent(cmd);
}

pub fn run_silent(cmd: &mut Command) {
    if !try_run_silent(cmd) {
        std::process::exit(1);
    }
}

pub fn try_run_silent(cmd: &mut Command) -> bool {
    let status = match cmd.status() {
        Ok(status) => status,
        Err(e) => fail(&format!(
            "failed to execute command: {:?}\nerror: {}",
            cmd, e
        )),
    };
    if !status.success() {
        println!(
            "\n\ncommand did not execute successfully: {:?}\n\
             expected success, got: {}\n\n",
            cmd, status
        );
    }
    status.success()
}

pub fn run_suppressed(cmd: &mut Command) {
    if !try_run_suppressed(cmd) {
        std::process::exit(1);
    }
}

pub fn try_run_suppressed(cmd: &mut Command) -> bool {
    let output = match cmd.output() {
        Ok(status) => status,
        Err(e) => fail(&format!(
            "failed to execute command: {:?}\nerror: {}",
            cmd, e
        )),
    };
    if !output.status.success() {
        println!(
            "\n\ncommand did not execute successfully: {:?}\n\
             expected success, got: {}\n\n\
             stdout ----\n{}\n\
             stderr ----\n{}\n\n",
            cmd,
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    output.status.success()
}

pub fn gnu_target(target: &str) -> &str {
    match target {
        "i686-pc-windows-msvc" => "i686-pc-win32",
        "x86_64-pc-windows-msvc" => "x86_64-pc-win32",
        "i686-pc-windows-gnu" => "i686-w64-mingw32",
        "x86_64-pc-windows-gnu" => "x86_64-w64-mingw32",
        s => s,
    }
}

pub fn make(host: &str) -> PathBuf {
    if host.contains("dragonfly")
        || host.contains("freebsd")
        || host.contains("netbsd")
        || host.contains("openbsd")
    {
        PathBuf::from("gmake")
    } else {
        PathBuf::from("make")
    }
}

pub fn output(cmd: &mut Command) -> String {
    let output = match cmd.stderr(Stdio::inherit()).output() {
        Ok(status) => status,
        Err(e) => fail(&format!(
            "failed to execute command: {:?}\nerror: {}",
            cmd, e
        )),
    };
    assert!(
        output.status.success(),
        "command did not execute successfully: {:?}\n\
            expected success, got: {}",
        cmd,
        output.status
    );
    String::from_utf8(output.stdout).unwrap()
}

pub fn rerun_if_changed_anything_in_dir(dir: &Path, filter: &[&str]) {
    let mut stack = dir
        .read_dir()
        .unwrap()
        .map(|e| e.unwrap())
        .filter(|e| !is_in_filter_list(&e.file_name(), filter))
        .collect::<Vec<_>>();
    while let Some(entry) = stack.pop() {
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            stack.extend(
                path.read_dir()
                    .unwrap()
                    .map(|e| e.unwrap())
                    .filter(|e| !is_in_filter_list(&e.file_name(), filter)),
            );
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}

fn is_in_filter_list(elem: &OsStr, filter: &[&str]) -> bool {
    filter.iter().any(|e| *e == elem) || elem == ".git"
}

/// Returns the last-modified time for `path`, or zero if it doesn't exist.
pub fn mtime(path: &Path) -> SystemTime {
    fs::metadata(path)
        .and_then(|f| f.modified())
        .unwrap_or(UNIX_EPOCH)
}

/// Returns `true` if `dst` is up to date given that the file or files in `src`
/// are used to generate it.
///
/// Uses last-modified time checks to verify this.
pub fn up_to_date(src: &Path, dst: &Path) -> bool {
    if !dst.exists() {
        return false;
    }
    let threshold = mtime(dst);
    let meta = match fs::metadata(src) {
        Ok(meta) => meta,
        Err(e) => panic!("source {:?} failed to get metadata: {}", src, e),
    };
    if meta.is_dir() {
        dir_up_to_date(src, threshold)
    } else {
        meta.modified().unwrap_or(UNIX_EPOCH) <= threshold
    }
}

#[must_use]
pub struct NativeLibBoilerplate {
    pub src_dir: PathBuf,
    pub out_dir: PathBuf,
}

impl NativeLibBoilerplate {
    /// On OSX we don't want to ship the exact filename that compiler-rt builds.
    /// This conflicts with the system and ours is likely a wildly different
    /// version, so they can't be substituted.
    ///
    /// As a result, we rename it here but we need to also use
    /// `install_name_tool` on OSX to rename the commands listed inside of it to
    /// ensure it's linked against correctly.
    pub fn fixup_sanitizer_lib_name(&self, sanitizer_name: &str) {
        if env::var("TARGET").unwrap() != "x86_64-apple-darwin" {
            return;
        }

        let dir = self.out_dir.join("build/lib/darwin");
        let name = format!("clang_rt.{}_osx_dynamic", sanitizer_name);
        let src = dir.join(format!("lib{}.dylib", name));
        let new_name = format!("lib__rustc__{}.dylib", name);
        let dst = dir.join(&new_name);

        println!("{} => {}", src.display(), dst.display());
        fs::rename(&src, &dst).unwrap();
        let status = Command::new("install_name_tool")
            .arg("-id")
            .arg(format!("@rpath/{}", new_name))
            .arg(&dst)
            .status()
            .expect("failed to execute `install_name_tool`");
        assert!(status.success());
    }
}

impl Drop for NativeLibBoilerplate {
    fn drop(&mut self) {
        if !thread::panicking() {
            t!(File::create(self.out_dir.join("rustbuild.timestamp")));
        }
    }
}

// Perform standard preparations for native libraries that are build only once for all stages.
// Emit rerun-if-changed and linking attributes for Cargo, check if any source files are
// updated, calculate paths used later in actual build with CMake/make or C/C++ compiler.
// If Err is returned, then everything is up-to-date and further build actions can be skipped.
// Timestamps are created automatically when the result of `native_lib_boilerplate` goes out
// of scope, so all the build actions should be completed until then.
#[allow(clippy::result_unit_err)]
pub fn native_lib_boilerplate(
    src_name: &str,
    out_name: &str,
    link_name: &str,
    search_subdir: &str,
    filter: &[&str],
) -> Result<NativeLibBoilerplate, ()> {
    let current_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let src_dir = current_dir.join("..").join(src_name);
    rerun_if_changed_anything_in_dir(&src_dir, filter);

    let out_dir =
        env::var_os("RUSTBUILD_NATIVE_DIR").unwrap_or_else(|| env::var_os("OUT_DIR").unwrap());
    let out_dir = PathBuf::from(out_dir).join(out_name);
    t!(fs::create_dir_all(&out_dir));
    if link_name.contains('=') {
        println!("cargo:rustc-link-lib={}", link_name);
    } else {
        println!("cargo:rustc-link-lib=static={}", link_name);
    }
    println!(
        "cargo:rustc-link-search=native={}",
        out_dir.join(search_subdir).display()
    );

    let timestamp = out_dir.join("rustbuild.timestamp");
    if !up_to_date(Path::new("build.rs"), &timestamp) || !up_to_date(&src_dir, &timestamp) {
        Ok(NativeLibBoilerplate { src_dir, out_dir })
    } else {
        Err(())
    }
}

#[allow(clippy::result_unit_err)]
pub fn sanitizer_lib_boilerplate(
    sanitizer_name: &str,
) -> Result<(NativeLibBoilerplate, String), ()> {
    let (link_name, search_path, apple) = match &*env::var("TARGET").unwrap() {
        "x86_64-unknown-linux-gnu" => (
            format!("clang_rt.{}-x86_64", sanitizer_name),
            "build/lib/linux",
            false,
        ),
        "x86_64-apple-darwin" => (
            format!("clang_rt.{}_osx_dynamic", sanitizer_name),
            "build/lib/darwin",
            true,
        ),
        _ => return Err(()),
    };
    let to_link = if apple {
        format!("dylib=__rustc__{}", link_name)
    } else {
        format!("static={}", link_name)
    };
    let lib = native_lib_boilerplate(
        "libcompiler_builtins/compiler-rt",
        sanitizer_name,
        &to_link,
        search_path,
        &[],
    )?;
    Ok((lib, link_name))
}

fn dir_up_to_date(src: &Path, threshold: SystemTime) -> bool {
    t!(fs::read_dir(src)).map(|e| t!(e)).all(|e| {
        let meta = t!(e.metadata());
        if meta.is_dir() {
            dir_up_to_date(&e.path(), threshold)
        } else {
            meta.modified().unwrap_or(UNIX_EPOCH) < threshold
        }
    })
}

fn fail(s: &str) -> ! {
    println!("\n\n{}\n\n", s);
    std::process::exit(1);
}
