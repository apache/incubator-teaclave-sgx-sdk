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

use super::*;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::mem::{self, ManuallyDrop};
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};
use sgx_ffi::c_str::{CStr, CString};
use sgx_ffi::memchr;
use sgx_sync::StaticRwLock;
use sgx_trts::trts::{is_within_enclave, EnclaveRange};
use sgx_types::error::OsResult;

pub unsafe fn getuid() -> OCallResult<uid_t> {
    let mut result: uid_t = 0;

    let status = u_getuid_ocall(&mut result as *mut uid_t);
    ensure!(status.is_success(), esgx!(status));
    Ok(result)
}

pub unsafe fn getgid() -> OCallResult<gid_t> {
    let mut result: gid_t = 0;

    let status = u_getgid_ocall(&mut result as *mut gid_t);
    ensure!(status.is_success(), esgx!(status));
    Ok(result)
}

pub unsafe fn getcwd() -> OCallResult<CString> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let mut buf = Vec::with_capacity(512);

    loop {
        let bufsz = buf.capacity();
        let status = u_getcwd_ocall(
            &mut result as *mut c_int,
            &mut error as *mut c_int,
            buf.as_mut_ptr() as *mut c_char,
            bufsz,
        );

        ensure!(status.is_success(), esgx!(status));

        if result == 0 {
            buf.set_len(bufsz);
            let value = shrink_to_fit_cstring(buf)?;
            return Ok(value);
        } else if error != ERANGE {
            bail!(eos!(error));
        }

        // Trigger the internal buffer resizing logic of `Vec` by requiring
        // more space than the current capacity.
        buf.set_len(bufsz);
        buf.reserve(1);
    }
}

pub unsafe fn chdir(dir: &CStr) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_chdir_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        dir.as_ptr(),
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn env() -> OCallResult<Vec<CString>> {
    let _guard = ENV_LOCK.read();

    let env_ptr = ENVIRON.load(Ordering::SeqCst);
    ensure!(
        !env_ptr.is_null(),
        ecust!("Environment variables are not initialized")
    );

    let os_env = ManuallyDrop::new(Box::from_raw(env_ptr));
    Ok(os_env.env())
}

pub unsafe fn getenv(name: &CStr) -> OCallResult<Option<CString>> {
    check_enclave_buffer(name.to_bytes_with_nul()).map_err(|_| eos!(EINVAL))?;

    let _guard = ENV_LOCK.read();

    let env_ptr = ENVIRON.load(Ordering::SeqCst);
    ensure!(
        !env_ptr.is_null(),
        ecust!("Environment variables are not initialized")
    );

    let os_env = ManuallyDrop::new(Box::from_raw(env_ptr));
    os_env.get(name).map_err(|e| eos!(e))
}

pub unsafe fn getenv_ref(name: &CStr) -> OCallResult<Option<&'static CStr>> {
    check_enclave_buffer(name.to_bytes_with_nul()).map_err(|_| eos!(EINVAL))?;

    let _guard = ENV_LOCK.read();

    let env_ptr = ENVIRON.load(Ordering::SeqCst);
    ensure!(
        !env_ptr.is_null(),
        ecust!("Environment variables are not initialized")
    );

    let os_env = ManuallyDrop::new(Box::from_raw(env_ptr));
    os_env
        .get_ptr(name)
        .map(|v| v.map(|ptr| CStr::from_ptr(ptr)))
        .map_err(|e| eos!(e))
}

pub unsafe fn setenv(name: &CStr, value: &CStr, overwrite: i32) -> OCallResult<()> {
    check_enclave_buffer(name.to_bytes_with_nul()).map_err(|_| eos!(EINVAL))?;
    check_enclave_buffer(value.to_bytes_with_nul()).map_err(|_| eos!(EINVAL))?;

    let _guard = ENV_LOCK.write();

    let env_ptr = ENVIRON.load(Ordering::SeqCst);
    ensure!(
        !env_ptr.is_null(),
        ecust!("Environment variables are not initialized")
    );

    let mut os_env = ManuallyDrop::new(Box::from_raw(env_ptr));

    let overwrite = overwrite > 0;
    os_env.set(name, value, overwrite).map_err(|e| eos!(e))
}

