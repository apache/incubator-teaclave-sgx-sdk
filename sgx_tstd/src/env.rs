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

//! Inspection and manipulation of the process's environment.
//!
//! This module contains functions to inspect various aspects such as
//! environment variables, process arguments, the current directory, and various
//! other important directories.
//!
//! There are several functions and structs in this module that have a
//! counterpart ending in `os`. Those ending in `os` will return an [`OsString`]
//! and those without will return a [`String`].

#![allow(clippy::needless_doctest_main)]

#[cfg(feature = "unit_test")]
mod tests;

use crate::error::Error;
use crate::ffi::{OsStr, OsString};
use crate::fmt;
#[cfg(feature = "env")]
use crate::io;
#[cfg(feature = "env")]
use crate::path::Path;
use crate::path::PathBuf;
use crate::sys;
use crate::sys::os as os_imp;

/// Returns the current working directory as a [`PathBuf`].
///
/// # Platform-specific behavior
///
/// This function [currently] corresponds to the `getcwd` function on Unix
/// and the `GetCurrentDirectoryW` function on Windows.
///
/// [currently]: crate::io#platform-specific-behavior
///
/// # Errors
///
/// Returns an [`Err`] if the current working directory value is invalid.
/// Possible cases:
///
/// * Current directory does not exist.
/// * There are insufficient permissions to access the current directory.
///
/// # Examples
///
/// ```
/// use std::env;
///
/// fn main() -> std::io::Result<()> {
///     let path = env::current_dir()?;
///     println!("The current directory is {}", path.display());
///     Ok(())
/// }
/// ```
#[cfg(feature = "env")]
pub fn current_dir() -> io::Result<PathBuf> {
    os_imp::getcwd()
}

/// Changes the current working directory to the specified path.
///
/// # Platform-specific behavior
///
/// This function [currently] corresponds to the `chdir` function on Unix
/// and the `SetCurrentDirectoryW` function on Windows.
///
/// Returns an [`Err`] if the operation fails.
///
/// [currently]: crate::io#platform-specific-behavior
///
/// # Examples
///
/// ```
/// use std::env;
/// use std::path::Path;
///
/// let root = Path::new("/");
/// assert!(env::set_current_dir(&root).is_ok());
/// println!("Successfully changed working directory to {}!", root.display());
/// ```
#[cfg(feature = "env")]
pub fn set_current_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    os_imp::chdir(path.as_ref())
}

/// An iterator over a snapshot of the environment variables of this process.
///
/// This structure is created by [`env::vars()`]. See its documentation for more.
///
/// [`env::vars()`]: vars
pub struct Vars {
    inner: VarsOs,
}

/// An iterator over a snapshot of the environment variables of this process.
///
/// This structure is created by [`env::vars_os()`]. See its documentation for more.
///
/// [`env::vars_os()`]: vars_os
pub struct VarsOs {
    inner: os_imp::Env,
}

/// Returns an iterator of (variable, value) pairs of strings, for all the
/// environment variables of the current process.
///
/// The returned iterator contains a snapshot of the process's environment
/// variables at the time of this invocation. Modifications to environment
/// variables afterwards will not be reflected in the returned iterator.
///
/// # Panics
///
/// While iterating, the returned iterator will panic if any key or value in the
/// environment is not valid unicode. If this is not desired, consider using
/// [`env::vars_os()`].
///
/// # Examples
///
/// ```
/// use std::env;
///
/// // We will iterate through the references to the element returned by
/// // env::vars();
/// for (key, value) in env::vars() {
///     println!("{key}: {value}");
/// }
/// ```
///
/// [`env::vars_os()`]: vars_os
#[must_use]
pub fn vars() -> Vars {
    Vars { inner: vars_os() }
}

/// Returns an iterator of (variable, value) pairs of OS strings, for all the
/// environment variables of the current process.
///
/// The returned iterator contains a snapshot of the process's environment
/// variables at the time of this invocation. Modifications to environment
/// variables afterwards will not be reflected in the returned iterator.
///
/// Note that the returned iterator will not check if the environment variables
/// are valid Unicode. If you want to panic on invalid UTF-8,
/// use the [`vars`] function instead.
///
/// # Examples
///
/// ```
/// use std::env;
///
/// // We will iterate through the references to the element returned by
/// // env::vars_os();
/// for (key, value) in env::vars_os() {
///     println!("{key:?}: {value:?}");
/// }
/// ```
#[must_use]
pub fn vars_os() -> VarsOs {
    VarsOs { inner: os_imp::env() }
}

