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

#[macro_use]
extern crate sgx_types;

use libc::{self, c_void};
use sgx_types::error::OsResult;
use std::ffi::{CStr, CString};
use std::fs::{self, File, OpenOptions};
use std::io::{Error, ErrorKind, SeekFrom};
use std::mem::{self, ManuallyDrop};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::path::Path;

const MILISECONDS_SLEEP_FOPEN: u32 = 10;
const MAX_FOPEN_RETRIES: usize = 10;
const NODE_SIZE: usize = 4096;
const RECOVERY_NODE_SIZE: usize = mem::size_of::<u64>() + NODE_SIZE;

#[derive(Debug)]
pub struct HostFile {
    stream: FileStream,
    fd: RawFd,
}

impl HostFile {
    pub fn open(name: &Path, readonly: bool) -> OsResult<HostFile> {
        let mut open_mode = OpenOptions::new();
        open_mode.read(true);
        if !readonly {
            open_mode.write(true).create(true);
        }

        let oflag = libc::O_LARGEFILE;
        let mode = libc::S_IRUSR
            | libc::S_IWUSR
            | libc::S_IRGRP
            | libc::S_IWGRP
            | libc::S_IROTH
            | libc::S_IWOTH;

        let file = open_mode
            .mode(mode)
            .custom_flags(oflag)
            .open(name)
            .map_err(|e| e.raw_os_error().unwrap_or(libc::EIO))?;

        // this lock is advisory only and programs with high priviliges can ignore it
        // it is set to help the user avoid mistakes, but it won't prevent intensional DOS attack from priviliged user
        let op = if readonly {
            libc::LOCK_SH
        } else {
            libc::LOCK_EX
        } | libc::LOCK_NB; // NB - non blocking

        let fd = file.as_raw_fd();
        unsafe {
            if libc::flock(fd, op) < 0 {
                Err(errno())
            } else {
                Ok(())
            }
        }?;

        let mode = CStr::from_bytes_with_nul(if readonly { b"rb\0" } else { b"r+b\0" })
            .map_err(|_| libc::EINVAL)?;
        let stream = unsafe {
            FileStream::from_raw_fd(fd, mode).map_err(|e| {
                libc::flock(fd, libc::LOCK_UN);
                e
            })
        }?;

        Ok(HostFile {
            stream,
            fd: file.into_raw_fd(),
        })
    }

    pub fn read(&mut self, number: u64, node: &mut [u8]) -> OsResult {
        ensure!(node.len() == NODE_SIZE, libc::EINVAL);

        let offset = number * NODE_SIZE as u64;
        self.stream.seek(SeekFrom::Start(offset))?;
        self.stream.read(node)
    }

    pub fn write(&mut self, number: u64, node: &[u8]) -> OsResult {
        ensure!(node.len() == NODE_SIZE, libc::EINVAL);

        let offset = number * NODE_SIZE as u64;
        self.stream.seek(SeekFrom::Start(offset))?;
        self.stream.write(node)
    }

    pub fn flush(&mut self) -> OsResult {
        self.stream.flush()
    }

    pub fn size(&self) -> OsResult<usize> {
        let file = ManuallyDrop::new(unsafe { File::from_raw_fd(self.fd) });
        let metadata = file
            .metadata()
            .map_err(|e| e.raw_os_error().unwrap_or(libc::EIO))?;

        ensure!(metadata.is_file(), libc::EINVAL);
        Ok(metadata.len() as usize)
    }

    pub fn into_raw_stream(self) -> RawFileStream {
        ManuallyDrop::new(self).stream.stream
    }

    /// # Safety
    pub unsafe fn from_raw_stream(stream: RawFileStream) -> OsResult<HostFile> {
        ensure!(!stream.is_null(), libc::EINVAL);
        let fd = libc::fileno(stream);
        ensure!(fd >= 0, errno());

        Ok(Self {
            stream: FileStream { stream },
            fd,
        })
    }
}

impl Drop for HostFile {
    fn drop(&mut self) {
        unsafe {
            libc::flock(self.fd, libc::LOCK_UN);
        }
    }
}

#[derive(Debug)]
pub struct RecoveryFile {
    stream: FileStream,
}

impl RecoveryFile {
    pub fn open(name: &Path) -> OsResult<RecoveryFile> {
        let mode = CStr::from_bytes_with_nul(b"wb\0").map_err(|_| libc::EINVAL)?;
        for _ in 0..MAX_FOPEN_RETRIES {
            if let Ok(stream) = FileStream::open(name, mode) {
                return Ok(RecoveryFile { stream });
            }
            unsafe { libc::usleep(MILISECONDS_SLEEP_FOPEN) };
        }
        Err(libc::EBUSY)
    }

    pub fn write(&mut self, node: &[u8]) -> OsResult {
        ensure!(node.len() == RECOVERY_NODE_SIZE, libc::EINVAL);

        self.stream.write(node)
    }

