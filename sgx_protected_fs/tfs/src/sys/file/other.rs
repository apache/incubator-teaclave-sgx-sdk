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

use crate::sys::error::{FsError, FsResult};
use crate::sys::file::{FileInner, FileStatus};
use crate::sys::host;
use crate::sys::metadata::FILENAME_MAX_LEN;
use sgx_types::error::errno::*;
use sgx_types::error::SgxStatus;
use sgx_types::types::Mac128bit;
use std::io::SeekFrom;
use std::path::Path;

impl FileInner {
    #[inline]
    pub fn remove(path: &Path) -> FsResult {
        host::remove(path)
    }

    #[inline]
    pub fn tell(&mut self) -> FsResult<u64> {
        ensure!(self.status.is_ok(), esgx!(SgxStatus::BadStatus));
        Ok(self.offset as u64)
    }

    #[inline]
    pub fn get_eof(&self) -> bool {
        self.end_of_file
    }

    #[inline]
    pub fn file_size(&self) -> FsResult<u64> {
        ensure!(self.status.is_ok(), esgx!(SgxStatus::BadStatus));
        Ok(self.metadata.encrypted_plain.size as u64)
    }

    pub fn seek(&mut self, pos: SeekFrom) -> FsResult<u64> {
        ensure!(self.status.is_ok(), esgx!(SgxStatus::BadStatus));

        let file_size = self.metadata.encrypted_plain.size as u64;
        let new_offset = match pos {
            SeekFrom::Start(off) => {
                if off <= file_size {
                    Some(off)
                } else {
                    None
                }
            }
            SeekFrom::End(off) => {
                if off <= 0 {
                    file_size.checked_sub((0 - off) as u64)
                } else {
                    None
                }
            }
            SeekFrom::Current(off) => {
                let cur_offset = self.offset as u64;
                if off >= 0 {
                    match cur_offset.checked_add(off as u64) {
                        Some(new_offset) if new_offset <= file_size => Some(new_offset),
                        _ => None,
                    }
                } else {
                    cur_offset.checked_sub((0 - off) as u64)
                }
            }
        }
        .ok_or(EINVAL)?;

        self.offset = new_offset as usize;
        self.end_of_file = false;
        Ok(self.offset as u64)
    }

    pub fn set_len(&mut self, size: u64) -> FsResult {
        let new_size = size as usize;
        let mut cur_offset = self.offset;
        let file_size = self.metadata.encrypted_plain.size;

        let mut reset_len = if new_size > file_size {
            // expand the file by padding null bytes
            self.seek(SeekFrom::End(0))?;
            new_size - file_size
        } else {
            // shrink the file by setting null bytes between len and file_size
            self.seek(SeekFrom::Start(size))?;
            file_size - new_size
        };

        static ZEROS: [u8; 0x1000] = [0; 0x1000];
        while reset_len > 0 {
            let len = reset_len.min(0x1000);

            let nwritten = match self.write(&ZEROS[..len]) {
                Ok(n) => n,
                Err(error) => {
                    if new_size > file_size {
                        self.offset = cur_offset;
                        bail!(error);
                    } else {
                        // ignore errors in shrinking files
                        break;
                    }
                }
            };
            reset_len -= nwritten;
        }

        if cur_offset > new_size {
            cur_offset = new_size;
        }
        self.offset = cur_offset;
        self.end_of_file = false;
        self.metadata.encrypted_plain.size = new_size;
        Ok(())
    }

    // clears the cache with all the plain data that was in it doesn't clear the metadata
    // and first node, which are part of the 'main' structure
    pub fn clear_cache(&mut self) -> FsResult {
        if self.status.is_ok() {
            self.internal_flush(true)?;
        } else {
            // attempt to fix the file, will also flush it
            self.clear_error()?;
        }

        ensure!(self.status.is_ok(), esgx!(SgxStatus::BadStatus));

        while let Some(node) = self.cache.pop_back() {
            if node.borrow().need_writing {
                bail!(esgx!(SgxStatus::BadStatus));
            }
        }
        Ok(())
    }

    pub fn clear_error(&mut self) -> FsResult {
        match self.status {
            FileStatus::Ok => {
                self.set_last_error(SgxStatus::Success);
                self.end_of_file = false;
            }
            FileStatus::WriteToDiskFailed => {
                self.write_to_disk(true)?;
                self.need_writing = false;
                self.set_file_status(FileStatus::Ok);
            }
            _ => {
                self.internal_flush(true)?;
                self.set_file_status(FileStatus::Ok);
            }
        }
        Ok(())
    }

    #[inline]
    pub fn get_metadata_mac(&mut self) -> FsResult<Mac128bit> {
        self.flush()?;
        Ok(self.metadata.node.metadata.plaintext.gmac)
    }

    pub fn rename(&mut self, old_name: &str, new_name: &str) -> FsResult {
        let old_len = old_name.len();
        ensure!(old_len > 0, eos!(EINVAL));
        ensure!(old_len < FILENAME_MAX_LEN - 1, eos!(ENAMETOOLONG));

        let new_len = new_name.len();
        ensure!(new_len > 0, eos!(EINVAL));
        ensure!(new_len < FILENAME_MAX_LEN - 1, eos!(ENAMETOOLONG));

        let meta_file_name = self.metadata.file_name()?;
        ensure!(meta_file_name == old_name, esgx!(SgxStatus::NameMismatch));

        self.metadata.encrypted_plain.file_name.fill(0);
        self.metadata.encrypted_plain.file_name[0..new_len].copy_from_slice(new_name.as_bytes());

        self.need_writing = true;
        Ok(())
    }
}

impl FileInner {
    #[inline]
    pub fn get_last_error(&self) -> FsError {
        if self.last_error.is_success() && !self.status.is_ok() {
            esgx!(SgxStatus::BadStatus)
        } else {
            self.last_error
        }
    }

    #[inline]
    pub fn set_last_error<E: Into<FsError>>(&mut self, error: E) {
        self.last_error = error.into();
    }

    #[inline]
    pub fn set_file_status(&mut self, status: FileStatus) {
        self.status = status;
    }
}
