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

use sgx_trts::libc::{c_int, mode_t, time_t, stat64, off64_t};
use os::unix::prelude::*;
use ffi::{CString, CStr, OsString};
use io::{self, Error, ErrorKind, SeekFrom};
use path::{Path, PathBuf};
use sys::fd::FileDesc;
use sys::time::SystemTime;
use sys::{cvt, cvt_r};
use sys_common::{AsInner, FromInner};
use core::fmt;
use core::mem;

pub struct File(FileDesc);

#[derive(Clone)]
pub struct FileAttr {
    stat: stat64,
}

#[derive(Clone, Debug)]
pub struct OpenOptions {
    // generic
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
    // system-specific
    custom_flags: i32,
    mode: mode_t,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FilePermissions { mode: mode_t }

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct FileType { mode: mode_t }

impl FileAttr {
    pub fn size(&self) -> u64 { self.stat.st_size as u64 }
    pub fn perm(&self) -> FilePermissions {
        FilePermissions { mode: (self.stat.st_mode as mode_t) }
    }

    pub fn file_type(&self) -> FileType {
        FileType { mode: self.stat.st_mode as mode_t }
    }
}

impl FileAttr {
    pub fn modified(&self) -> io::Result<SystemTime> {
        Ok(SystemTime::from(libc::timespec {
            tv_sec: self.stat.st_mtime as time_t,
            tv_nsec: self.stat.st_mtime_nsec as _,
        }))
    }

    pub fn accessed(&self) -> io::Result<SystemTime> {
        Ok(SystemTime::from(libc::timespec {
            tv_sec: self.stat.st_atime as time_t,
            tv_nsec: self.stat.st_atime_nsec as _,
        }))
    }

    pub fn created(&self) -> io::Result<SystemTime> {
        Err(io::Error::new(io::ErrorKind::Other,
                           "creation time is not available on this platform \
                            currently"))
    }
}

impl AsInner<stat64> for FileAttr {
    fn as_inner(&self) -> &stat64 { &self.stat }
}

impl FilePermissions {
    pub fn readonly(&self) -> bool {
        // check if any class (owner, group, others) has write permission
        self.mode & 0o222 == 0
    }

    pub fn set_readonly(&mut self, readonly: bool) {
        if readonly {
            // remove write permission for all classes; equivalent to `chmod a-w <file>`
            self.mode &= !0o222;
        } else {
            // add write permission for all classes; equivalent to `chmod a+w <file>`
            self.mode |= 0o222;
        }
    }
    pub fn mode(&self) -> u32 { self.mode as u32 }
}

impl FileType {
    pub fn is_dir(&self) -> bool { self.is(libc::S_IFDIR) }
    pub fn is_file(&self) -> bool { self.is(libc::S_IFREG) }
    pub fn is_symlink(&self) -> bool { self.is(libc::S_IFLNK) }

    pub fn is(&self, mode: mode_t) -> bool { self.mode & libc::S_IFMT == mode }
}

impl FromInner<u32> for FilePermissions {
    fn from_inner(mode: u32) -> FilePermissions {
        FilePermissions { mode: mode as mode_t }
    }
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions {
            // generic
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
            // system-specific
            custom_flags: 0,
            mode: 0o666,
        }
    }

    pub fn read(&mut self, read: bool) { self.read = read; }
    pub fn write(&mut self, write: bool) { self.write = write; }
    pub fn append(&mut self, append: bool) { self.append = append; }
    pub fn truncate(&mut self, truncate: bool) { self.truncate = truncate; }
    pub fn create(&mut self, create: bool) { self.create = create; }
    pub fn create_new(&mut self, create_new: bool) { self.create_new = create_new; }

    pub fn custom_flags(&mut self, flags: i32) { self.custom_flags = flags; }
    pub fn mode(&mut self, mode: u32) { self.mode = mode as mode_t; }

    fn get_access_mode(&self) -> io::Result<c_int> {
        match (self.read, self.write, self.append) {
            (true,  false, false) => Ok(libc::O_RDONLY),
            (false, true,  false) => Ok(libc::O_WRONLY),
            (true,  true,  false) => Ok(libc::O_RDWR),
            (false, _,     true)  => Ok(libc::O_WRONLY | libc::O_APPEND),
            (true,  _,     true)  => Ok(libc::O_RDWR | libc::O_APPEND),
            (false, false, false) => Err(Error::from_raw_os_error(libc::EINVAL)),
        }
    }