impl Iterator for Vars {
    type Item = (String, String);
    fn next(&mut self) -> Option<(String, String)> {
        self.inner.next().map(|(a, b)| (a.into_string().unwrap(), b.into_string().unwrap()))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl fmt::Debug for Vars {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner: VarsOs { inner } } = self;
        f.debug_struct("Vars").field("inner", &inner.str_debug()).finish()
    }
}

impl Iterator for VarsOs {
    type Item = (OsString, OsString);
    fn next(&mut self) -> Option<(OsString, OsString)> {
        self.inner.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl fmt::Debug for VarsOs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        f.debug_struct("VarsOs").field("inner", inner).finish()
    }
}

/// Fetches the environment variable `key` from the current process.
///
/// # Errors
///
/// This function will return an error if the environment variable isn't set.
///
/// This function may return an error if the environment variable's name contains
/// the equal sign character (`=`) or the NUL character.
///
/// This function will return an error if the environment variable's value is
/// not valid Unicode. If this is not desired, consider using [`var_os`].
///
/// # Examples
///
/// ```
/// use std::env;
///
/// let key = "HOME";
/// match env::var(key) {
///     Ok(val) => println!("{key}: {val:?}"),
///     Err(e) => println!("couldn't interpret {key}: {e}"),
/// }
/// ```
pub fn var<K: AsRef<OsStr>>(key: K) -> Result<String, VarError> {
    _var(key.as_ref())
}

fn _var(key: &OsStr) -> Result<String, VarError> {
    match var_os(key) {
        Some(s) => s.into_string().map_err(VarError::NotUnicode),
        None => Err(VarError::NotPresent),
    }
}

/// Fetches the environment variable `key` from the current process, returning
/// [`None`] if the variable isn't set or if there is another error.
///
/// It may return `None` if the environment variable's name contains
/// the equal sign character (`=`) or the NUL character.
///
/// Note that this function will not check if the environment variable
/// is valid Unicode. If you want to have an error on invalid UTF-8,
/// use the [`var`] function instead.
///
/// # Examples
///
/// ```
/// use std::env;
///
/// let key = "HOME";
/// match env::var_os(key) {
///     Some(val) => println!("{key}: {val:?}"),
///     None => println!("{key} is not defined in the environment.")
/// }
/// ```
#[must_use]
pub fn var_os<K: AsRef<OsStr>>(key: K) -> Option<OsString> {
    _var_os(key.as_ref())
}

fn _var_os(key: &OsStr) -> Option<OsString> {
    os_imp::getenv(key)
        .unwrap_or_else(|e| panic!("failed to get environment variable `{key:?}`: {e}"))
}

/// The error type for operations interacting with environment variables.
/// Possibly returned from [`env::var()`].
///
/// [`env::var()`]: var
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum VarError {
    /// The specified environment variable was not present in the current
    /// process's environment.
    NotPresent,

    /// The specified environment variable was found, but it did not contain
    /// valid unicode data. The found data is returned as a payload of this
    /// variant.
    NotUnicode(OsString),
}

impl fmt::Display for VarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            VarError::NotPresent => write!(f, "environment variable not found"),
            VarError::NotUnicode(ref s) => {
                write!(f, "environment variable was not valid unicode: {:?}", s)
            }
        }
    }
}

impl Error for VarError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        match *self {
            VarError::NotPresent => "environment variable not found",
            VarError::NotUnicode(..) => "environment variable was not valid unicode",
        }
    }
}

/// Sets the environment variable `key` to the value `value` for the currently running
/// process.
///
/// Note that while concurrent access to environment variables is safe in Rust,
/// some platforms only expose inherently unsafe non-threadsafe APIs for
/// inspecting the environment. As a result, extra care needs to be taken when
/// auditing calls to unsafe external FFI functions to ensure that any external
/// environment accesses are properly synchronized with accesses in Rust.
///
/// Discussion of this unsafety on Unix may be found in:
///
///  - [Austin Group Bugzilla](https://austingroupbugs.net/view.php?id=188)
///  - [GNU C library Bugzilla](https://sourceware.org/bugzilla/show_bug.cgi?id=15607#c2)
///
/// # Panics
///
/// This function may panic if `key` is empty, contains an ASCII equals sign `'='`
/// or the NUL character `'\0'`, or when `value` contains the NUL character.
///
/// # Examples
///
/// ```
/// use std::env;
///
/// let key = "KEY";
/// env::set_var(key, "VALUE");
/// assert_eq!(env::var(key), Ok("VALUE".to_string()));
/// ```
pub fn set_var<K: AsRef<OsStr>, V: AsRef<OsStr>>(key: K, value: V) {
    _set_var(key.as_ref(), value.as_ref())
}

