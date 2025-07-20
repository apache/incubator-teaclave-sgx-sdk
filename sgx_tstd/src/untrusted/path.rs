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

use crate::fs;
use crate::io;
use crate::path::{self, Path, PathBuf};

pub trait PathEx {
    fn metadata(&self) -> io::Result<fs::Metadata>;
    fn symlink_metadata(&self) -> io::Result<fs::Metadata>;
    fn canonicalize(&self) -> io::Result<PathBuf>;
    fn read_link(&self) -> io::Result<PathBuf>;
    fn read_dir(&self) -> io::Result<fs::ReadDir>;
    fn exists(&self) -> bool;
    fn try_exists(&self) -> io::Result<bool>;
    fn is_file(&self) -> bool;
    fn is_dir(&self) -> bool;
    fn is_symlink(&self) -> bool;
}

impl PathEx for Path {
    /// Queries the file system to get information about a file, directory, etc.
    ///
    /// This function will traverse symbolic links to query information about the
    /// destination file.
    ///
    /// This is an alias to [`fs::metadata`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use std::untrusted::path::PathEx;
    ///
    /// let path = Path::new("/Minas/tirith");
    /// let metadata = path.metadata().expect("metadata call failed");
    /// println!("{:?}", metadata.file_type());
    /// ```
    #[inline]
    fn metadata(&self) -> io::Result<fs::Metadata> {
        fs::metadata(self)
    }

    /// Queries the metadata about a file without following symlinks.
    ///
    /// This is an alias to [`fs::symlink_metadata`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use std::untrusted::path::PathEx;
    ///
    /// let path = Path::new("/Minas/tirith");
    /// let metadata = path.symlink_metadata().expect("symlink_metadata call failed");
    /// println!("{:?}", metadata.file_type());
    /// ```
    #[inline]
    fn symlink_metadata(&self) -> io::Result<fs::Metadata> {
        fs::symlink_metadata(self)
    }

    /// Returns the canonical, absolute form of the path with all intermediate
    /// components normalized and symbolic links resolved.
    ///
    /// This is an alias to [`fs::canonicalize`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::{Path, PathBuf};
    /// use std::untrusted::path::PathEx;
    ///
    /// let path = Path::new("/foo/test/../test/bar.rs");
    /// assert_eq!(path.canonicalize().unwrap(), PathBuf::from("/foo/test/bar.rs"));
    /// ```
    #[inline]
    fn canonicalize(&self) -> io::Result<PathBuf> {
        fs::canonicalize(self)
    }

    /// Reads a symbolic link, returning the file that the link points to.
    ///
    /// This is an alias to [`fs::read_link`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use std::untrusted::path::PathEx;
    ///
    /// let path = Path::new("/laputa/sky_castle.rs");
    /// let path_link = path.read_link().expect("read_link call failed");
    /// ```
    #[inline]
    fn read_link(&self) -> io::Result<PathBuf> {
        fs::read_link(self)
    }

    /// Returns an iterator over the entries within a directory.
    ///
    /// The iterator will yield instances of [`io::Result`]`<`[`fs::DirEntry`]`>`. New
    /// errors may be encountered after an iterator is initially constructed.
    ///
    /// This is an alias to [`fs::read_dir`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use std::untrusted::path::PathEx;
    ///
    /// let path = Path::new("/laputa");
    /// for entry in path.read_dir().expect("read_dir call failed") {
    ///     if let Ok(entry) = entry {
    ///         println!("{:?}", entry.path());
    ///     }
    /// }
    /// ```
    #[inline]
    fn read_dir(&self) -> io::Result<fs::ReadDir> {
        fs::read_dir(self)
    }

    /// Returns `true` if the path points at an existing entity.
    ///
    /// This function will traverse symbolic links to query information about the
    /// destination file.
    ///
    /// If you cannot access the metadata of the file, e.g. because of a
    /// permission error or broken symbolic links, this will return `false`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use std::untrusted::path::PathEx;
    /// assert!(!Path::new("does_not_exist.txt").exists());
    /// ```
    ///
    /// # See Also
    ///
    /// This is a convenience function that coerces errors to false. If you want to
    /// check errors, call [`fs::metadata`].
    #[must_use]
    #[inline]
    fn exists(&self) -> bool {
        fs::metadata(self).is_ok()
    }

    /// Returns `Ok(true)` if the path points at an existing entity.
    ///
    /// This function will traverse symbolic links to query information about the
    /// destination file. In case of broken symbolic links this will return `Ok(false)`.
    ///
    /// As opposed to the `exists()` method, this one doesn't silently ignore errors
    /// unrelated to the path not existing. (E.g. it will return `Err(_)` in case of permission
    /// denied on some of the parent directories.)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// #![feature(path_try_exists)]
    ///
    /// use std::path::Path;
    /// use std::untrusted::path::PathEx;
    /// assert!(!Path::new("does_not_exist.txt").try_exists().expect("Can't check existence of file does_not_exist.txt"));
    /// assert!(Path::new("/root/secret_file.txt").try_exists().is_err());
    /// ```
    // FIXME: stabilization should modify documentation of `exists()` to recommend this method
    // instead.
    #[inline]
    fn try_exists(&self) -> io::Result<bool> {
        fs::try_exists(self)
    }