pub unsafe fn unsetenv(name: &CStr) -> OCallResult<()> {
    check_enclave_buffer(name.to_bytes_with_nul()).map_err(|_| eos!(EINVAL))?;

    let _guard = ENV_LOCK.write();

    let env_ptr = ENVIRON.load(Ordering::SeqCst);
    ensure!(
        !env_ptr.is_null(),
        ecust!("Environment variables are not initialized")
    );

    let mut os_env = ManuallyDrop::new(Box::from_raw(env_ptr));
    os_env.remove(name).map_err(|e| eos!(e))
}

pub unsafe fn initenv(env: Option<Vec<CString>>) -> OCallResult<()> {
    if !ENVIRON.load(Ordering::SeqCst).is_null() {
        return Ok(());
    }

    let env = match env {
        Some(env) => env,
        None => {
            #[cfg(feature = "init_env")]
            {
                env_ocall()?
            }
            #[cfg(not(feature = "init_env"))]
            {
                bail!(eos!(EINVAL));
            }
        }
    };

    let _guard = ENV_LOCK.write();

    if ENVIRON.load(Ordering::SeqCst).is_null() {
        let os_env = Box::new(OsEnviron::new(env, &mut environ).map_err(|e| eos!(e))?);
        ENVIRON.store(Box::into_raw(os_env), Ordering::SeqCst);
    }

    return Ok(());

    #[allow(dead_code)]
    unsafe fn env_ocall() -> OCallResult<Vec<CString>> {
        let mut result: ssize_t = 0;
        let mut error: c_int = 0;
        let mut buf: Vec<u8> = Vec::with_capacity(4096);

        loop {
            let bufsz = buf.capacity();
            let status = u_env_ocall(
                &mut result as *mut ssize_t,
                &mut error as *mut c_int,
                buf.as_mut_ptr() as *mut c_uchar,
                bufsz,
            );

            ensure!(status.is_success(), esgx!(status));

            if result >= 0 {
                let buf_read = result as usize;
                ensure!(buf_read <= bufsz, ecust!("Malformed return value."));
                buf.set_len(buf_read);
                buf.shrink_to_fit();
                break;
            } else if error != ERANGE {
                bail!(eos!(error));
            }

            // Trigger the internal buffer resizing logic of `Vec` by requiring
            // more space than the current capacity.
            buf.set_len(bufsz);
            buf.reserve(1);
        }

        let env = buf
            .split(|&c| c == 0)
            .filter_map(|var| {
                if !var.is_empty() {
                    CString::new(var).ok()
                } else {
                    None
                }
            })
            .collect();
        Ok(env)
    }
}

pub unsafe fn args() -> OCallResult<Vec<CString>> {
    let _guard = ARG_LOCK.read();

    let args_ptr = ARGS.load(Ordering::SeqCst);
    ensure!(
        !args_ptr.is_null(),
        ecust!("Command line arguments are not initialized")
    );

    let os_args = ManuallyDrop::new(Box::from_raw(args_ptr));
    Ok(os_args.args())
}

pub unsafe fn argc() -> OCallResult<usize> {
    let _guard = ARG_LOCK.read();

    let args_ptr = ARGS.load(Ordering::SeqCst);
    ensure!(
        !args_ptr.is_null(),
        ecust!("Command line arguments are not initialized")
    );

    let os_args = ManuallyDrop::new(Box::from_raw(args_ptr));
    Ok(os_args.argc())
}

pub unsafe fn argv() -> OCallResult<*const *const c_char> {
    let _guard = ARG_LOCK.read();

    let args_ptr = ARGS.load(Ordering::SeqCst);
    ensure!(
        !args_ptr.is_null(),
        ecust!("Command line arguments are not initialized")
    );

    let os_args = ManuallyDrop::new(Box::from_raw(args_ptr));
    Ok(os_args.as_ptr())
}