fn _set_var(key: &OsStr, value: &OsStr) {
    os_imp::setenv(key, value).unwrap_or_else(|e| {
        panic!("failed to set environment variable `{key:?}` to `{value:?}`: {e}")
    })
}

/// Removes an environment variable from the environment of the currently running process.
///
/// Note that while concurrent access to environment variables is safe in Rust,
/// some platforms only expose inherently unsafe non-threadsafe APIs for
/// inspecting the environment. As a result extra care needs to be taken when
/// auditing calls to unsafe external FFI functions to ensure that any external
/// environment accesses are properly synchronized with accesses in Rust.
///
/// Discussion of this unsafety on Unix may be found in:
///
///  - [Austin Group Bugzilla](https://austingroupbugs.net/view.php?id=188)
///  - [GNU C library Bugzilla](https://sourceware.org/bugzilla/show_bug.cgi?id=15607#c2)
///
/// # Panics
///
/// This function may panic if `key` is empty, contains an ASCII equals sign
/// `'='` or the NUL character `'\0'`, or when the value contains the NUL
/// character.
///
/// # Examples
///
/// ```
/// use std::env;
///
/// let key = "KEY";
/// env::set_var(key, "VALUE");
/// assert_eq!(env::var(key), Ok("VALUE".to_string()));
///
/// env::remove_var(key);
/// assert!(env::var(key).is_err());
/// ```
pub fn remove_var<K: AsRef<OsStr>>(key: K) {
    _remove_var(key.as_ref())
}

fn _remove_var(key: &OsStr) {
    os_imp::unsetenv(key)
        .unwrap_or_else(|e| panic!("failed to remove environment variable `{key:?}`: {e}"))
}

/// An iterator that splits an environment variable into paths according to
/// platform-specific conventions.
///
/// The iterator element type is [`PathBuf`].
///
/// This structure is created by [`env::split_paths()`]. See its
/// documentation for more.
///
/// [`env::split_paths()`]: split_paths
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct SplitPaths<'a> {
    inner: os_imp::SplitPaths<'a>,
}

/// Parses input according to platform conventions for the `PATH`
/// environment variable.
///
/// Returns an iterator over the paths contained in `unparsed`. The iterator
/// element type is [`PathBuf`].
///
/// # Examples
///
/// ```
/// use std::env;
///
/// let key = "PATH";
/// match env::var_os(key) {
///     Some(paths) => {
///         for path in env::split_paths(&paths) {
///             println!("'{}'", path.display());
///         }
///     }
///     None => println!("{key} is not defined in the environment.")
/// }
/// ```
pub fn split_paths<T: AsRef<OsStr> + ?Sized>(unparsed: &T) -> SplitPaths<'_> {
    SplitPaths { inner: os_imp::split_paths(unparsed.as_ref()) }
}

impl<'a> Iterator for SplitPaths<'a> {
    type Item = PathBuf;
    fn next(&mut self) -> Option<PathBuf> {
        self.inner.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl fmt::Debug for SplitPaths<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SplitPaths").finish_non_exhaustive()
    }
}

/// The error type for operations on the `PATH` variable. Possibly returned from
/// [`env::join_paths()`].
///
/// [`env::join_paths()`]: join_paths
#[derive(Debug)]
pub struct JoinPathsError {
    inner: os_imp::JoinPathsError,
}