    fn get_creation_mode(&self) -> io::Result<c_int> {
        match (self.write, self.append) {
            (true, false) => {}
            (false, false) =>
                if self.truncate || self.create || self.create_new {
                    return Err(Error::from_raw_os_error(libc::EINVAL));
                },
            (_, true) =>
                if self.truncate && !self.create_new {
                    return Err(Error::from_raw_os_error(libc::EINVAL));
                },
        }

        Ok(match (self.create, self.truncate, self.create_new) {
                (false, false, false) => 0,
                (true,  false, false) => libc::O_CREAT,
                (false, true,  false) => libc::O_TRUNC,
                (true,  true,  false) => libc::O_CREAT | libc::O_TRUNC,
                (_,      _,    true)  => libc::O_CREAT | libc::O_EXCL,
           })
    }
}

impl File {
    pub fn open(path: &Path, opts: &OpenOptions) -> io::Result<File> {
        let path = cstr(path)?;
        File::open_c(&path, opts)
    }

    pub fn open_c(path: &CStr, opts: &OpenOptions) -> io::Result<File> {
        let flags = libc::O_CLOEXEC |
                    opts.get_access_mode()? |
                    opts.get_creation_mode()? |
                    (opts.custom_flags as c_int & !libc::O_ACCMODE);
        let fd = cvt_r(|| unsafe {
            libc::open64(path.as_ptr(), flags, opts.mode as c_int)
        })?;
        let fd = FileDesc::new(fd);

        // Currently the standard library supports Linux 2.6.18 which did not
        // have the O_CLOEXEC flag (passed above). If we're running on an older
        // Linux kernel then the flag is just ignored by the OS, so we continue
        // to explicitly ask for a CLOEXEC fd here.
        //
        // The CLOEXEC flag, however, is supported on versions of macOS/BSD/etc
        // that we support, so we only do this on Linux currently.
        if cfg!(target_os = "linux") {
            fd.set_cloexec()?;
        }

        Ok(File(fd))
    }

    pub fn file_attr(&self) -> io::Result<FileAttr> {
        let mut stat: stat64 = unsafe { mem::zeroed() };
        cvt(unsafe {
            libc::fstat64(self.0.raw(), &mut stat)
        })?;
        Ok(FileAttr { stat: stat })
    }

    pub fn fsync(&self) -> io::Result<()> {
        cvt_r(|| unsafe { libc::fsync(self.0.raw()) })?;
        Ok(())
    }

    pub fn datasync(&self) -> io::Result<()> {
        cvt_r(|| unsafe { os_datasync(self.0.raw()) })?;
        return Ok(());

        unsafe fn os_datasync(fd: c_int) -> c_int { libc::fdatasync(fd) }
    }

    pub fn truncate(&self, size: u64) -> io::Result<()> {
        return cvt_r(|| unsafe {
            libc::ftruncate64(self.0.raw(), size as off64_t)
        }).map(|_| ());
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    pub fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        self.0.read_at(buf, offset)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    pub fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        self.0.write_at(buf, offset)
    }

    pub fn flush(&self) -> io::Result<()> { Ok(()) }

    pub fn seek(&self, pos: SeekFrom) -> io::Result<u64> {
        let (whence, pos) = match pos {
            // Casting to `i64` is fine, too large values will end up as
            // negative which will cause an error in `lseek64`.
            SeekFrom::Start(off) => (libc::SEEK_SET, off as i64),
            SeekFrom::End(off) => (libc::SEEK_END, off),
            SeekFrom::Current(off) => (libc::SEEK_CUR, off),
        };

        let n = cvt(unsafe { libc::lseek64(self.0.raw(), pos, whence) })?;
        Ok(n as u64)
    }

    pub fn duplicate(&self) -> io::Result<File> {
        self.0.duplicate().map(File)
    }

    pub fn fd(&self) -> &FileDesc { &self.0 }

    pub fn into_fd(self) -> FileDesc { self.0 }

