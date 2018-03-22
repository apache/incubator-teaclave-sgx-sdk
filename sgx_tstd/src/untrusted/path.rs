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

use untrusted::fs;
use io;
use path::Path;
use path::PathBuf;

pub trait PathEx {
    fn metadata(&self) -> io::Result<fs::Metadata>;
    fn symlink_metadata(&self) -> io::Result<fs::Metadata>;
    fn canonicalize(&self) -> io::Result<PathBuf>;
    fn read_link(&self) -> io::Result<PathBuf>;
    fn exists(&self) -> bool;
    fn is_file(&self) -> bool;
    fn is_dir(&self) -> bool;
}

impl PathEx for Path {
    /// Queries the file system to get information about a file, directory, etc.
    ///
    /// This function will traverse symbolic links to query information about the
    /// destination file.
    ///
    /// This is an alias to [`fs::metadata`].
    ///
    /// [`fs::metadata`]: ../fs/fn.metadata.html
    ///
    fn metadata(&self) -> io::Result<fs::Metadata> {
        fs::metadata(self)
    }

    /// Queries the metadata about a file without following symlinks.
    ///
    /// This is an alias to [`fs::symlink_metadata`].
    ///
    /// [`fs::symlink_metadata`]: ../fs/fn.symlink_metadata.html
    ///
    fn symlink_metadata(&self) -> io::Result<fs::Metadata> {
        fs::symlink_metadata(self)
    }

    /// Returns the canonical form of the path with all intermediate components
    /// normalized and symbolic links resolved.
    ///
    /// This is an alias to [`fs::canonicalize`].
    ///
    /// [`fs::canonicalize`]: ../fs/fn.canonicalize.html
    ///
    fn canonicalize(&self) -> io::Result<PathBuf> {
        fs::canonicalize(self)
    }

    /// Reads a symbolic link, returning the file that the link points to.
    ///
    /// This is an alias to [`fs::read_link`].
    ///
    /// [`fs::read_link`]: ../fs/fn.read_link.html
    ///
    fn read_link(&self) -> io::Result<PathBuf> {
        fs::read_link(self)
    }

    /// Returns whether the path points at an existing entity.
    ///
    /// This function will traverse symbolic links to query information about the
    /// destination file. In case of broken symbolic links this will return `false`.
    ///
    /// If you cannot access the directory containing the file, e.g. because of a
    /// permission error, this will return `false`.
    ///
    fn exists(&self) -> bool {
        fs::metadata(self).is_ok()
    }

    /// Returns whether the path exists on disk and is pointing at a regular file.
    ///
    /// This function will traverse symbolic links to query information about the
    /// destination file. In case of broken symbolic links this will return `false`.
    ///
    /// If you cannot access the directory containing the file, e.g. because of a
    /// permission error, this will return `false`.
    ///
    fn is_file(&self) -> bool {
        fs::metadata(self).map(|m| m.is_file()).unwrap_or(false)
    }

    /// Returns whether the path exists on disk and is pointing at a directory.
    ///
    /// This function will traverse symbolic links to query information about the
    /// destination file. In case of broken symbolic links this will return `false`.
    ///
    /// If you cannot access the directory containing the file, e.g. because of a
    /// permission error, this will return `false`.
    ///
    fn is_dir(&self) -> bool {
        fs::metadata(self).map(|m| m.is_dir()).unwrap_or(false)
    }
}
