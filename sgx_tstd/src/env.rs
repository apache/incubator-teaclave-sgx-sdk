// Copyright (c) 2017 Baidu, Inc. All Rights Reserved.
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

use error::Error;
use ffi::{OsStr, OsString};
use path::PathBuf;
use sys::os as os_imp;
use core::fmt;

/// An iterator that splits an environment variable into paths according to
/// platform-specific conventions.
///
pub struct SplitPaths<'a> { inner: os_imp::SplitPaths<'a> }

/// Parses input according to platform conventions for the `PATH`
/// environment variable.
///
/// Returns an iterator over the paths contained in `unparsed`.
///
pub fn split_paths<T: AsRef<OsStr> + ?Sized>(unparsed: &T) -> SplitPaths {
    SplitPaths { inner: os_imp::split_paths(unparsed.as_ref()) }
}

impl<'a> Iterator for SplitPaths<'a> {
    type Item = PathBuf;
    fn next(&mut self) -> Option<PathBuf> { self.inner.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}

impl<'a> fmt::Debug for SplitPaths<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("SplitPaths { .. }")
    }
}

/// The error type for operations on the `PATH` variable. Possibly returned from
/// the [`env::join_paths`] function.
///
#[derive(Debug)]
pub struct JoinPathsError {
    inner: os_imp::JoinPathsError
}

/// Joins a collection of [`Path`]s appropriately for the `PATH`
/// environment variable.
///
/// # Errors
///
/// Returns an [`Err`][err] (containing an error message) if one of the input
/// [`Path`]s contains an invalid character for constructing the `PATH`
/// variable (a double quote on Windows or a colon on Unix).
///
/// [`Path`]: ../../std/path/struct.Path.html
/// [`OsString`]: ../../std/ffi/struct.OsString.html
/// [err]: ../../std/result/enum.Result.html#variant.Err
///
pub fn join_paths<I, T>(paths: I) -> Result<OsString, JoinPathsError>
    where I: IntoIterator<Item=T>, T: AsRef<OsStr>
{
    os_imp::join_paths(paths.into_iter()).map_err(|e| {
        JoinPathsError { inner: e }
    })
}

impl fmt::Display for JoinPathsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl Error for JoinPathsError {
    fn description(&self) -> &str { self.inner.description() }
}