    pub fn set_permissions(&self, perm: FilePermissions) -> io::Result<()> {
        cvt_r(|| unsafe { libc::fchmod(self.0.raw(), perm.mode) })?;
        Ok(())
    }
}

fn cstr(path: &Path) -> io::Result<CString> {
    Ok(CString::new(path.as_os_str().as_bytes())?)
}

impl FromInner<c_int> for File {
    fn from_inner(fd: c_int) -> File {
        File(FileDesc::new(fd))
    }
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {  
        fn get_path(fd: c_int) -> Option<PathBuf> {
            let mut p = PathBuf::from("/proc/self/fd");
            p.push(&fd.to_string());
            readlink(&p).ok()
        }

        fn get_mode(fd: c_int) -> Option<(bool, bool)> {
            let mode = unsafe { libc::fcntl_arg0(fd, libc::F_GETFL) };
            if mode == -1 {
                return None;
            }
            match mode & libc::O_ACCMODE {
                libc::O_RDONLY => Some((true, false)),
                libc::O_RDWR => Some((true, true)),
                libc::O_WRONLY => Some((false, true)),
                _ => None
            }
        }

        let fd = self.0.raw();
        let mut b = f.debug_struct("File");
        b.field("fd", &fd);
        if let Some(path) = get_path(fd) {
            b.field("path", &path);
        }
        if let Some((read, write)) = get_mode(fd) {
            b.field("read", &read).field("write", &write);
        }
        b.finish()
    }
}

pub fn unlink(p: &Path) -> io::Result<()> {
    let p = cstr(p)?;
    cvt(unsafe { libc::unlink(p.as_ptr()) })?;
    Ok(())
}

pub fn rename(old: &Path, new: &Path) -> io::Result<()> {
    let old = cstr(old)?;
    let new = cstr(new)?;
    cvt(unsafe { libc::rename(old.as_ptr(), new.as_ptr()) })?;
    Ok(())
}

pub fn set_perm(p: &Path, perm: FilePermissions) -> io::Result<()> {
    let p = cstr(p)?;
    cvt_r(|| unsafe { libc::chmod(p.as_ptr(), perm.mode) })?;
    Ok(())
}

pub fn readlink(p: &Path) -> io::Result<PathBuf> {
    let c_path = cstr(p)?;
    let p = c_path.as_ptr();

    let mut buf = Vec::with_capacity(256);

    loop {
        let buf_read = cvt(unsafe {
            libc::readlink(p, buf.as_mut_ptr() as *mut _, buf.capacity())
        })? as usize;

        unsafe { buf.set_len(buf_read); }

        if buf_read != buf.capacity() {
            buf.shrink_to_fit();

            return Ok(PathBuf::from(OsString::from_vec(buf)));
        }

        // Trigger the internal buffer resizing logic of `Vec` by requiring
        // more space than the current capacity. The length is guaranteed to be
        // the same as the capacity due to the if statement above.
        buf.reserve(1);
    }
}

pub fn symlink(src: &Path, dst: &Path) -> io::Result<()> {
    let src = cstr(src)?;
    let dst = cstr(dst)?;
    cvt(unsafe { libc::symlink(src.as_ptr(), dst.as_ptr()) })?;
    Ok(())
}

pub fn link(src: &Path, dst: &Path) -> io::Result<()> {
    let src = cstr(src)?;
    let dst = cstr(dst)?;
    cvt(unsafe { libc::link(src.as_ptr(), dst.as_ptr()) })?;
    Ok(())
}

pub fn stat(p: &Path) -> io::Result<FileAttr> {
    let p = cstr(p)?;
    let mut stat: stat64 = unsafe { mem::zeroed() };
    cvt(unsafe {
        libc::stat64(p.as_ptr(), &mut stat as *mut _)
    })?;
    Ok(FileAttr { stat: stat })
}

pub fn lstat(p: &Path) -> io::Result<FileAttr> {
    let p = cstr(p)?;
    let mut stat: stat64 = unsafe { mem::zeroed() };
    cvt(unsafe {
        libc::lstat64(p.as_ptr(), &mut stat as *mut _)
    })?;
    Ok(FileAttr { stat: stat })
}

