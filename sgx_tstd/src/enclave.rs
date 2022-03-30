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

use crate::fs;
use crate::io;
use crate::path::{Path, PathBuf};

use sgx_sync::SpinMutex;
use sgx_types::types::EnclaveId;

static ENCLAVE: Enclave = Enclave::new();

#[derive(Debug)]
pub struct Enclave {
    inner: SpinMutex<Inner>,
}

#[derive(Debug)]
struct Inner {
    eid: Option<EnclaveId>,
    path: Option<PathBuf>,
}

impl Inner {
    const fn new() -> Inner {
        Self {
            eid: None,
            path: None,
        }
    }

    #[inline]
    fn get_id(&self) -> Option<EnclaveId> {
        self.eid
    }

    #[inline]
    fn get_path(&self) -> Option<PathBuf> {
        self.path.as_ref().map(|p| p.to_owned())
    }

    fn set_id(&mut self, eid: EnclaveId) -> io::Result<()> {
        if eid == 0 {
            return Err(io::const_io_error!(io::ErrorKind::InvalidInput, "eid is incorrect"));
        }
        self.eid = Some(eid);
        Ok(())
    }

    fn set_path<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let _ = fs::metadata(&path)?;
        self.path = Some(path.as_ref().to_owned());
        Ok(())
    }

    #[inline]
    unsafe fn set_path_unchecked<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        self.path = Some(path.as_ref().to_owned());
        Ok(())
    }
}

impl Enclave {
    const fn new() -> Enclave {
        Self {
            inner: SpinMutex::new(Inner::new()),
        }
    }
}

impl Enclave {
    ///
    /// get_id is to get enclave ID.
    ///
    pub fn get_id() -> Option<EnclaveId> {
        ENCLAVE.inner.lock().get_id()
    }

    ///
    /// set_id is to set enclave ID.
    ///
    pub fn set_id(eid: EnclaveId) -> io::Result<()> {
        ENCLAVE.inner.lock().set_id(eid)
    }

    ///
    /// get_path is to get the path or name of the enclave.
    ///
    pub fn get_path() -> Option<PathBuf> {
        ENCLAVE.inner.lock().get_path()
    }

    ///
    /// set_path is to set the path or name of the enclave.
    ///
    pub fn set_path<P: AsRef<Path>>(path: P) -> io::Result<()> {
        ENCLAVE.inner.lock().set_path(path)
    }

    pub(crate) unsafe fn set_path_unchecked<P: AsRef<Path>>(path: P) -> io::Result<()> {
        ENCLAVE.inner.lock().set_path_unchecked(path)
    }
}