pub unsafe fn initargs(args: Option<Vec<CString>>) -> OCallResult<()> {
    if !ARGS.load(Ordering::SeqCst).is_null() {
        return Ok(());
    }

    let args = match args {
        Some(args) => args,
        None => {
            #[cfg(feature = "init_env")]
            {
                args_ocall()?
            }
            #[cfg(not(feature = "init_env"))]
            {
                bail!(eos!(EINVAL));
            }
        }
    };

    let _guard = ARG_LOCK.write();

    if ARGS.load(Ordering::SeqCst).is_null() {
        let os_args = Box::new(OsArgs::new(args).map_err(|e| eos!(e))?);
        ARGS.store(Box::into_raw(os_args), Ordering::SeqCst);
    }

    return Ok(());

    #[allow(dead_code)]
    unsafe fn args_ocall() -> OCallResult<Vec<CString>> {
        let mut result: ssize_t = 0;
        let mut error: c_int = 0;
        let mut buf: Vec<u8> = Vec::with_capacity(4096);

        loop {
            let bufsz = buf.capacity();
            let status = u_args_ocall(
                &mut result as *mut ssize_t,
                &mut error as *mut c_int,
                buf.as_mut_ptr() as *mut c_uchar,
                bufsz,
            );

            ensure!(status.is_success(), esgx!(status));

            if result >= 0 {
                let buf_read = result as usize;
                ensure!(buf_read <= bufsz, ecust!("Malformed return value."));
                buf.set_len(buf_read);
                buf.shrink_to_fit();
                break;
            } else if error != ERANGE {
                bail!(eos!(error));
            }

            // Trigger the internal buffer resizing logic of `Vec` by requiring
            // more space than the current capacity.
            buf.set_len(bufsz);
            buf.reserve(1);
        }

        let args = buf
            .split(|&c| c == 0)
            .filter_map(|arg| {
                if !arg.is_empty() {
                    CString::new(arg).ok()
                } else {
                    None
                }
            })
            .collect();

        Ok(args)
    }
}

#[no_mangle]
pub static mut environ: *const *const c_char = ptr::null();

static ENV_LOCK: StaticRwLock = StaticRwLock::new();
static mut ENVIRON: AtomicPtr<OsEnviron> = AtomicPtr::new(ptr::null_mut());

struct OsEnviron<'a> {
    env: Vec<CString>,
    ptrs: Vec<*const c_char>,
    c_environ: &'a mut *const *const c_char,
}

impl<'a> OsEnviron<'a> {
    fn new(env: Vec<CString>, c_environ: &'a mut *const *const c_char) -> OsResult<Self> {
        if !env.is_empty() {
            ensure!(
                is_within_enclave(env.as_ptr() as *const u8, env.capacity()),
                EINVAL
            );
        }

        let mut ptrs = Vec::with_capacity((env.len() + 1) * mem::size_of::<*const c_char>());
        for ptr in env.iter().filter_map(|var| {
            let bytes = var.as_bytes();
            if !bytes.is_empty() && var.as_bytes_with_nul().is_enclave_range() {
                Some(bytes.as_ptr().cast())
            } else {
                None
            }
        }) {
            ptrs.push(ptr);
        }
        ptrs.push(ptr::null());
        *c_environ = ptrs.as_ptr();

        Ok(Self {
            env,
            ptrs,
            c_environ,
        })
    }

    #[inline]
    fn env(&self) -> Vec<CString> {
        self.env.clone()
    }

    fn get(&self, key: &CStr) -> OsResult<Option<CString>> {
        ensure!(check(key.to_bytes()), EINVAL);

        Ok(self.env.iter().find_map(|var| {
            parse(var.as_bytes())
                .filter(|&(k, _)| k == key.to_bytes())
                .map(|(_, v)| CString::new(v).unwrap())
        }))
    }

