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

use crate::enclave::is_within_enclave;
use crate::thread::tls::bitset::Bitset;
use alloc::collections::linked_list::LinkedList;
use core::cell::{Cell, RefCell};
use core::mem;
use core::num::NonZeroUsize;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};
use sgx_types::error::{SgxResult, SgxStatus};

const USIZE_BITS: usize = 64;
const TLS_KEYS: usize = 256;
const TLS_KEYS_BITSET_SIZE: usize = (TLS_KEYS + (USIZE_BITS - 1)) / USIZE_BITS;

#[thread_local]
static TLS_STORAGE: Storage = Storage::new();

static TLS_KEY_IN_USE: Bitset = Bitset::new();

macro_rules! dup {
    ((* $($exp:tt)*) $($val:tt)*) => (dup!( ($($exp)*) $($val)* $($val)* ));
    (() $($val:tt)*) => ([$($val),*])
}
static TLS_DESTRUCTOR: [AtomicUsize; TLS_KEYS] = dup!((* * * * * * * *) (AtomicUsize::new(0)));

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct Key(NonZeroUsize);

impl Key {
    fn to_index(self) -> usize {
        self.0.get() - 1
    }

    fn from_index(index: usize) -> Key {
        Key(NonZeroUsize::new(index + 1).unwrap())
    }

    pub unsafe fn from_usize(index: usize) -> Option<Key> {
        if index > 0 && index <= TLS_KEYS {
            Some(Key(NonZeroUsize::new_unchecked(index)))
        } else {
            None
        }
    }

    pub fn as_usize(self) -> usize {
        self.0.get()
    }
}

#[derive(Clone, Debug)]
struct StorageNode {
    key: Key,
    value: Cell<*mut u8>,
}

#[derive(Default)]
struct Storage {
    data: RefCell<LinkedList<StorageNode>>,
}

impl Storage {
    const fn new() -> Storage {
        Storage {
            data: RefCell::new(LinkedList::new()),
        }
    }
}

pub struct Tls;

impl Tls {
    pub(crate) fn init() {
        TLS_STORAGE.data.borrow_mut().clear();
    }

    pub fn create(dtor: Option<unsafe extern "C" fn(*mut u8)>) -> SgxResult<Key> {
        if let Some(f) = dtor {
            ensure!(
                is_within_enclave(f as *const u8, 0),
                SgxStatus::InvalidParameter
            );
        }

        let index = if let Some(index) = TLS_KEY_IN_USE.set() {
            index
        } else {
            bail!(SgxStatus::Unexpected);
        };
        TLS_DESTRUCTOR[index].store(dtor.map_or(0, |f| f as usize), Ordering::Relaxed);
        Ok(Key::from_index(index))
    }

    pub fn set(key: Key, value: *mut u8) -> SgxResult {
        ensure!(
            TLS_KEY_IN_USE.get(key.to_index()),
            SgxStatus::InvalidParameter
        );

        let tls_storage = unsafe { &mut *TLS_STORAGE.data.as_ptr() };
        if let Some(storage) = tls_storage.iter().find(|s| s.key == key) {
            storage.value.set(value);
        } else {
            tls_storage.push_back(StorageNode {
                key,
                value: Cell::new(value),
            });
        }
        Ok(())
    }

    pub fn get(key: Key) -> SgxResult<Option<*mut u8>> {
        ensure!(
            TLS_KEY_IN_USE.get(key.to_index()),
            SgxStatus::InvalidParameter
        );

        if let Some(storage) = TLS_STORAGE.data.borrow().iter().find(|s| s.key == key) {
            Ok(Some(storage.value.get()))
        } else {
            Ok(None)
        }
    }

    pub fn destroy(key: Key) {
        TLS_KEY_IN_USE.clear(key.to_index());
    }

    pub(crate) fn activate<'a>() -> ActiveTls<'a> {
        let ptr = &TLS_STORAGE as *const Storage;
        ActiveTls {
            tls: unsafe { &*ptr },
        }
    }
}

pub(crate) struct ActiveTls<'a> {
    tls: &'a Storage,
}

impl<'a> !Send for ActiveTls<'a> {}

impl<'a> Drop for ActiveTls<'a> {
    fn drop(&mut self) {
        let value_with_destructor = |storage: &'a StorageNode| {
            let index = storage.key.to_index();
            if TLS_KEY_IN_USE.get(index) {
                let ptr = TLS_DESTRUCTOR[index].load(Ordering::Relaxed);
                unsafe { mem::transmute::<_, Option<unsafe extern "C" fn(*mut u8)>>(ptr) }
                    .map(|dtor| (&storage.value, dtor))
            } else {
                None
            }
        };
        let tls_storage = self.tls.data.borrow();

        let mut any_non_null_dtor = true;
        while any_non_null_dtor {
            any_non_null_dtor = false;
            for (value, dtor) in tls_storage.iter().filter_map(&value_with_destructor) {
                let value = value.replace(ptr::null_mut());
                if !value.is_null() {
                    any_non_null_dtor = true;
                    unsafe { dtor(value) }
                }
            }
        }

        drop(tls_storage);
        self.tls.data.take();
    }
}

mod bitset;
