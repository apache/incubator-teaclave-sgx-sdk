// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
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

use sgx_types::{sgx_status_t, sgx_key_128bit_t};
use sgx_trts::libc;
use sgx_tprotected_fs::{self, SgxFileStream};
use os::unix::prelude::*;
use ffi::{CString, CStr};
use io::{self, Error, ErrorKind, SeekFrom};
use path::Path;
use sys_common::FromInner;

pub struct SgxFile(SgxFileStream);

#[derive(Clone, Debug)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
    update: bool,
    binary: bool,
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions {
            read: false,
            write: false,
            append: false,
            update: false,
            binary: false,
        }
    }

    pub fn read(&mut self, read: bool) { self.read = read; }
    pub fn write(&mut self, write: bool) { self.write = write; }
    pub fn append(&mut self, append: bool) { self.append = append; }
    pub fn update(&mut self, update: bool) { self.update = update; }
    pub fn binary(&mut self, binary: bool) { self.binary = binary; }

    fn get_access_mode(&self) -> io::Result<String> {

        let mut mode = match (self.read, self.write, self.append) {
            (true,  false, false) => "r".to_string(),
            (false, true,  false) => "w".to_string(),
            (false, false, true)  => "a".to_string(),
            _ => {return Err(Error::from_raw_os_error(libc::EINVAL))},
        };
        if self.update == true {
            mode += "+";
        }
        if self.binary == true {
            mode += "b";
        }
        Ok(mode)
    }
}

impl SgxFile {
    pub fn open(path: &Path, opts: &OpenOptions) -> io::Result<SgxFile> {

        let path = cstr(path)?;
        let mode = opts.get_access_mode()?;
        let opts = CString::new(mode.as_bytes())?;
        SgxFile::open_c(&path, &opts, &sgx_key_128bit_t::default(), true)
    }

    pub fn open_ex(path: &Path, opts: &OpenOptions, key: &sgx_key_128bit_t) -> io::Result<SgxFile> {

        let path = cstr(path)?;
        let mode = opts.get_access_mode()?;
        let opts = CString::new(mode.as_bytes())?;
        SgxFile::open_c(&path, &opts, key, false)
    }