    /// Returns `true` if the path exists on disk and is pointing at a regular file.
    ///
    /// This function will traverse symbolic links to query information about the
    /// destination file.
    ///
    /// If you cannot access the metadata of the file, e.g. because of a
    /// permission error or broken symbolic links, this will return `false`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use std::untrusted::path::PathEx;
    /// assert_eq!(Path::new("./is_a_directory/").is_file(), false);
    /// assert_eq!(Path::new("a_file.txt").is_file(), true);
    /// ```
    ///
    /// # See Also
    ///
    /// This is a convenience function that coerces errors to false. If you want to
    /// check errors, call [`fs::metadata`] and handle its [`Result`]. Then call
    /// [`fs::Metadata::is_file`] if it was [`Ok`].
    ///
    /// When the goal is simply to read from (or write to) the source, the most
    /// reliable way to test the source can be read (or written to) is to open
    /// it. Only using `is_file` can break workflows like `diff <( prog_a )` on
    /// a Unix-like system for example. See [`fs::File::open`] or
    /// [`fs::OpenOptions::open`] for more information.
    #[must_use]
    fn is_file(&self) -> bool {
        fs::metadata(self).map(|m| m.is_file()).unwrap_or(false)
    }

    /// Returns `true` if the path exists on disk and is pointing at a directory.
    ///
    /// This function will traverse symbolic links to query information about the
    /// destination file.
    ///
    /// If you cannot access the metadata of the file, e.g. because of a
    /// permission error or broken symbolic links, this will return `false`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use std::untrusted::path::PathEx;
    /// assert_eq!(Path::new("./is_a_directory/").is_dir(), true);
    /// assert_eq!(Path::new("a_file.txt").is_dir(), false);
    /// ```
    ///
    /// # See Also
    ///
    /// This is a convenience function that coerces errors to false. If you want to
    /// check errors, call [`fs::metadata`] and handle its [`Result`]. Then call
    /// [`fs::Metadata::is_dir`] if it was [`Ok`].
    #[must_use]
    fn is_dir(&self) -> bool {
        fs::metadata(self).map(|m| m.is_dir()).unwrap_or(false)
    }

    /// Returns true if the path exists on disk and is pointing at a symbolic link.
    ///
    /// This function will not traverse symbolic links.
    /// In case of a broken symbolic link this will also return true.
    ///
    /// If you cannot access the directory containing the file, e.g., because of a
    /// permission error, this will return false.
    ///
    /// # Examples
    ///
    #[cfg_attr(unix, doc = "```no_run")]
    #[cfg_attr(not(unix), doc = "```ignore")]
    /// use std::path::Path;
    /// use std::os::unix::fs::symlink;
    /// use std::untrusted::path::PathEx;
    ///
    /// let link_path = Path::new("link");
    /// symlink("/origin_does_not_exists/", link_path).unwrap();
    /// assert_eq!(link_path.is_symlink(), true);
    /// assert_eq!(link_path.exists(), false);
    /// ```
    #[must_use]
    fn is_symlink(&self) -> bool {
        fs::symlink_metadata(self).map(|m| m.is_symlink()).unwrap_or(false)
    }
}

/// Makes the path absolute without accessing the filesystem.
///
/// If the path is relative, the current directory is used as the base directory.
/// All intermediate components will be resolved according to platforms-specific
/// rules but unlike [`canonicalize`][crate::fs::canonicalize] this does not
/// resolve symlinks and may succeed even if the path does not exist.
///
/// If the `path` is empty or getting the
/// [current directory][crate::env::current_dir] fails then an error will be
/// returned.
///
/// # Examples
///
/// ## Posix paths
///
/// ```
/// #![feature(absolute_path)]
/// # #[cfg(unix)]
/// fn main() -> std::io::Result<()> {
///   use std::path::{self, Path};
///
///   // Relative to absolute
///   let absolute = path::absolute("foo/./bar")?;
///   assert!(absolute.ends_with("foo/bar"));
///
///   // Absolute to absolute
///   let absolute = path::absolute("/foo//test/.././bar.rs")?;
///   assert_eq!(absolute, Path::new("/foo/test/../bar.rs"));
///   Ok(())
/// }
/// # #[cfg(not(unix))]
/// # fn main() {}
/// ```
///
/// The path is resolved using [POSIX semantics][posix-semantics] except that
/// it stops short of resolving symlinks. This means it will keep `..`
/// components and trailing slashes.
///
/// ## Windows paths
///
/// ```
/// #![feature(absolute_path)]
/// # #[cfg(windows)]
/// fn main() -> std::io::Result<()> {
///   use std::path::{self, Path};
///
///   // Relative to absolute
///   let absolute = path::absolute("foo/./bar")?;
///   assert!(absolute.ends_with(r"foo\bar"));
///
///   // Absolute to absolute
///   let absolute = path::absolute(r"C:\foo//test\..\./bar.rs")?;
///
///   assert_eq!(absolute, Path::new(r"C:\foo\bar.rs"));
///   Ok(())
/// }
/// # #[cfg(not(windows))]
/// # fn main() {}
/// ```
///
/// For verbatim paths this will simply return the path as given. For other
/// paths this is currently equivalent to calling [`GetFullPathNameW`][windows-path]
/// This may change in the future.
///
/// [posix-semantics]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap04.html#tag_04_13
/// [windows-path]: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfullpathnamew
pub fn absolute<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    path::_absolute(path)
}
