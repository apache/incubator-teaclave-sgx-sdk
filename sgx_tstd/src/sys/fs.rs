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

use sgx_trts::libc::{c_int, mode_t, time_t, stat64, off64_t, DIR, dirent64};
use crate::os::unix::prelude::*;
use crate::ffi::{CStr, CString, OsStr, OsString};
use crate::io::{self, Error, ErrorKind, IoSlice, IoSliceMut, SeekFrom};
use crate::path::{Path, PathBuf};
use crate::sys::fd::FileDesc;
use crate::sys::time::SystemTime;
use crate::sys::{cvt, cvt_r};
use crate::sys_common::{AsInner, FromInner};
use core::{fmt, mem, ptr};
use alloc_crate::sync::Arc;

pub use crate::sys_common::fs::remove_dir_all;

pub struct File(FileDesc);

#[derive(Clone)]
pub struct FileAttr {
    stat: stat64,
}

#[derive(Debug)]
pub struct DirBuilder { 
    mode: mode_t,
 }

// all DirEntry's will have a reference to this struct
struct InnerReadDir {
    dirp: Dir,
    root: PathBuf,
}

#[derive(Clone)]
pub struct ReadDir {
    inner: Arc<InnerReadDir>,
    end_of_stream: bool,
}

struct Dir(*mut DIR);

unsafe impl Send for Dir {}
unsafe impl Sync for Dir {}

pub struct DirEntry {
    entry: dirent64,
    dir: ReadDir,
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
pub struct FilePermissions {
    mode: mode_t,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct FileType {
    mode: mode_t,
}

impl FileAttr {
    fn from_stat64(stat: stat64) -> Self {
        Self { stat }
    }
}

impl FileAttr {
    pub fn size(&self) -> u64 {
        self.stat.st_size as u64
    }
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
        Err(io::Error::new(
            io::ErrorKind::Other,
            "creation time is not available on this platform \
                            currently",
        ))
    }
}

impl AsInner<stat64> for FileAttr {
    fn as_inner(&self) -> &stat64 {
        &self.stat
    }
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
    pub fn mode(&self) -> u32 {
        self.mode as u32
    }
}

impl FileType {
    pub fn is_dir(&self) -> bool {
        self.is(libc::S_IFDIR)
    }
    pub fn is_file(&self) -> bool {
        self.is(libc::S_IFREG)
    }
    pub fn is_symlink(&self) -> bool {
        self.is(libc::S_IFLNK)
    }

    pub fn is(&self, mode: mode_t) -> bool {
        self.mode & libc::S_IFMT == mode
    }
}

impl FromInner<u32> for FilePermissions {
    fn from_inner(mode: u32) -> FilePermissions {
        FilePermissions { mode: mode as mode_t }
    }
}

impl fmt::Debug for ReadDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // This will only be called from std::fs::ReadDir, which will add a "ReadDir()" frame.
        // Thus the result will be e g 'ReadDir("/home")'
        fmt::Debug::fmt(&*self.inner.root, f)
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        if self.end_of_stream {
            return None;
        }