    pub fn into_raw_stream(self) -> RawFileStream {
        ManuallyDrop::new(self).stream.stream
    }

    /// # Safety
    pub unsafe fn from_raw_stream(stream: RawFileStream) -> OsResult<RecoveryFile> {
        Ok(Self {
            stream: FileStream { stream },
        })
    }
}

pub type RawFileStream = *mut libc::FILE;

fn cstr(name: &Path) -> OsResult<CString> {
    CString::new(name.to_str().ok_or(libc::EINVAL)?).map_err(|_| libc::EINVAL)
}

fn errno() -> i32 {
    Error::last_os_error().raw_os_error().unwrap_or(0)
}

#[derive(Debug)]
struct FileStream {
    stream: RawFileStream,
}

impl FileStream {
    fn open(name: &Path, mode: &CStr) -> OsResult<FileStream> {
        let name = cstr(name)?;
        let stream = unsafe { libc::fopen(name.as_ptr(), mode.as_ptr()) };
        if stream.is_null() {
            Err(errno())
        } else {
            Ok(FileStream { stream })
        }
    }

    fn read(&self, buf: &mut [u8]) -> OsResult {
        let size =
            unsafe { libc::fread(buf.as_mut_ptr() as *mut c_void, buf.len(), 1, self.stream) };
        if size != 1 {
            let err = self.last_error();
            if err != 0 {
                bail!(err);
            } else if errno() != 0 {
                bail!(errno());
            } else {
                bail!(libc::EIO);
            }
        }
        Ok(())
    }

    fn write(&self, buf: &[u8]) -> OsResult {
        let size =
            unsafe { libc::fwrite(buf.as_ptr() as *const c_void, 1, buf.len(), self.stream) };
        if size != buf.len() {
            let err = self.last_error();
            if err != 0 {
                bail!(err);
            } else if errno() != 0 {
                bail!(errno());
            } else {
                bail!(libc::EIO);
            }
        }
        Ok(())
    }

    fn flush(&self) -> OsResult {
        if unsafe { libc::fflush(self.stream) } != 0 {
            bail!(errno())
        }
        Ok(())
    }

    fn seek(&self, pos: SeekFrom) -> OsResult {
        let (offset, whence) = match pos {
            SeekFrom::Start(off) => (off as i64, libc::SEEK_SET),
            SeekFrom::End(off) => (off, libc::SEEK_END),
            SeekFrom::Current(off) => (off, libc::SEEK_CUR),
        };
        if unsafe { libc::fseeko(self.stream, offset, whence) } != 0 {
            bail!(errno())
        }
        Ok(())
    }

    fn tell(&self) -> OsResult<u64> {
        let off = unsafe { libc::ftello(self.stream) };
        ensure!(off >= 0, errno());

        Ok(off as u64)
    }

    fn last_error(&self) -> i32 {
        unsafe { libc::ferror(self.stream) }
    }

    /// # Safety
    unsafe fn from_raw_fd(fd: RawFd, mode: &CStr) -> OsResult<FileStream> {
        let stream = libc::fdopen(fd, mode.as_ptr());
        ensure!(!stream.is_null(), errno());

        Ok(FileStream { stream })
    }
}

impl Drop for FileStream {
    fn drop(&mut self) {
        let _ = unsafe { libc::fclose(self.stream) };
    }
}

pub fn remove(name: &Path) -> OsResult {
    fs::remove_file(name).map_err(|e| e.raw_os_error().unwrap_or(libc::EIO))
}

pub fn try_exists(path: &Path) -> OsResult<bool> {
    match fs::metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.raw_os_error().unwrap_or(libc::EIO)),
    }
}

pub fn recovery(source: &Path, recovery: &Path) -> OsResult {
    let mode = CStr::from_bytes_with_nul(b"rb\0").map_err(|_| libc::EINVAL)?;
    let recov = FileStream::open(recovery, mode)?;

    recov.seek(SeekFrom::End(0))?;
    let size = recov.tell()? as usize;
    recov.seek(SeekFrom::Start(0))?;

    ensure!(size % RECOVERY_NODE_SIZE == 0, libc::ENOTSUP);

    let nodes_count = size / RECOVERY_NODE_SIZE;

    let mode = CStr::from_bytes_with_nul(b"r+b\0").map_err(|_| libc::EINVAL)?;
    let src = FileStream::open(source, mode)?;

    let mut data = vec![0_u8; RECOVERY_NODE_SIZE];
    for _ in 0..nodes_count {
        recov.read(data.as_mut_slice())?;
        // seek the regular file to the required offset
        let mut number = [0u8; 8];
        number.copy_from_slice(&data[0..8]);
        let physical_node_number = u64::from_ne_bytes(number);

        src.seek(SeekFrom::Start(physical_node_number * NODE_SIZE as u64))?;
        src.write(&data[8..])?;
    }

    src.flush()?;
    remove(recovery)
}
