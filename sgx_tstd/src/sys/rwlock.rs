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

use crate::cell::UnsafeCell;
use crate::collections::LinkedList;
use crate::sync::SgxThreadSpinlock;
use crate::sys::mutex;
use crate::thread::rsgx_thread_self;
use crate::time::Duration;

use sgx_libc as libc;
use sgx_trts::enclave::SgxThreadData;
use sgx_types::{sgx_thread_t, SysError, SGX_THREAD_T_NULL};

struct SgxThreadRwLockInner {
    reader_count: u32,
    writer_waiting: u32,
    lock: SgxThreadSpinlock,
    owner: sgx_thread_t,
    reader_queue: LinkedList<sgx_thread_t>,
    writer_queue: LinkedList<sgx_thread_t>,
}

impl SgxThreadRwLockInner {
    const fn new() -> Self {
        SgxThreadRwLockInner {
            reader_count: 0,
            writer_waiting: 0,
            lock: SgxThreadSpinlock::new(),
            owner: SGX_THREAD_T_NULL,
            reader_queue: LinkedList::new(),
            writer_queue: LinkedList::new(),
        }
    }

    unsafe fn read(&mut self) -> SysError {
        let current = rsgx_thread_self();

        self.lock.lock();
        if self.owner == SGX_THREAD_T_NULL {
            self.reader_count += 1;
        } else {
            if self.owner == current {
                self.lock.unlock();
                return Err(libc::EDEADLK);
            }

            self.reader_queue.push_back(current);

            loop {
                self.lock.unlock();
                mutex::thread_wait_event(
                    SgxThreadData::from_raw(current).get_tcs(),
                    Duration::new(u64::MAX, 1_000_000_000 - 1),
                );

                self.lock.lock();
                if self.owner == SGX_THREAD_T_NULL {
                    self.reader_count += 1;
                    if let Some(pos) = self
                        .reader_queue
                        .iter()
                        .position(|&waiter| waiter == current)
                    {
                        self.reader_queue.remove(pos);
                    }
                    break;
                }
            }
        }
        self.lock.unlock();
        Ok(())
    }

    unsafe fn try_read(&mut self) -> SysError {
        self.lock.lock();
        let ret = if self.owner == SGX_THREAD_T_NULL {
            self.reader_count += 1;
            Ok(())
        } else {
            Err(libc::EBUSY)
        };
        self.lock.unlock();
        ret
    }

    unsafe fn write(&mut self) -> SysError {
        let current = rsgx_thread_self();

        self.lock.lock();
        if self.owner == SGX_THREAD_T_NULL && self.reader_count == 0 {
            self.owner = current;
        } else {
            if self.owner == current {
                self.lock.unlock();
                return Err(libc::EDEADLK);
            }

            self.writer_queue.push_back(current);

            loop {
                self.lock.unlock();
                mutex::thread_wait_event(
                    SgxThreadData::from_raw(current).get_tcs(),
                    Duration::new(u64::MAX, 1_000_000_000 - 1),
                );

                self.lock.lock();
                if self.owner == SGX_THREAD_T_NULL && self.reader_count == 0 {
                    self.owner = current;
                    if let Some(pos) = self
                        .writer_queue
                        .iter()
                        .position(|&waiter| waiter == current)
                    {
                        self.writer_queue.remove(pos);
                    }
                    break;
                }
            }
        }
        self.lock.unlock();
        Ok(())
    }

    unsafe fn try_write(&mut self) -> SysError {
        let current = rsgx_thread_self();

        self.lock.lock();
        let ret = if self.owner == SGX_THREAD_T_NULL && self.reader_count == 0 {
            self.owner = current;
            Ok(())
        } else {
            Err(libc::EBUSY)
        };
        self.lock.unlock();
        ret
    }

    unsafe fn read_unlock(&mut self) -> SysError {
        self.lock.lock();

        if self.reader_count == 0 {
            self.lock.unlock();
            return Err(libc::EPERM);
        }

        self.reader_count -= 1;
        if self.reader_count == 0 {
            let waiter = self.reader_queue.front();
            self.lock.unlock();
            if let Some(td) = waiter {
                mutex::thread_set_event(SgxThreadData::from_raw(*td).get_tcs());
            }
        } else {
            self.lock.unlock();
        }
        Ok(())
    }

