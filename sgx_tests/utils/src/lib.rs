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

#![no_std]
#![cfg_attr(target_vendor = "teaclave", feature(rustc_private))]
#![feature(core_intrinsics)]

#[macro_use]
extern crate alloc;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use core::marker::PhantomData;
use core::ops::Deref;
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};

pub use sgx_test_macro::{bench_case, test_case};

mod bench;
mod stats;
mod time;

pub use bench::*;
pub use stats::{Stats, Summary};

pub struct BenchCase(pub String, pub fn(&mut Bencher) -> ());

impl BenchCase {
    #[inline]
    pub fn new(name: &str, func: fn(&mut Bencher) -> ()) -> Self {
        Self(name.to_string(), func)
    }
}

pub struct TestCase(pub String, pub fn() -> ());

impl TestCase {
    #[inline]
    pub fn new(name: &str, func: fn() -> ()) -> Self {
        Self(name.to_string(), func)
    }
}

pub trait Collect: Sized + 'static {
    fn registry() -> &'static Registry<Self>;
}

pub struct Registry<T: 'static> {
    head: AtomicPtr<Node<T>>,
}

impl Collect for TestCase {
    #[inline]
    fn registry() -> &'static Registry<Self> {
        static REGISTRY: Registry<TestCase> = Registry::new();
        &REGISTRY
    }
}

impl Collect for BenchCase {
    #[inline]
    fn registry() -> &'static Registry<Self> {
        static REGISTRY: Registry<BenchCase> = Registry::new();
        &REGISTRY
    }
}

struct Node<T: 'static> {
    value: T,
    next: Option<&'static Node<T>>,
}

pub fn submit<T: Collect>(value: T) {
    T::registry().submit(Box::new(Node { value, next: None }));
}

impl<T: 'static> Registry<T> {
    pub const fn new() -> Self {
        Registry {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    fn submit(&'static self, new: Box<Node<T>>) {
        let mut new = ptr::NonNull::from(Box::leak(new));
        let mut head = self.head.load(Ordering::SeqCst);
        loop {
            // `new` is always a valid Node<T>, and is not yet visible through the registry.
            // `head` is always null or valid &'static Node<T>.
            unsafe { new.as_mut().next = head.as_ref() };
            match self
                .head
                .compare_exchange(head, new.as_ptr(), Ordering::SeqCst, Ordering::SeqCst)
            {
                Ok(_) => return,
                Err(prev) => head = prev,
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct TestIter<T>(PhantomData<T>);

impl<T> TestIter<T> {
    #[inline]
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

fn impl_into_iter<T: Collect>() -> Iter<T> {
    let head = T::registry().head.load(Ordering::SeqCst);
    Iter {
        node: unsafe { head.as_ref() },
    }
}

impl<T: Collect> IntoIterator for TestIter<T> {
    type Item = &'static T;
    type IntoIter = Iter<T>;

    fn into_iter(self) -> Self::IntoIter {
        impl_into_iter()
    }
}

impl<T: Collect> Deref for TestIter<T> {
    type Target = fn() -> Iter<T>;
    fn deref(&self) -> &Self::Target {
        &(impl_into_iter as fn() -> Iter<T>)
    }
}

#[derive(Clone)]
pub struct Iter<T: 'static> {
    node: Option<&'static Node<T>>,
}

impl<T: 'static> Iterator for Iter<T> {
    type Item = &'static T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.node?;
        let value = &node.value;
        self.node = node.next;
        Some(value)
    }
}
