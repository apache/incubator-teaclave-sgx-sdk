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

use super::HostFs;
use crate::sys::error::FsResult;
use crate::sys::node::NODE_SIZE;
use sgx_types::error::errno::*;
use sgx_types::error::SgxStatus;
use sgx_uprotected_fs as ufs;
use std::path::Path;

#[derive(Debug)]
pub struct HostFile {
    file: ufs::HostFile,
    size: usize,
}

impl HostFile {
    pub fn open(name: &Path, readonly: bool) -> FsResult<HostFile> {
        let file = ufs::HostFile::open(name, readonly)?;
        let size = file.size()?;

        ensure!(
            size <= i64::MAX as usize && size % NODE_SIZE == 0,
            esgx!(SgxStatus::NotSgxFile)
        );
        Ok(HostFile { file, size })
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }
}

impl HostFs for HostFile {
    fn read(&mut self, number: u64, node: &mut dyn AsMut<[u8]>) -> FsResult {
        self.file.read(number, node.as_mut()).map_err(|e| eos!(e))
    }

    fn write(&mut self, number: u64, node: &dyn AsRef<[u8]>) -> FsResult {
        self.file.write(number, node.as_ref()).map_err(|e| eos!(e))
    }

    fn flush(&mut self) -> FsResult {
        self.file.flush().map_err(|_| esgx!(SgxStatus::FluchFailed))
    }
}

#[derive(Debug)]
pub struct RecoveryFile {
    file: ufs::RecoveryFile,
}

impl RecoveryFile {
    pub fn open(name: &Path) -> FsResult<RecoveryFile> {
        let file = ufs::RecoveryFile::open(name)?;
        Ok(RecoveryFile { file })
    }
}

impl HostFs for RecoveryFile {
    fn read(&mut self, _number: u64, _node: &mut dyn AsMut<[u8]>) -> FsResult {
        bail!(eos!(ENOTSUP))
    }

    fn write(&mut self, _number: u64, node: &dyn AsRef<[u8]>) -> FsResult {
        self.file.write(node.as_ref()).map_err(|e| eos!(e))
    }

    fn flush(&mut self) -> FsResult {
        bail!(eos!(ENOTSUP))
    }
}

pub fn try_exists(name: &Path) -> FsResult<bool> {
    ufs::try_exists(name).map_err(|e| eos!(e))
}

pub fn remove(name: &Path) -> FsResult {
    ufs::remove(name).map_err(|e| eos!(e))
}

pub fn recovery(source: &Path, recovery: &Path) -> FsResult {
    ufs::recovery(source, recovery).map_err(|e| eos!(e))
}