/// Joins a collection of [`Path`]s appropriately for the `PATH`
/// environment variable.
///
/// # Errors
///
/// Returns an [`Err`] (containing an error message) if one of the input
/// [`Path`]s contains an invalid character for constructing the `PATH`
/// variable (a double quote on Windows or a colon on Unix).
///
/// # Examples
///
/// Joining paths on a Unix-like platform:
///
/// ```
/// use std::env;
/// use std::ffi::OsString;
/// use std::path::Path;
///
/// fn main() -> Result<(), env::JoinPathsError> {
/// # if cfg!(unix) {
///     let paths = [Path::new("/bin"), Path::new("/usr/bin")];
///     let path_os_string = env::join_paths(paths.iter())?;
///     assert_eq!(path_os_string, OsString::from("/bin:/usr/bin"));
/// # }
///     Ok(())
/// }
/// ```
///
/// Joining a path containing a colon on a Unix-like platform results in an
/// error:
///
/// ```
/// # if cfg!(unix) {
/// use std::env;
/// use std::path::Path;
///
/// let paths = [Path::new("/bin"), Path::new("/usr/bi:n")];
/// assert!(env::join_paths(paths.iter()).is_err());
/// # }
/// ```
///
/// Using `env::join_paths()` with [`env::split_paths()`] to append an item to
/// the `PATH` environment variable:
///
/// ```
/// use std::env;
/// use std::path::PathBuf;
///
/// fn main() -> Result<(), env::JoinPathsError> {
///     if let Some(path) = env::var_os("PATH") {
///         let mut paths = env::split_paths(&path).collect::<Vec<_>>();
///         paths.push(PathBuf::from("/home/xyz/bin"));
///         let new_path = env::join_paths(paths)?;
///         env::set_var("PATH", &new_path);
///     }
///
///     Ok(())
/// }
/// ```
///
/// [`env::split_paths()`]: split_paths
pub fn join_paths<I, T>(paths: I) -> Result<OsString, JoinPathsError>
where
    I: IntoIterator<Item = T>,
    T: AsRef<OsStr>,
{
    os_imp::join_paths(paths.into_iter()).map_err(|e| JoinPathsError { inner: e })
}

impl fmt::Display for JoinPathsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl Error for JoinPathsError {
    #[allow(deprecated, deprecated_in_future)]
    fn description(&self) -> &str {
        self.inner.description()
    }
}

/// Returns the path of the current user's home directory if known.
///
/// # Unix
///
/// - Returns the value of the 'HOME' environment variable if it is set
///   (including to an empty string).
/// - Otherwise, it tries to determine the home directory by invoking the `getpwuid_r` function
///   using the UID of the current user. An empty home directory field returned from the
///   `getpwuid_r` function is considered to be a valid value.
/// - Returns `None` if the current user has no entry in the /etc/passwd file.
///
/// # Windows
///
/// - Returns the value of the 'HOME' environment variable if it is set
///   (including to an empty string).
/// - Otherwise, returns the value of the 'USERPROFILE' environment variable if it is set
///   (including to an empty string).
/// - If both do not exist, [`GetUserProfileDirectory`][msdn] is used to return the path.
///
/// [msdn]: https://docs.microsoft.com/en-us/windows/win32/api/userenv/nf-userenv-getuserprofiledirectorya
///
/// # Deprecation
///
/// This function is deprecated because the behaviour on Windows is not correct.
/// The 'HOME' environment variable is not standard on Windows, and may not produce
/// desired results; for instance, under Cygwin or Mingw it will return `/home/you`
/// when it should return `C:\Users\you`.
///
/// # Examples
///
/// ```
/// use std::env;
///
/// match env::home_dir() {
///     Some(path) => println!("Your home directory, probably: {}", path.display()),
///     None => println!("Impossible to get your home dir!"),
/// }
/// ```
#[must_use]
pub fn home_dir() -> Option<PathBuf> {
    os_imp::home_dir()
}

/// Returns the path of a temporary directory.
///
/// The temporary directory may be shared among users, or between processes
/// with different privileges; thus, the creation of any files or directories
/// in the temporary directory must use a secure method to create a uniquely
/// named file. Creating a file or directory with a fixed or predictable name
/// may result in "insecure temporary file" security vulnerabilities. Consider
/// using a crate that securely creates temporary files or directories.
///
/// # Platform-specific behavior
///
/// On Unix, returns the value of the `TMPDIR` environment variable if it is
/// set, otherwise for non-Android it returns `/tmp`. On Android, since there
/// is no global temporary folder (it is usually allocated per-app), it returns
/// `/data/local/tmp`.
/// On Windows, the behavior is equivalent to that of [`GetTempPath2`][GetTempPath2] /
/// [`GetTempPath`][GetTempPath], which this function uses internally.
/// Note that, this [may change in the future][changes].
///
/// [changes]: io#platform-specific-behavior
/// [GetTempPath2]: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-gettemppath2a
/// [GetTempPath]: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-gettemppatha
///
/// ```no_run
/// use std::env;
///
/// fn main() {
///     let dir = env::temp_dir();
///     println!("Temporary directory: {}", dir.display());
/// }
/// ```
#[must_use]
pub fn temp_dir() -> PathBuf {
    os_imp::temp_dir()
}

