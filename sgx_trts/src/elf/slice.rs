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

use core::intrinsics::assume;
use core::marker::PhantomData;
use core::mem;

// no relocate
#[repr(C)]
pub union Slice<T> {
    rust: *const [T],
    rust_mut: *mut [T],
    raw: FatPtr<T>,
}

#[repr(C)]
struct FatPtr<T> {
    data: *const T,
    len: usize,
}

impl<T> Clone for FatPtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for FatPtr<T> {}

pub trait AsSlice<T> {
    fn as_slice(&self) -> Slice<T>;
}

impl<T> AsSlice<T> for [T] {
    fn as_slice(&self) -> Slice<T> {
        Slice { rust: self }
    }
}

impl<T> Slice<T> {
    pub const fn as_ptr(&self) -> *const T {
        unsafe { self.rust as *const T }
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        unsafe { self.rust_mut as *mut T }
    }

    pub const fn len(&self) -> usize {
        unsafe { self.raw.len }
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx < self.len() {
            Some(unsafe { self.get_unchecked(idx) })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if idx < self.len() {
            Some(unsafe { self.get_mut_unchecked(idx) })
        } else {
            None
        }
    }

    pub unsafe fn get_unchecked(&self, idx: usize) -> &T {
        let size = if mem::size_of::<T>() == 0 {
            1
        } else {
            mem::size_of::<T>()
        };
        &*((self.as_ptr() as usize + size * idx) as *const T)
    }

    pub unsafe fn get_mut_unchecked(&mut self, idx: usize) -> &mut T {
        let size = if mem::size_of::<T>() == 0 {
            1
        } else {
            mem::size_of::<T>()
        };
        &mut *((self.as_mut_ptr() as usize + size * idx) as *mut T)
    }

    pub fn into_slice<'a>(self, rang: (usize, usize)) -> Option<&'a [T]> {
        if rang.1 > rang.0 && rang.1 < self.len() {
            Some(unsafe { self.into_slice_unchecked(rang) })
        } else {
            None
        }
    }

    pub fn into_mut_slice<'a>(self, rang: (usize, usize)) -> Option<&'a mut [T]> {
        if rang.1 > rang.0 && rang.1 < self.len() {
            Some(unsafe { self.into_mut_slice_unchecked(rang) })
        } else {
            None
        }
    }

    pub unsafe fn into_slice_unchecked<'a>(self, rang: (usize, usize)) -> &'a [T] {
        let start = self.as_ptr() as usize + rang.0;
        let len = rang.1 - rang.0;
        let size = if mem::size_of::<T>() == 0 {
            1
        } else {
            mem::size_of::<T>()
        };
        from_raw_parts(start as *const T, len * size)
    }

    pub unsafe fn into_mut_slice_unchecked<'a>(mut self, rang: (usize, usize)) -> &'a mut [T] {
        let start = self.as_mut_ptr() as usize + rang.0;
        let len = rang.1 - rang.0;
        let size = if mem::size_of::<T>() == 0 {
            1
        } else {
            mem::size_of::<T>()
        };
        from_raw_parts_mut(start as *mut T, len * size)
    }

    pub fn eq(&self, other: &[T]) -> bool {
        let t_len = self.len();
        let other = other.as_slice();
        if t_len != other.len() {
            return false;
        }

        unsafe {
            memcmp(
                self.as_ptr() as *const u8,
                other.as_ptr() as *const u8,
                mem::size_of::<T>() * t_len,
            )
        }
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(self)
    }
}

pub unsafe fn from_raw_parts<'a, T>(data: *const T, len: usize) -> &'a [T] {
    // SAFETY: Accessing the value from the `Repr` union is safe since *const [T]
    // and FatPtr have the same memory layouts. Only std can make this
    // guarantee.
    &*(Slice {
        raw: FatPtr { data, len },
    }
    .rust)
}

pub unsafe fn from_raw_parts_mut<'a, T>(data: *mut T, len: usize) -> &'a mut [T] {
    // SAFETY: Accessing the value from the `Repr` union is safe since *mut [T]
    // and FatPtr have the same memory layouts
    &mut *(Slice {
        raw: FatPtr { data, len },
    }
    .rust_mut)
}

pub fn eq<T>(src: &[T], other: &[T]) -> bool {
    let t_len = src.as_slice().len();
    if t_len != other.as_slice().len() {
        return false;
    }

    unsafe {
        memcmp(
            src.as_slice().as_ptr() as *const u8,
            other.as_slice().as_ptr() as *const u8,
            mem::size_of::<T>() * t_len,
        )
    }
}

unsafe fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> bool {
    if n != 0 {
        let mut i = 0;
        let mut src_ptr = s1 as usize;
        let mut other_ptr = s2 as usize;
        while i < n {
            if *(src_ptr as *const u8) != *(other_ptr as *const u8) {
                return false;
            }
            src_ptr += 1;
            other_ptr += 1;
            i += 1;
        }
    }
    true
}

pub struct Iter<'a, T: 'a> {
    ptr: *const T,
    end: *const T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Iter<'a, T> {
    pub fn new(slice: &'a Slice<T>) -> Self {
        let ptr = slice.as_ptr();
        unsafe {
            assume((ptr as usize) != 0);

            let end = if mem::size_of::<T>() == 0 {
                ((ptr as usize) + slice.len()) as *const T
            } else {
                ((ptr as usize) + slice.len() * mem::size_of::<T>()) as *const T
            };

            Self {
                ptr,
                end,
                _marker: PhantomData,
            }
        }
    }

    #[inline]
    #[allow(clippy::while_let_on_iterator)]
    pub fn for_each<F>(mut self, mut f: F)
    where
        Self: Sized,
        F: FnMut(&'a T),
    {
        while let Some(x) = self.next() {
            f(x);
        }
    }

    #[inline]
    #[allow(clippy::while_let_on_iterator)]
    pub fn all<F>(mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(&'a T) -> bool,
    {
        while let Some(x) = self.next() {
            if !f(x) {
                return false;
            }
        }
        true
    }

    #[inline]
    #[allow(clippy::while_let_on_iterator)]
    pub fn any<F>(mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(&'a T) -> bool,
    {
        while let Some(x) = self.next() {
            if f(x) {
                return true;
            }
        }
        false
    }

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        unsafe {
            assume((self.ptr as usize) != 0);
            if mem::size_of::<T>() != 0 {
                assume((self.end as usize) != 0);
            }

            if self.ptr as usize == self.end as usize {
                None
            } else {
                let old = self.ptr;
                if mem::size_of::<T>() == 0 {
                    self.ptr = (self.ptr as usize + 1) as *const T;
                } else {
                    self.ptr = (self.ptr as usize + mem::size_of::<T>()) as *const T;
                }
                Some(&*old)
            }
        }
    }
}