        unsafe {
            let mut ret = DirEntry { entry: mem::zeroed(), dir: self.clone() };
            let mut entry_ptr = ptr::null_mut();
            loop {
                if libc::readdir64_r(self.inner.dirp.0, &mut ret.entry, &mut entry_ptr) != 0 {
                    if entry_ptr.is_null() {
                        // We encountered an error (which will be returned in this iteration), but
                        // we also reached the end of the directory stream. The `end_of_stream`
                        // flag is enabled to make sure that we return `None` in the next iteration
                        // (instead of looping forever)
                        self.end_of_stream = true;
                    }
                    return Some(Err(Error::last_os_error()));
                }
                if entry_ptr.is_null() {
                    return None;
                }
                if ret.name_bytes() != b"." && ret.name_bytes() != b".." {
                    return Some(Ok(ret));
                }
            }
        }
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let r = unsafe { libc::closedir(self.0) };
        debug_assert_eq!(r, 0);
    }
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.dir.inner.root.join(OsStr::from_bytes(self.name_bytes()))
    }

    pub fn file_name(&self) -> OsString {
        OsStr::from_bytes(self.name_bytes()).to_os_string()
    }

    pub fn metadata(&self) -> io::Result<FileAttr> {
        let fd = cvt(unsafe {libc::dirfd(self.dir.inner.dirp.0)})?;
        let mut stat: stat64 = unsafe { mem::zeroed() };
        cvt(unsafe {
            libc::fstatat64(fd, self.entry.d_name.as_ptr(), &mut stat, libc::AT_SYMLINK_NOFOLLOW)
        })?;
        Ok(FileAttr { stat })
    }

    pub fn file_type(&self) -> io::Result<FileType> {
        match self.entry.d_type {
            libc::DT_CHR => Ok(FileType { mode: libc::S_IFCHR }),
            libc::DT_FIFO => Ok(FileType { mode: libc::S_IFIFO }),
            libc::DT_LNK => Ok(FileType { mode: libc::S_IFLNK }),
            libc::DT_REG => Ok(FileType { mode: libc::S_IFREG }),
            libc::DT_SOCK => Ok(FileType { mode: libc::S_IFSOCK }),
            libc::DT_DIR => Ok(FileType { mode: libc::S_IFDIR }),
            libc::DT_BLK => Ok(FileType { mode: libc::S_IFBLK }),
            _ => lstat(&self.path()).map(|m| m.file_type()),
        }
    }

    pub fn ino(&self) -> u64 {
        self.entry.d_ino as u64
    }

    fn name_bytes(&self) -> &[u8] {
        unsafe { CStr::from_ptr(self.entry.d_name.as_ptr()).to_bytes() }
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

    pub fn read(&mut self, read: bool) {
        self.read = read;
    }
    pub fn write(&mut self, write: bool) {
        self.write = write;
    }
    pub fn append(&mut self, append: bool) {
        self.append = append;
    }
    pub fn truncate(&mut self, truncate: bool) {
        self.truncate = truncate;
    }
    pub fn create(&mut self, create: bool) {
        self.create = create;
    }
    pub fn create_new(&mut self, create_new: bool) {
        self.create_new = create_new;
    }

    pub fn custom_flags(&mut self, flags: i32) {
        self.custom_flags = flags;
    }
    pub fn mode(&mut self, mode: u32) {
        self.mode = mode as mode_t;
    }

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
            (false, false) => {
                if self.truncate || self.create || self.create_new {
                    return Err(Error::from_raw_os_error(libc::EINVAL));
                }
            }
            (_, true) => {
                if self.truncate && !self.create_new {
                    return Err(Error::from_raw_os_error(libc::EINVAL));
                }
            }
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
        let flags = libc::O_CLOEXEC
            | opts.get_access_mode()?
            | opts.get_creation_mode()?
            | (opts.custom_flags as c_int & !libc::O_ACCMODE);
        let fd = cvt_r(|| unsafe { libc::open64(path.as_ptr(), flags, opts.mode as c_int) })?;
        let fd = FileDesc::new(fd);

        // Currently the standard library supports Linux 2.6.18 which did not
        // have the O_CLOEXEC flag (passed above). If we're running on an older
        // Linux kernel then the flag is just ignored by the OS. After we open
        // the first file, we check whether it has CLOEXEC set. If it doesn't,
        // we will explicitly ask for a CLOEXEC fd for every further file we
        // open, if it does, we will skip that step.
        //
        // The CLOEXEC flag, however, is supported on versions of macOS/BSD/etc
        // that we support, so we only do this on Linux currently.
        fn ensure_cloexec(fd: &FileDesc) -> io::Result<()> {
            use crate::sync::atomic::{AtomicUsize, Ordering};

            const OPEN_CLOEXEC_UNKNOWN: usize = 0;
            const OPEN_CLOEXEC_SUPPORTED: usize = 1;
            const OPEN_CLOEXEC_NOTSUPPORTED: usize = 2;
            static OPEN_CLOEXEC: AtomicUsize = AtomicUsize::new(OPEN_CLOEXEC_UNKNOWN);

            let need_to_set;
            match OPEN_CLOEXEC.load(Ordering::Relaxed) {
                OPEN_CLOEXEC_UNKNOWN => {
                    need_to_set = !fd.get_cloexec()?;
                    OPEN_CLOEXEC.store(
                        if need_to_set {
                            OPEN_CLOEXEC_NOTSUPPORTED
                        } else {
                            OPEN_CLOEXEC_SUPPORTED
                        },
                        Ordering::Relaxed,
                    );
                }
                OPEN_CLOEXEC_SUPPORTED => need_to_set = false,
                OPEN_CLOEXEC_NOTSUPPORTED => need_to_set = true,
                _ => unreachable!(),
            }
            if need_to_set {
                fd.set_cloexec()?;
            }
            Ok(())
        }

        ensure_cloexec(&fd)?;
        Ok(File(fd))
    }

    pub fn file_attr(&self) -> io::Result<FileAttr> {
        let mut stat: stat64 = unsafe { mem::zeroed() };
        cvt(unsafe { libc::fstat64(self.0.raw(), &mut stat) })?;
        Ok(FileAttr::from_stat64(stat))
    }

    pub fn fsync(&self) -> io::Result<()> {
        cvt_r(|| unsafe { libc::fsync(self.0.raw()) })?;
        Ok(())
    }

    pub fn datasync(&self) -> io::Result<()> {
        cvt_r(|| unsafe { os_datasync(self.0.raw()) })?;
        return Ok(());

        unsafe fn os_datasync(fd: c_int) -> c_int {
            libc::fdatasync(fd)
        }
    }

    pub fn truncate(&self, size: u64) -> io::Result<()> {
        use crate::convert::TryInto;
            let size: off64_t =
                size.try_into().map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            cvt_r(|| unsafe { libc::ftruncate64(self.0.raw(), size) }).map(drop)
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }

    pub fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        self.0.read_at(buf, offset)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    pub fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        self.0.write_at(buf, offset)
    }

    pub fn flush(&self) -> io::Result<()> {
        Ok(())
    }

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

    pub fn fd(&self) -> &FileDesc {
        &self.0
    }

    pub fn into_fd(self) -> FileDesc {
        self.0
    }

    pub fn set_permissions(&self, perm: FilePermissions) -> io::Result<()> {
        cvt_r(|| unsafe { libc::fchmod(self.0.raw(), perm.mode) })?;
        Ok(())
    }
}

