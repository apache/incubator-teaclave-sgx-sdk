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

use crate::sys::error::FsResult;
use crate::sys::file::FileInner;
use crate::sys::metadata::MD_USER_DATA_SIZE;
use crate::sys::node::NODE_SIZE;
use sgx_types::error::errno::EACCES;
use sgx_types::error::SgxStatus;
use std::io::SeekFrom;

#[cfg(feature = "tfs")]
use sgx_trts::trts::EnclaveRange;

impl FileInner {
    pub fn write(&mut self, buf: &[u8]) -> FsResult<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        #[cfg(feature = "tfs")]
        ensure!(buf.is_enclave_range(), esgx!(SgxStatus::InvalidParameter));
        ensure!(self.status.is_ok(), esgx!(SgxStatus::BadStatus));
        ensure!(
            self.opts.write || self.opts.append || self.opts.update,
            eos!(EACCES)
        );

        if self.opts.append {
            self.offset = self.metadata.encrypted_plain.size;
        }

        let mut left_to_write = buf.len();
        let mut offset = 0;

        // the first block of user data is written in the meta-data encrypted part
        if self.offset < MD_USER_DATA_SIZE {
            let len = left_to_write.min(MD_USER_DATA_SIZE - self.offset);
            self.metadata.encrypted_plain.data[self.offset..self.offset + len]
                .copy_from_slice(&buf[offset..offset + len]);
            left_to_write -= len;
            offset += len;
            self.offset += len;

            if self.offset > self.metadata.encrypted_plain.size {
                self.metadata.encrypted_plain.size = self.offset;
            }
            self.need_writing = true;
        }

        while left_to_write > 0 {
            let file_node = match self.get_data_node() {
                Ok(node) => node,
                Err(error) => {
                    self.set_last_error(error);
                    break;
                }
            };

            let offset_in_node = (self.offset - MD_USER_DATA_SIZE) % NODE_SIZE;
            let len = left_to_write.min(NODE_SIZE - offset_in_node);
            file_node.borrow_mut().plaintext.as_mut()[offset_in_node..offset_in_node + len]
                .copy_from_slice(&buf[offset..offset + len]);

            left_to_write -= len;
            offset += len;
            self.offset += len;

            if self.offset > self.metadata.encrypted_plain.size {
                self.metadata.encrypted_plain.size = self.offset;
            }

            let mut file_node = file_node.borrow_mut();
            if !file_node.need_writing {
                file_node.need_writing = true;

                let mut parent = file_node.parent.clone();
                while let Some(mht) = parent {
                    let mut mht = mht.borrow_mut();
                    if !mht.is_root_mht() {
                        mht.need_writing = true;
                        parent = mht.parent.clone();
                    } else {
                        break;
                    }
                }

                self.root_mht.borrow_mut().need_writing = true;
                self.need_writing = true;
            }
        }
        Ok(offset)
    }

    pub fn write_at(&mut self, buf: &[u8], offset: u64) -> FsResult<usize> {
        let cur_offset = self.offset;
        let file_size = self.metadata.encrypted_plain.size as u64;

        if offset > file_size {
            self.seek(SeekFrom::End(0))?;

            static ZEROS: [u8; 0x1000] = [0; 0x1000];
            let mut left_to_write = offset - file_size;
            while left_to_write > 0 {
                let len = left_to_write.min(0x1000) as usize;
                let written_len = self.write(&ZEROS[..len]).map_err(|error| {
                    self.offset = cur_offset;
                    error
                })?;
                left_to_write -= written_len as u64;
            }
        }

        self.seek(SeekFrom::Start(offset)).map_err(|error| {
            self.offset = cur_offset;
            error
        })?;
        let result = self.write(buf);
        self.offset = cur_offset;

        result
    }
}
