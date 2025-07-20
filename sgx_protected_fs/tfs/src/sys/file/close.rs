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
use crate::sys::file::{CloseMode, FileInner, FileStatus};
use crate::sys::host;
use sgx_types::error::SgxStatus;
use sgx_types::types::Key128bit;

impl FileInner {
    pub fn close(&mut self, mode: CloseMode) -> FsResult<Option<Key128bit>> {
        match mode {
            CloseMode::Import | CloseMode::Export => {
                ensure!(
                    self.metadata.encrypt_flags().is_auto_key(),
                    esgx!(SgxStatus::Unexpected)
                );
            }
            _ => (),
        }

        if mode == CloseMode::Import {
            self.need_writing = true;
        }

        if !self.status.is_ok() {
            self.clear_error()?;
        } else {
            self.internal_flush(true)?;
        }

        if self.status.is_ok() && self.last_error.is_success() {
            self.remove_recovery_file();
        }

        self.set_file_status(FileStatus::Closed);
        if mode == CloseMode::Export {
            self.metadata.restore_key(&self.key_gen).map(Some)
        } else {
            Ok(None)
        }
    }

    fn remove_recovery_file(&mut self) {
        if !self.recovery_path.as_path().as_os_str().is_empty() {
            let _ = host::remove(&self.recovery_path);
        }
    }
}