    fn get_ptr(&self, key: &CStr) -> OsResult<Option<*const c_char>> {
        ensure!(check(key.to_bytes()), EINVAL);

        Ok(self.env.iter().find_map(|var| {
            parse(var.as_bytes())
                .filter(|&(k, _)| k == key.to_bytes())
                .map(|(_, v)| v.as_ptr().cast())
        }))
    }

    fn get_pos(&self, key: &CStr) -> OsResult<Option<usize>> {
        ensure!(check(key.to_bytes()), EINVAL);

        Ok(self.env.iter().enumerate().find_map(|(idx, env)| {
            parse(env.as_bytes())
                .filter(|&(k, _)| k == key.to_bytes())
                .map(|(_, _)| idx)
        }))
    }

    fn remove(&mut self, key: &CStr) -> OsResult {
        if let Some(pos) = self.get_pos(key)? {
            self.ptrs.remove(pos);
            self.env.remove(pos);
        }
        Ok(())
    }

    fn set(&mut self, key: &CStr, value: &CStr, overwrite: bool) -> OsResult {
        let pos = self.get_pos(key)?;
        if !overwrite && pos.is_some() {
            return Ok(());
        }

        let env = {
            let mut bytes = Vec::with_capacity(key.to_bytes().len() + value.to_bytes().len() + 1);
            bytes.extend_from_slice(key.to_bytes());
            bytes.push(b'=');
            bytes.extend_from_slice(value.to_bytes());
            CString::new(bytes).unwrap()
        };
        let c_ptr = env.as_ptr().cast();

        if let Some(pos) = pos {
            self.env[pos] = env;
            self.ptrs[pos] = c_ptr;
        } else {
            self.env.push(env);
            self.ptrs.insert(self.ptrs.len() - 1, c_ptr);
            *self.c_environ = self.ptrs.as_ptr();
        }

        Ok(())
    }
}

impl<'a> Drop for OsEnviron<'a> {
    fn drop(&mut self) {
        *self.c_environ = ptr::null();
    }
}

static ARG_LOCK: StaticRwLock = StaticRwLock::new();
static mut ARGS: AtomicPtr<OsArgs> = AtomicPtr::new(ptr::null_mut());

struct OsArgs {
    args: Vec<CString>,
    ptrs: Vec<*const c_char>,
}

impl OsArgs {
    fn new(args: Vec<CString>) -> OsResult<Self> {
        if !args.is_empty() {
            ensure!(
                args.is_empty() || is_within_enclave(args.as_ptr() as *const u8, args.capacity()),
                EINVAL
            );
        }

        let mut ptrs = Vec::with_capacity((args.len() + 1) * mem::size_of::<*const c_char>());
        for ptr in args.iter().filter_map(|arg| {
            let bytes = arg.as_bytes();
            if !bytes.is_empty() && arg.as_bytes_with_nul().is_enclave_range() {
                Some(bytes.as_ptr().cast())
            } else {
                None
            }
        }) {
            ptrs.push(ptr);
        }
        ptrs.push(ptr::null());

        Ok(Self { args, ptrs })
    }

    #[inline]
    fn args(&self) -> Vec<CString> {
        self.args.clone()
    }

    #[inline]
    fn argc(&self) -> usize {
        self.args.len()
    }

    #[inline]
    fn as_ptr(&self) -> *const *const c_char {
        self.ptrs.as_ptr()
    }
}

fn check(input: &[u8]) -> bool {
    if input.is_empty() {
        return false;
    }
    !input.iter().any(|&c| c == b'=')
}

fn parse(input: &[u8]) -> Option<(&[u8], &[u8])> {
    if input.is_empty() {
        return None;
    }
    let pos = memchr::memchr(b'=', &input[1..]).map(|p| p + 1);
    pos.map(|p| (&input[..p], &input[p + 1..]))
}
