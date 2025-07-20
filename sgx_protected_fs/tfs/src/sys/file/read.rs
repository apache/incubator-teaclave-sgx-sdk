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
    pub fn read(&mut self, buf: &mut [u8]) -> FsResult<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        #[cfg(feature = "tfs")]
        ensure!(buf.is_enclave_range(), esgx!(SgxStatus::InvalidParameter));
        ensure!(self.status.is_ok(), esgx!(SgxStatus::BadStatus));
        ensure!(self.opts.read || self.opts.update, eos!(EACCES));

        if self.end_of_file {
            return Ok(0);
        }
        let file_size = self.metadata.encrypted_plain.size;
        if self.offset == file_size {
            self.end_of_file = true;
            return Ok(0);
        }

        let mut left_to_read = buf.len();
        if left_to_read > file_size - self.offset {
            left_to_read = file_size - self.offset;
        }
        let attempted_to_read = left_to_read;
        let mut offset = 0;

        if self.offset < MD_USER_DATA_SIZE {
            let len = left_to_read.min(MD_USER_DATA_SIZE - self.offset);
            buf[offset..offset + len].copy_from_slice(
                &self.metadata.encrypted_plain.data[self.offset..self.offset + len],
            );
            offset += len;
            left_to_read -= len;
            self.offset += len;
        }

        while left_to_read > 0 {
            let file_node = match self.get_data_node() {
                Ok(node) => node,
                Err(error) => {
                    self.set_last_error(error);
                    break;
                }
            };

            let offset_in_node = (self.offset - MD_USER_DATA_SIZE) % NODE_SIZE;
            let len = left_to_read.min(NODE_SIZE - offset_in_node);
            buf[offset..offset + len].copy_from_slice(
                &file_node.borrow().plaintext.as_ref()[offset_in_node..offset_in_node + len],
            );
            offset += len;
            left_to_read -= len;
            self.offset += len;
        }

        // user wanted to read more and we had to shrink the request
        if left_to_read == 0 && attempted_to_read != buf.len() {
            assert!(self.offset == file_size);
            self.end_of_file = true;
        }

        Ok(attempted_to_read - left_to_read)
    }

    pub fn read_at(&mut self, buf: &mut [u8], offset: u64) -> FsResult<usize> {
        let cur_offset = self.offset;
        let file_size = self.metadata.encrypted_plain.size as u64;

        if offset > file_size {
            return Ok(0);
        }

        self.seek(SeekFrom::Start(offset))?;
        let result = self.read(buf);
        self.offset = cur_offset;

        result
    }
}