pub fn canonicalize(p: &Path) -> io::Result<PathBuf> {
    let path = CString::new(p.as_os_str().as_bytes())?;
    let buf;
    unsafe {
        let r = libc::realpath(path.as_ptr());
        if r.is_null() {
            return Err(io::Error::last_os_error())
        }
        buf = CStr::from_ptr(r).to_bytes().to_vec();
        libc::free(r as *mut _);
    }
    Ok(PathBuf::from(OsString::from_vec(buf)))
}

pub fn copy(from: &Path, to: &Path) -> io::Result<u64> {
    use fs::{File, set_permissions};
    if !from.is_file() {
        return Err(Error::new(ErrorKind::InvalidInput,
                              "the source path is not an existing regular file"))
    }

    let mut reader = File::open(from)?;
    let mut writer = File::create(to)?;
    let perm = reader.metadata()?.permissions();

    let ret = io::copy(&mut reader, &mut writer)?;
    set_permissions(to, perm)?;
    Ok(ret)
}

mod libc {
    use sgx_types::sgx_status_t;
    use io;
    use core::ptr;
    pub use sgx_trts::libc::*;

    extern "C" {

        pub fn u_fs_open64_ocall(result: * mut c_int,
                                 error: * mut c_int,
                                 path: * const c_char,
                                 oflag: c_int,
                                 mode: c_int) -> sgx_status_t;
        
        pub fn u_fs_fstat64_ocall(result: * mut c_int,
                                  error: * mut c_int,
                                  fd: c_int,
                                  buf: * mut stat64) -> sgx_status_t;

        pub fn u_fs_fsync_ocall(result: * mut c_int,
                                error: * mut c_int,
                                fd: c_int) -> sgx_status_t;

        pub fn u_fs_fdatasync_ocall(result: * mut c_int,
                                    error: * mut c_int,
                                    fd: c_int) -> sgx_status_t;

        pub fn u_fs_ftruncate64_ocall(result: * mut c_int,
                                      error: * mut c_int,
                                      fd: c_int,
                                      length: off64_t) -> sgx_status_t;

        pub fn u_fs_lseek64_ocall(result: * mut off64_t,
                                  error: * mut c_int,
                                  fd: c_int,
                                  offset: off64_t,
                                  whence: c_int) -> sgx_status_t;

        pub fn u_fs_fchmod_ocall(result: * mut c_int,
                                 error: * mut c_int,
                                 fd: c_int,
                                 mode: mode_t) -> sgx_status_t;

        pub fn u_fs_unlink_ocall(result: * mut c_int,
                                 error: * mut c_int,
                                 pathname: * const c_char) -> sgx_status_t;

        pub fn u_fs_link_ocall(result: * mut c_int,
                               error: * mut c_int,
                               oldpath: * const c_char,
                               newpath: * const c_char) -> sgx_status_t;

        pub fn u_fs_rename_ocall(result: * mut c_int,
                                 error: * mut c_int,
                                 oldpath: * const c_char,
                                 newpath: * const c_char) -> sgx_status_t;

        pub fn u_fs_chmod_ocall(result: * mut c_int,
                                error: * mut c_int,
                                path: * const c_char,
                                mode: mode_t) -> sgx_status_t;

        pub fn u_fs_readlink_ocall(result: * mut ssize_t,
                                   error: * mut c_int,
                                   path: * const c_char,
                                   buf: * mut c_char,
                                   bufsz: size_t) -> sgx_status_t;

        pub fn u_fs_symlink_ocall(result: * mut c_int,
                                  error: * mut c_int,
                                  path1: * const c_char,
                                  path2: * const c_char) -> sgx_status_t;

        pub fn u_fs_stat64_ocall(result: * mut c_int,
                                 error: * mut c_int,
                                 path: * const c_char,
                                 buf: * mut stat64) -> sgx_status_t;

        pub fn u_fs_lstat64_ocall(result: * mut c_int,
                                  error: * mut c_int,
                                  path: * const c_char,
                                  buf: * mut stat64) -> sgx_status_t;

        pub fn u_fs_fcntl_arg0_ocall(result: * mut c_int,
                                     errno: * mut c_int,
                                     fd: c_int,
                                     cmd: c_int) -> sgx_status_t;

        pub fn u_fs_realpath_ocall(result: * mut * mut c_char,
                                   error: * mut c_int,
                                   pathname: * const c_char) -> sgx_status_t;

        pub fn u_fs_free_ocall(p: * mut c_void) -> sgx_status_t;
    }

