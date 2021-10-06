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

use core::cell::UnsafeCell;
use crate::sync::{SgxThreadCondvar, SgxThreadMutex, SgxThreadSpinlock};
use crate::thread;
use sgx_trts::libc;
use sgx_types::{sgx_thread_t, SysError, SGX_THREAD_T_NULL};

struct SgxThreadRwLockInner {
    readers_num: u32,
    writers_num: u32,
    busy: u32,
    writer_thread: sgx_thread_t,
    condvar: SgxThreadCondvar,
    mutex: SgxThreadMutex,
    spinlock: SgxThreadSpinlock,
}

impl SgxThreadRwLockInner {
    const fn new() -> Self {
        SgxThreadRwLockInner {
            readers_num: 0,
            writers_num: 0,
            busy: 0,
            writer_thread: SGX_THREAD_T_NULL,
            condvar: SgxThreadCondvar::new(),
            mutex: SgxThreadMutex::new(),
            spinlock: SgxThreadSpinlock::new(),
        }
    }

    unsafe fn ref_busy(&mut self) -> SysError {
        let ret: SysError;
        self.spinlock.lock();
        {
            if self.busy == u32::MAX {
                ret = Err(libc::EAGAIN);
            } else {
                self.busy += 1;
                ret = Ok(());
            }
        }
        self.spinlock.unlock();
        ret
    }

    unsafe fn deref_busy(&mut self) -> SysError {
        let ret: SysError;
        self.spinlock.lock();
        {
            if self.busy == 0 {
                ret = Err(libc::EAGAIN);
            } else {
                self.busy -= 1;
                ret = Ok(());
            }
        }
        self.spinlock.unlock();
        ret
    }

    unsafe fn read(&mut self) -> SysError {
        self.ref_busy()?;
        self.mutex.lock();
        {
            if self.writer_thread == thread::rsgx_thread_self() {
                self.mutex.unlock();
                self.deref_busy();
                return Err(libc::EDEADLK);
            }
            if self.readers_num == u32::MAX {
                self.mutex.unlock();
                self.deref_busy();
                return Err(libc::EAGAIN);
            }
            while self.writers_num > 0 {
                self.condvar.wait(&self.mutex);
            }
            self.readers_num += 1;
        }
        self.mutex.unlock();
        self.deref_busy();
        Ok(())
    }

    unsafe fn try_read(&mut self) -> SysError {
        self.ref_busy()?;
        self.mutex.lock();
        {
            let mut ret = Ok(());
            if self.writer_thread == thread::rsgx_thread_self() {
                ret = Err(libc::EDEADLK);
            } else if self.readers_num == u32::MAX {
                ret = Err(libc::EAGAIN);
            } else if self.writers_num > 0 {
                ret = Err(libc::EBUSY);
            }
            match ret {
                Ok(_) => {}
                Err(e) => {
                    self.mutex.unlock();
                    self.deref_busy();
                    return Err(e);
                }
            }
            self.readers_num += 1;
        }
        self.mutex.unlock();
        self.deref_busy();
        Ok(())
    }

    unsafe fn write(&mut self) -> SysError {
        self.ref_busy()?;
        self.mutex.lock();
        {
            if self.writer_thread == thread::rsgx_thread_self() {
                self.mutex.unlock();
                self.deref_busy();
                return Err(libc::EDEADLK);
            }

            if self.writers_num == u32::MAX {
                self.mutex.unlock();
                self.deref_busy();
                return Err(libc::EAGAIN);
            }

            self.writers_num += 1;
            while self.readers_num > 0 {
                self.condvar.wait(&self.mutex);
            }
            while self.writer_thread != SGX_THREAD_T_NULL {
                self.condvar.wait(&self.mutex);
            }
            self.writer_thread = thread::rsgx_thread_self();
        }
        self.mutex.unlock();
        self.deref_busy();
        Ok(())
    }

    pub unsafe fn try_write(&mut self) -> SysError {
        self.ref_busy()?;
        self.mutex.lock();
        {
            let mut ret = Ok(());
            if self.writer_thread == thread::rsgx_thread_self() {
                ret = Err(libc::EDEADLK);
            } else if self.writers_num == u32::MAX {
                ret = Err(libc::EAGAIN);
            } else if self.readers_num > 0 || self.writer_thread != SGX_THREAD_T_NULL {
                ret = Err(libc::EBUSY);
            }

            match ret {
                Ok(_) => {}
                Err(e) => {
                    self.mutex.unlock();
                    self.deref_busy();
                    return Err(e);
                }
            }
            self.writers_num += 1;
            self.writer_thread = thread::rsgx_thread_self();
        }
        self.mutex.unlock();
        self.deref_busy();
        Ok(())
    }

    unsafe fn read_unlock(&mut self) -> SysError {
        self.raw_unlock()
    }

    unsafe fn write_unlock(&mut self) -> SysError {
        self.raw_unlock()
    }

    unsafe fn raw_unlock(&mut self) -> SysError {
        self.mutex.lock();
        {
            if self.readers_num > 0 {
                self.readers_num -= 1;
                if self.readers_num == 0 && self.writers_num > 0 {
                    self.condvar.broadcast();
                }
            } else {
                if self.writer_thread != thread::rsgx_thread_self() {
                    self.mutex.unlock();
                    return Err(libc::EPERM);
                }
                self.writers_num -= 1;
                self.writer_thread = SGX_THREAD_T_NULL;
                if self.busy > 0 {
                    self.condvar.broadcast();
                }
            }
        }
        self.mutex.unlock();
        Ok(())
    }

    unsafe fn destroy(&mut self) -> SysError {
        self.mutex.lock();
        {
            if self.readers_num > 0 || self.writers_num > 0 || self.busy > 0 {
                self.spinlock.unlock();
                return Err(libc::EBUSY);
            }

            self.condvar.destroy();
            self.mutex.destroy();
        }
        self.spinlock.unlock();
        Ok(())
    }
}

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

    /// Destroys OS-related resources with this RWLock.
    #[inline]
    pub unsafe fn destroy(&self) -> SysError {
        let rwlock: &mut SgxThreadRwLockInner = &mut *self.lock.get();
        rwlock.destroy()
    }
}
