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

#![allow(dead_code)]

use crate::enclave::SgxEnclave;
use sgx_types::error::SgxResult;
use sgx_types::function::sgx_ecall;
use sgx_types::types::EnclaveId;
use std::alloc::{Allocator, Global};
use std::env;
use std::mem;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr;
use std::slice;
use std::vec;

impl SgxEnclave {
    pub(crate) fn init(&self) -> SgxResult {
        const ECMD_INIT: i32 = i32::MAX;

        let env = env();
        let args = args();
        cfg_if! {
            if #[cfg(feature = "hyper")] {
                use crate::msbuf::MsBufAlloc;
                use sgx_types::error::SgxStatus;

                let alloc = MsBufAlloc::new(self.eid());
                let remain_size = alloc.remain_size();

                let info = InitInfo::new_with_allocator(self.eid(), self.path(), env, alloc);
                ensure!(info.len() < remain_size, SgxStatus::OutOfMemory);

                let bytes = info.into_bytes_with_allocator();
            } else {
                let info = InitInfo::new(self.eid(), self.path(), env, args);
                let bytes = info.into_bytes();
            }
        }

        let status =
            unsafe { sgx_ecall(self.eid(), ECMD_INIT, ptr::null(), bytes.as_ptr().cast()) };

        if status.is_success() {
            Ok(())
        } else {
            Err(status)
        }
    }
}

pub fn env() -> Vec<u8> {
    let mut result = Vec::new();
    for var in env::vars() {
        result.extend_from_slice(var.0.as_bytes());
        result.push(b'=');
        result.extend_from_slice(var.1.as_bytes());
        result.push(0);
    }
    result
}

pub fn args() -> Vec<u8> {
    let mut result = Vec::new();
    for arg in env::args() {
        result.extend_from_slice(arg.as_bytes());
        result.push(0);
    }
    result
}

#[derive(Debug)]
pub struct InitInfo<A: Allocator = Global> {
    eid: EnclaveId,
    path: Option<PathBuf>,
    env: Vec<u8>,
    args: Vec<u8>,
    alloc: A,
}

impl InitInfo<Global> {
    #[inline]
    pub fn new(eid: EnclaveId, path: Option<PathBuf>, env: Vec<u8>, args: Vec<u8>) -> InitInfo {
        Self::new_with_allocator(eid, path, env, args, Global)
    }

    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        self.into_bytes_with_allocator()
    }
}

impl<A: Allocator> InitInfo<A> {
    pub fn new_with_allocator(
        eid: EnclaveId,
        path: Option<PathBuf>,
        env: Vec<u8>,
        args: Vec<u8>,
        alloc: A,
    ) -> InitInfo<A> {
        InitInfo {
            eid,
            path,
            env,
            args,
            alloc,
        }
    }

    pub fn len(&self) -> usize {
        let path = self
            .path
            .as_ref()
            .map(|path| path.as_path().as_os_str().as_bytes())
            .unwrap_or(&[0]);
        let path_len = path.len();
        let env_len = self.env.len();
        let args_len = self.args.len();

        mem::size_of::<InfoHeader>() + path_len + env_len + args_len
    }

    pub fn into_bytes_with_allocator(self) -> Vec<u8, A> {
        let path = self
            .path
            .as_ref()
            .map(|path| path.as_path().as_os_str().as_bytes())
            .unwrap_or(&[0]);
        let path_len = path.len();
        let env_len = self.env.len();
        let args_len = self.args.len();
        let header_len = mem::size_of::<InfoHeader>();

        let info_size = header_len + path_len + env_len + args_len;
        let mut bytes = vec::from_elem_in(0_u8, info_size, self.alloc);

        let raw_info = InfoHeader {
            eid: self.eid,
            info_size,
            path_len,
            env_len,
            args_len,
        };
        bytes[..header_len].copy_from_slice(raw_info.as_ref());
        bytes[header_len..header_len + path_len].copy_from_slice(path);
        bytes[header_len + path_len..header_len + path_len + env_len]
            .copy_from_slice(self.env.as_slice());
        bytes[header_len + path_len + env_len..].copy_from_slice(self.args.as_slice());

        bytes
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct InfoHeader {
    eid: EnclaveId,
    info_size: usize,
    path_len: usize,
    env_len: usize,
    args_len: usize,
}

impl AsRef<[u8]> for InfoHeader {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self as *const _ as *const u8, mem::size_of::<InfoHeader>())
        }
    }
}