/// Returns the full filesystem path of the current running executable.
///
/// # Platform-specific behavior
///
/// If the executable was invoked through a symbolic link, some platforms will
/// return the path of the symbolic link and other platforms will return the
/// path of the symbolic link’s target.
///
/// If the executable is renamed while it is running, platforms may return the
/// path at the time it was loaded instead of the new path.
///
/// # Errors
///
/// Acquiring the path of the current executable is a platform-specific operation
/// that can fail for a good number of reasons. Some errors can include, but not
/// be limited to, filesystem operations failing or general syscall failures.
///
/// # Security
///
/// The output of this function should not be trusted for anything
/// that might have security implications. Basically, if users can run
/// the executable, they can change the output arbitrarily.
///
/// As an example, you can easily introduce a race condition. It goes
/// like this:
///
/// 1. You get the path to the current executable using `current_exe()`, and
///    store it in a variable.
/// 2. Time passes. A malicious actor removes the current executable, and
///    replaces it with a malicious one.
/// 3. You then use the stored path to re-execute the current
///    executable.
///
/// You expected to safely execute the current executable, but you're
/// instead executing something completely different. The code you
/// just executed run with your privileges.
///
/// This sort of behavior has been known to [lead to privilege escalation] when
/// used incorrectly.
///
/// [lead to privilege escalation]: https://securityvulns.com/Wdocument183.html
///
/// # Examples
///
/// ```
/// use std::env;
///
/// match env::current_exe() {
///     Ok(exe_path) => println!("Path of this executable is: {}",
///                              exe_path.display()),
///     Err(e) => println!("failed to get current exe path: {e}"),
/// };
/// ```
#[cfg(feature = "env")]
pub fn current_exe() -> io::Result<PathBuf> {
    os_imp::current_exe()
}

/// An iterator over the arguments of a process, yielding a [`String`] value for
/// each argument.
///
/// This struct is created by [`env::args()`]. See its documentation
/// for more.
///
/// The first element is traditionally the path of the executable, but it can be
/// set to arbitrary text, and might not even exist. This means this property
/// should not be relied upon for security purposes.
///
/// [`env::args()`]: args
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Args {
    inner: ArgsOs,
}

/// An iterator over the arguments of a process, yielding an [`OsString`] value
/// for each argument.
///
/// This struct is created by [`env::args_os()`]. See its documentation
/// for more.
///
/// The first element is traditionally the path of the executable, but it can be
/// set to arbitrary text, and might not even exist. This means this property
/// should not be relied upon for security purposes.
///
/// [`env::args_os()`]: args_os
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct ArgsOs {
    inner: sys::args::Args,
}

/// Returns the arguments that this program was started with (normally passed
/// via the command line).
///
/// The first element is traditionally the path of the executable, but it can be
/// set to arbitrary text, and might not even exist. This means this property should
/// not be relied upon for security purposes.
///
/// On Unix systems the shell usually expands unquoted arguments with glob patterns
/// (such as `*` and `?`). On Windows this is not done, and such arguments are
/// passed as-is.
///
/// On glibc Linux systems, arguments are retrieved by placing a function in `.init_array`.
/// glibc passes `argc`, `argv`, and `envp` to functions in `.init_array`, as a non-standard
/// extension. This allows `std::env::args` to work even in a `cdylib` or `staticlib`, as it
/// does on macOS and Windows.
///
/// # Panics
///
/// The returned iterator will panic during iteration if any argument to the
/// process is not valid Unicode. If this is not desired,
/// use the [`args_os`] function instead.
///
/// # Examples
///
/// ```
/// use std::env;
///
/// // Prints each argument on a separate line
/// for argument in env::args() {
///     println!("{argument}");
/// }
/// ```
pub fn args() -> Args {
    Args { inner: args_os() }
}

/// Returns the arguments that this program was started with (normally passed
/// via the command line).
///
/// The first element is traditionally the path of the executable, but it can be
/// set to arbitrary text, and might not even exist. This means this property should
/// not be relied upon for security purposes.
///
/// On Unix systems the shell usually expands unquoted arguments with glob patterns
/// (such as `*` and `?`). On Windows this is not done, and such arguments are
/// passed as-is.
///
/// On glibc Linux systems, arguments are retrieved by placing a function in `.init_array`.
/// glibc passes `argc`, `argv`, and `envp` to functions in `.init_array`, as a non-standard
/// extension. This allows `std::env::args_os` to work even in a `cdylib` or `staticlib`, as it
/// does on macOS and Windows.
///
/// Note that the returned iterator will not check if the arguments to the
/// process are valid Unicode. If you want to panic on invalid UTF-8,
/// use the [`args`] function instead.
///
/// # Examples
///
/// ```
/// use std::env;
///
/// // Prints each argument on a separate line
/// for argument in env::args_os() {
///     println!("{argument:?}");
/// }
/// ```
pub fn args_os() -> ArgsOs {
    ArgsOs { inner: sys::args::args() }
}