impl DirBuilder {
    pub fn new() -> DirBuilder {
        DirBuilder { mode: 0o777 }
    }

    pub fn mkdir(&self, p: &Path) -> io::Result<()> {
        let p = cstr(p)?;
        cvt(unsafe { libc::mkdir(p.as_ptr(), self.mode) })?;
        Ok(())
    }

    pub fn set_mode(&mut self, mode: u32) {
        self.mode = mode as mode_t;
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
                _ => None,
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

pub fn readdir(p: &Path) -> io::Result<ReadDir> {
    let root = p.to_path_buf();
    let p = cstr(p)?;
    unsafe {
        let ptr = libc::opendir(p.as_ptr());
        if ptr.is_null() {
            Err(Error::last_os_error())
        } else {
            let inner = InnerReadDir { dirp: Dir(ptr), root };
            Ok(ReadDir { inner: Arc::new(inner), end_of_stream: false })
        }
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

pub fn rmdir(p: &Path) -> io::Result<()> {
    let p = cstr(p)?;
    cvt(unsafe { libc::rmdir(p.as_ptr()) })?;
    Ok(())
}

pub fn readlink(p: &Path) -> io::Result<PathBuf> {
    let c_path = cstr(p)?;
    let p = c_path.as_ptr();

    let mut buf = Vec::with_capacity(256);

    loop {
        let buf_read =
            cvt(unsafe { libc::readlink(p, buf.as_mut_ptr() as *mut _, buf.capacity()) })? as usize;

        unsafe {
            buf.set_len(buf_read);
        }

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
    cvt(unsafe { libc::stat64(p.as_ptr(), &mut stat as *mut _) })?;
    Ok(FileAttr::from_stat64(stat))
}

pub fn lstat(p: &Path) -> io::Result<FileAttr> {
    let p = cstr(p)?;
    let mut stat: stat64 = unsafe { mem::zeroed() };
    cvt(unsafe { libc::lstat64(p.as_ptr(), &mut stat as *mut _) })?;
    Ok(FileAttr::from_stat64(stat))
}

pub fn canonicalize(p: &Path) -> io::Result<PathBuf> {
    let path = CString::new(p.as_os_str().as_bytes())?;
    let buf;
    unsafe {
        let r = libc::realpath(path.as_ptr());
        if r.is_null() {
            return Err(io::Error::last_os_error());
        }
        buf = CStr::from_ptr(r).to_bytes().to_vec();
        libc::free(r as *mut _);
    }
    Ok(PathBuf::from(OsString::from_vec(buf)))
}

pub fn copy(from: &Path, to: &Path) -> io::Result<u64> {
    cfg_if! {
        if #[cfg(feature = "untrusted_fs")] {
            use crate::fs::File;
        } else {
            use crate::untrusted::fs::File;
            use crate::untrusted::path::PathEx;
        }
    }

    if !from.is_file() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "the source path is not an existing regular file",
        ));
    }

    let mut reader = File::open(from)?;
    let mut writer = File::create(to)?;
    let perm = reader.metadata()?.permissions();

    let ret = io::copy(&mut reader, &mut writer)?;
    writer.set_permissions(perm)?;
    Ok(ret)
}

mod libc {
    pub use sgx_trts::libc::*;
    pub use sgx_trts::libc::ocall::{open64, fstat64, fsync, fdatasync, ftruncate64, lseek64, fchmod,
                                    unlink, link, rename, chmod, readlink, symlink, stat64, lstat64,
                                    fcntl_arg0, realpath, free, readdir64_r, closedir, dirfd, mkdir, rmdir, opendir, fstatat64};
}