    pub fn open_c(path: &CStr, opts: &CStr, key: &sgx_key_128bit_t, auto: bool) -> io::Result<SgxFile> {

        let file = if auto == true {
            SgxFileStream::open_auto_key(path, opts)
        } else {
            SgxFileStream::open(path, opts, key)
        };

        file.map(|stream| SgxFile(stream))
            .map_err(|err| {
                match err {
                    1 => Error::from_sgx_error(sgx_status_t::SGX_ERROR_UNEXPECTED),
                    2 => Error::from_raw_os_error(libc::ENOENT),
                    3 => Error::from_sgx_error(sgx_status_t::SGX_ERROR_OUT_OF_MEMORY),
                    4 | 5 => Error::from_raw_os_error(err),
                    r if r > 4096 => {
                        let status = sgx_status_t::from_repr(r as u32).unwrap_or(sgx_status_t::SGX_ERROR_UNEXPECTED);
                        Error::from_sgx_error(status)
                    },
                    _ => Error::from_raw_os_error(err),
                }
            })
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {

        self.0.read(buf).map_err(|err| {
            match err {
                1 => Error::from_sgx_error(sgx_status_t::SGX_ERROR_UNEXPECTED),
                2 => Error::from_sgx_error(sgx_status_t::SGX_ERROR_INVALID_PARAMETER),
                3 => Error::from_sgx_error(sgx_status_t::SGX_ERROR_OUT_OF_MEMORY),
                4 | 5 => Error::from_raw_os_error(err),
                r if r > 4096 => {
                    let status = sgx_status_t::from_repr(r as u32).unwrap_or(sgx_status_t::SGX_ERROR_UNEXPECTED);
                    Error::from_sgx_error(status)
                },
                _ => Error::from_raw_os_error(err),
            }
        })
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {

        self.0.write(buf).map_err(|err| {
            match err {
                1 => Error::from_sgx_error(sgx_status_t::SGX_ERROR_UNEXPECTED),
                2 => Error::from_sgx_error(sgx_status_t::SGX_ERROR_INVALID_PARAMETER),
                3 => Error::from_sgx_error(sgx_status_t::SGX_ERROR_OUT_OF_MEMORY),
                4 | 5 => Error::from_raw_os_error(err),
                r if r > 4096 => {
                    let status = sgx_status_t::from_repr(r as u32).unwrap_or(sgx_status_t::SGX_ERROR_UNEXPECTED);
                    Error::from_sgx_error(status)
                },
                _ => Error::from_raw_os_error(err),
            }
        })
    }

    pub fn tell(&self) -> io::Result<u64> {

        self.0.tell().map_err(|err| {
            match err {
                r if r > 4096 => {
                    let status = sgx_status_t::from_repr(r as u32).unwrap_or(sgx_status_t::SGX_ERROR_UNEXPECTED);
                    Error::from_sgx_error(status)
                },
                _ => Error::from_raw_os_error(err),
            }
        })
        .map(|offset| offset as u64)
    }

    pub fn seek(&self, pos: SeekFrom) -> io::Result<u64> {

        let (whence, offset) = match pos {
            SeekFrom::Start(off) => (sgx_tprotected_fs::SeekFrom::Start, off as i64),
            SeekFrom::End(off) => (sgx_tprotected_fs::SeekFrom::End, off),
            SeekFrom::Current(off) => (sgx_tprotected_fs::SeekFrom::Current, off),
        };

        try!(self.0.seek(offset, whence).map_err(|err| {
            match err {
                r if r > 4096 => {
                    let status = sgx_status_t::from_repr(r as u32).unwrap_or(sgx_status_t::SGX_ERROR_UNEXPECTED);
                    Error::from_sgx_error(status)
                },
                _ => Error::from_raw_os_error(err),
            }
        }));

        let offset = try!(self.tell());
        Ok(offset as u64)
    }

    pub fn flush(&self) -> io::Result<()> {

        self.0.flush().map_err(|err| {
            match err {
                1 => Error::from_sgx_error(sgx_status_t::SGX_ERROR_UNEXPECTED),
                2 => Error::from_sgx_error(sgx_status_t::SGX_ERROR_INVALID_PARAMETER),
                3 => Error::from_sgx_error(sgx_status_t::SGX_ERROR_OUT_OF_MEMORY),
                4 | 5 => Error::from_raw_os_error(err),
                r if r > 4096 => {
                    let status = sgx_status_t::from_repr(r as u32).unwrap_or(sgx_status_t::SGX_ERROR_UNEXPECTED);
                    Error::from_sgx_error(status)
                },
                _ => Error::from_raw_os_error(err),
            }
        })
    }

    pub fn is_eof(&self) -> bool {
        self.0.is_eof()
    }

    pub fn clearerr(&self) {
        self.0.clearerr()
    }

    pub fn clear_cache(&self) -> io::Result<()> {

        self.0.clear_cache().map_err(|err| {
            match err {
                1 => Error::from_sgx_error(sgx_status_t::SGX_ERROR_UNEXPECTED),
                2 => Error::from_sgx_error(sgx_status_t::SGX_ERROR_INVALID_PARAMETER),
                3 => Error::from_sgx_error(sgx_status_t::SGX_ERROR_OUT_OF_MEMORY),
                4 | 5 => Error::from_raw_os_error(err),
                r if r > 4096 => {
                    let status = sgx_status_t::from_repr(r as u32).unwrap_or(sgx_status_t::SGX_ERROR_UNEXPECTED);
                    Error::from_sgx_error(status)
                },
                _ => Error::from_raw_os_error(err),
            }
        })
    }
}

pub fn remove(path: &Path) -> io::Result<()> {

    let path = cstr(path)?;
    sgx_tprotected_fs::remove(&path).map_err(|err| {
        match err {
            1 => Error::from_sgx_error(sgx_status_t::SGX_ERROR_UNEXPECTED),
            2 => Error::from_raw_os_error(libc::ENOENT),
            3 => Error::from_sgx_error(sgx_status_t::SGX_ERROR_OUT_OF_MEMORY),
            4 | 5 => Error::from_raw_os_error(err),
            r if r > 4096 => {
                let status = sgx_status_t::from_repr(r as u32).unwrap_or(sgx_status_t::SGX_ERROR_UNEXPECTED);
                Error::from_sgx_error(status)
            },
            _ => Error::from_raw_os_error(err),
        }
    })
}

pub fn export_auto_key(path: &Path) -> io::Result<sgx_key_128bit_t> {

    let path = cstr(path)?;
    sgx_tprotected_fs::export_auto_key(&path).map_err(|err| {
        match err {
            1 => Error::from_sgx_error(sgx_status_t::SGX_ERROR_UNEXPECTED),
            2 => Error::from_raw_os_error(libc::ENOENT),
            3 => Error::from_sgx_error(sgx_status_t::SGX_ERROR_OUT_OF_MEMORY),
            4 | 5 => Error::from_raw_os_error(err),
            r if r > 4096 => {
                let status = sgx_status_t::from_repr(r as u32).unwrap_or(sgx_status_t::SGX_ERROR_UNEXPECTED);
                Error::from_sgx_error(status)
            },
            _ => Error::from_raw_os_error(err),
        }
    })
}

pub fn import_auto_key(path: &Path, key: &sgx_key_128bit_t) -> io::Result<()> {

    let path = cstr(path)?;
    sgx_tprotected_fs::import_auto_key(&path, key).map_err(|err| {
        match err {
            1 => Error::from_sgx_error(sgx_status_t::SGX_ERROR_UNEXPECTED),
            2 => Error::from_raw_os_error(libc::ENOENT),
            3 => Error::from_sgx_error(sgx_status_t::SGX_ERROR_OUT_OF_MEMORY),
            4 | 5 => Error::from_raw_os_error(err),
            r if r > 4096 => {
                let status = sgx_status_t::from_repr(r as u32).unwrap_or(sgx_status_t::SGX_ERROR_UNEXPECTED);
                Error::from_sgx_error(status)
            },
            _ => Error::from_raw_os_error(err),
        }
    })
}

fn cstr(path: &Path) -> io::Result<CString> {
    Ok(CString::new(path.as_os_str().as_bytes())?)
}

impl FromInner<SgxFileStream> for SgxFile {
    fn from_inner(stream: SgxFileStream) -> SgxFile {
        SgxFile(stream)
    }
}

pub fn copy(from: &Path, to: &Path) -> io::Result<u64> {

    use sgxfs::SgxFile;
    cfg_if! {
        if #[cfg(feature = "untrusted_fs")] {
            use fs;
        } else {
            use untrusted::fs;
            use untrusted::path::PathEx;
        }
    }

    let metadata = from.metadata()?;
    if !metadata.is_file() {
        return Err(Error::new(ErrorKind::InvalidInput,
                              "the source path is not an existing regular file"))
    }

    let mut reader = SgxFile::open(from)?;
    let mut writer = SgxFile::create(to)?;
    let perm = metadata.permissions();

    let ret = io::copy(&mut reader, &mut writer)?;
    fs::set_permissions(to, perm)?;
    Ok(ret)
}