impl !Send for Args {}

impl !Sync for Args {}

impl Iterator for Args {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        self.inner.next().map(|s| s.into_string().unwrap())
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for Args {
    fn len(&self) -> usize {
        self.inner.len()
    }
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl DoubleEndedIterator for Args {
    fn next_back(&mut self) -> Option<String> {
        self.inner.next_back().map(|s| s.into_string().unwrap())
    }
}

impl fmt::Debug for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner: ArgsOs { inner } } = self;
        f.debug_struct("Args").field("inner", inner).finish()
    }
}

impl !Send for ArgsOs {}

impl !Sync for ArgsOs {}

impl Iterator for ArgsOs {
    type Item = OsString;
    fn next(&mut self) -> Option<OsString> {
        self.inner.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for ArgsOs {
    fn len(&self) -> usize {
        self.inner.len()
    }
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl DoubleEndedIterator for ArgsOs {
    fn next_back(&mut self) -> Option<OsString> {
        self.inner.next_back()
    }
}

impl fmt::Debug for ArgsOs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        f.debug_struct("ArgsOs").field("inner", inner).finish()
    }
}

/// Constants associated with the current target
pub mod consts {
    use crate::sys::env::os;

    /// A string describing the architecture of the CPU that is currently
    /// in use.
    ///
    /// Some possible values:
    ///
    /// - x86
    /// - x86_64
    /// - arm
    /// - aarch64
    /// - loongarch64
    /// - m68k
    /// - csky
    /// - mips
    /// - mips64
    /// - powerpc
    /// - powerpc64
    /// - riscv64
    /// - s390x
    /// - sparc64
    pub const ARCH: &str = super::arch::ARCH;

    /// The family of the operating system. Example value is `unix`.
    ///
    /// Some possible values:
    ///
    /// - unix
    /// - windows
    pub const FAMILY: &str = os::FAMILY;

    /// A string describing the specific operating system in use.
    /// Example value is `linux`.
    ///
    /// Some possible values:
    ///
    /// - linux
    /// - macos
    /// - ios
    /// - freebsd
    /// - dragonfly
    /// - netbsd
    /// - openbsd
    /// - solaris
    /// - android
    /// - windows
    pub const OS: &str = os::OS;

    /// Specifies the filename prefix used for shared libraries on this
    /// platform. Example value is `lib`.
    ///
    /// Some possible values:
    ///
    /// - lib
    /// - `""` (an empty string)
    pub const DLL_PREFIX: &str = os::DLL_PREFIX;

    /// Specifies the filename suffix used for shared libraries on this
    /// platform. Example value is `.so`.
    ///
    /// Some possible values:
    ///
    /// - .so
    /// - .dylib
    /// - .dll
    pub const DLL_SUFFIX: &str = os::DLL_SUFFIX;

    /// Specifies the file extension used for shared libraries on this
    /// platform that goes after the dot. Example value is `so`.
    ///
    /// Some possible values:
    ///
    /// - so
    /// - dylib
    /// - dll
    pub const DLL_EXTENSION: &str = os::DLL_EXTENSION;

    /// Specifies the filename suffix used for executable binaries on this
    /// platform. Example value is `.exe`.
    ///
    /// Some possible values:
    ///
    /// - .exe
    /// - .nexe
    /// - .pexe
    /// - `""` (an empty string)
    pub const EXE_SUFFIX: &str = os::EXE_SUFFIX;

    /// Specifies the file extension, if any, used for executable binaries
    /// on this platform. Example value is `exe`.
    ///
    /// Some possible values:
    ///
    /// - exe
    /// - `""` (an empty string)
    pub const EXE_EXTENSION: &str = os::EXE_EXTENSION;
}

#[cfg(target_arch = "x86")]
mod arch {
    pub const ARCH: &str = "x86";
}

#[cfg(target_arch = "x86_64")]
mod arch {
    pub const ARCH: &str = "x86_64";
}