    pub unsafe fn open64(path: * const c_char, oflag: c_int, mode: c_int) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_fs_open64_ocall(&mut result as * mut c_int,
                                       &mut error as * mut c_int,
                                       path,
                                       oflag,
                                       mode);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn fstat64(fd: c_int, buf: * mut stat64) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_fs_fstat64_ocall(&mut result as * mut c_int,
                                        &mut error as * mut c_int,
                                        fd,
                                        buf);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn fsync(fd: c_int) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_fs_fsync_ocall(&mut result as * mut c_int,
                                     &mut error as * mut c_int,
                                     fd);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn fdatasync(fd: c_int) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_fs_fdatasync_ocall(&mut result as * mut c_int,
                                          &mut error as * mut c_int,
                                          fd);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn ftruncate64(fd: c_int, length: off64_t) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_fs_ftruncate64_ocall(&mut result as * mut c_int,
                                            &mut error as * mut c_int,
                                            fd,
                                            length);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn lseek64(fd: c_int, offset: off64_t, whence: c_int) -> off64_t {

        let mut result: off64_t = 0;
        let mut error: c_int = 0;
        let status = u_fs_lseek64_ocall(&mut result as * mut off64_t,
                                        &mut error as * mut c_int,
                                        fd,
                                        offset,
                                        whence);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn fchmod(fd: c_int, mode: mode_t) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_fs_fchmod_ocall(&mut result as * mut c_int,
                                       &mut error as * mut c_int,
                                       fd,
                                       mode);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn unlink(pathname: * const c_char) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_fs_unlink_ocall(&mut result as * mut c_int,
                                       &mut error as * mut c_int,
                                       pathname);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn link(oldpath: * const c_char, newpath: * const c_char) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_fs_link_ocall(&mut result as * mut c_int,
                                     &mut error as * mut c_int,
                                     oldpath,
                                     newpath);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn rename(oldpath: * const c_char, newpath: * const c_char) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_fs_rename_ocall(&mut result as * mut c_int,
                                       &mut error as * mut c_int,
                                       oldpath,
                                       newpath);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn chmod(path: * const c_char, mode: mode_t) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_fs_chmod_ocall(&mut result as * mut c_int,
                                      &mut error as * mut c_int,
                                      path,
                                      mode);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn readlink(path: * const c_char, buf: * mut c_char, bufsz: size_t) -> ssize_t {

        let mut result: ssize_t = 0;
        let mut error: c_int = 0;
        let status = u_fs_readlink_ocall(&mut result as * mut ssize_t,
                                         &mut error as * mut c_int,
                                         path,
                                         buf,
                                         bufsz);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn symlink(path1: * const c_char, path2: * const c_char) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_fs_symlink_ocall(&mut result as * mut c_int,
                                        &mut error as * mut c_int,
                                        path1,
                                        path2);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn stat64(path: * const c_char, buf: * mut stat64) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_fs_stat64_ocall(&mut result as * mut c_int,
                                       &mut error as * mut c_int,
                                       path,
                                       buf);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn lstat64(path: * const c_char, buf: * mut stat64) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_fs_lstat64_ocall(&mut result as * mut c_int,
                                        &mut error as * mut c_int,
                                        path,
                                        buf);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn fcntl_arg0(fd: c_int, cmd: c_int) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_fs_fcntl_arg0_ocall(&mut result as * mut c_int,
                                           &mut error as * mut c_int,
                                           fd,
                                           cmd);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn realpath(pathname: * const c_char) -> * mut c_char {

        let mut result: * mut c_char = ptr::null_mut();
        let mut error: c_int = 0;
        let status = u_fs_realpath_ocall(&mut result as * mut * mut c_char,
                                         &mut error as * mut c_int,
                                         pathname);

        if status == sgx_status_t::SGX_SUCCESS {
            if result.is_null() {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = ptr::null_mut();
        }
        result
    }

    pub unsafe fn free(p: * mut c_void) {

        let _ = u_fs_free_ocall(p);
    }
}