    unsafe fn write_unlock(&mut self) -> SysError {
        let current = rsgx_thread_self();

        self.lock.lock();

        if self.owner != current {
            self.lock.unlock();
            return Err(libc::EPERM);
        }

        self.owner = SGX_THREAD_T_NULL;
        if !self.reader_queue.is_empty() {
            let mut tcs_vec: Vec<usize> = Vec::new();
            for waiter in self.reader_queue.iter() {
                tcs_vec.push(SgxThreadData::from_raw(*waiter).get_tcs())
            }
            self.lock.unlock();
            mutex::thread_set_multiple_events(tcs_vec.as_slice());
        } else {
            let waiter = self.writer_queue.front();
            self.lock.unlock();
            if let Some(td) = waiter {
                mutex::thread_set_event(SgxThreadData::from_raw(*td).get_tcs());
            }
        }
        Ok(())
    }

    unsafe fn unlock(&mut self) -> SysError {
        if self.owner == rsgx_thread_self() {
            self.write_unlock()
        } else {
            self.read_unlock()
        }
    }

    unsafe fn destroy(&mut self) -> SysError {
        self.lock.lock();
        let ret = if self.owner != SGX_THREAD_T_NULL
            || self.reader_count != 0
            || self.writer_waiting != 0
            || !self.reader_queue.is_empty()
            || !self.writer_queue.is_empty()
        {
            Err(libc::EBUSY)
        } else {
            Ok(())
        };
        self.lock.unlock();
        ret
    }
}

pub type SgxMovableThreadRwLock = Box<SgxThreadRwLock>;

unsafe impl Send for SgxThreadRwLock {}
unsafe impl Sync for SgxThreadRwLock {}

/// An OS-based reader-writer lock.
///
/// This structure is entirely unsafe and serves as the lowest layer of a
/// cross-platform binding of system rwlocks. It is recommended to use the
/// safer types at the top level of this crate instead of this type.
pub struct SgxThreadRwLock {
    lock: UnsafeCell<SgxThreadRwLockInner>,
}

impl SgxThreadRwLock {
    /// Creates a new reader-writer lock for use.
    pub const fn new() -> Self {
        SgxThreadRwLock {
            lock: UnsafeCell::new(SgxThreadRwLockInner::new()),
        }
    }

    /// Acquires shared access to the underlying lock, blocking the current
    /// thread to do so.
    #[inline]
    pub unsafe fn read(&self) -> SysError {
        let rwlock: &mut SgxThreadRwLockInner = &mut *self.lock.get();
        rwlock.read()
    }

    /// Attempts to acquire shared access to this lock, returning whether it
    /// succeeded or not.
    ///
    /// This function does not block the current thread.
    #[inline]
    pub unsafe fn try_read(&self) -> SysError {
        let rwlock: &mut SgxThreadRwLockInner = &mut *self.lock.get();
        rwlock.try_read()
    }

    /// Acquires write access to the underlying lock, blocking the current thread
    /// to do so.
    #[inline]
    pub unsafe fn write(&self) -> SysError {
        let rwlock: &mut SgxThreadRwLockInner = &mut *self.lock.get();
        rwlock.write()
    }

    /// Attempts to acquire exclusive access to this lock, returning whether it
    /// succeeded or not.
    ///
    /// This function does not block the current thread.
    #[inline]
    pub unsafe fn try_write(&self) -> SysError {
        let rwlock: &mut SgxThreadRwLockInner = &mut *self.lock.get();
        rwlock.try_write()
    }

    /// Unlocks previously acquired shared access to this lock.
    #[inline]
    pub unsafe fn read_unlock(&self) -> SysError {
        let rwlock: &mut SgxThreadRwLockInner = &mut *self.lock.get();
        rwlock.read_unlock()
    }

    /// Unlocks previously acquired exclusive access to this lock.
    #[inline]
    pub unsafe fn write_unlock(&self) -> SysError {
        let rwlock: &mut SgxThreadRwLockInner = &mut *self.lock.get();
        rwlock.write_unlock()
    }

    #[inline]
    pub unsafe fn unlock(&self) -> SysError {
        let rwlock: &mut SgxThreadRwLockInner = &mut *self.lock.get();
        rwlock.unlock()
    }

    /// Destroys OS-related resources with this RWLock.
    #[inline]
    pub unsafe fn destroy(&self) -> SysError {
        let rwlock: &mut SgxThreadRwLockInner = &mut *self.lock.get();
        rwlock.destroy()
    }
}
