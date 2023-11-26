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

use crate::arch::Tcs;
use crate::rand::Rng;
use crate::sync::OnceCell;
use crate::sync::SpinMutex;
use alloc::collections::linked_list::{Iter, LinkedList};
use core::fmt;
use core::num::NonZeroUsize;
use core::ptr::NonNull;

pub static TCS_LIST: SpinMutex<TcsList> = SpinMutex::new(TcsList::new());

#[derive(Debug)]
pub struct TcsList {
    cookie: OnceCell<NonZeroUsize>,
    list: LinkedList<TcsNode>,
}

impl TcsList {
    pub const fn new() -> TcsList {
        TcsList {
            cookie: OnceCell::new(),
            list: LinkedList::new(),
        }
    }

    pub fn save_tcs(&mut self, tcs: NonNull<Tcs>) {
        let node = TcsNode::new(tcs, self.cookie());
        self.list.push_back(node);
    }

    pub fn del_tcs(&mut self, tcs: NonNull<Tcs>) -> Option<NonNull<Tcs>> {
        let node = TcsNode::new(tcs, self.cookie());
        let node = self.list.extract_if(|n| *n == node).next();
        node.map(|n| n.as_tcs(self.cookie()))
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.list.len()
    }

    #[inline]
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    #[inline]
    pub fn list(&self) -> &LinkedList<TcsNode> {
        &self.list
    }

    #[inline]
    pub fn list_mut(&mut self) -> &mut LinkedList<TcsNode> {
        &mut self.list
    }

    pub fn cookie(&self) -> NonZeroUsize {
        *self.cookie.get_or_init(|| loop {
            let r = Rng::new().next_usize();
            if r != 0 {
                break NonZeroUsize::new(r).unwrap();
            }
        })
    }

    pub fn iter(&self) -> TcsIter<'_> {
        TcsIter {
            iter: self.list.iter(),
            len: self.len(),
            cookie: self.cookie(),
        }
    }

    pub fn iter_mut(&mut self) -> TcsIterMut<'_> {
        TcsIterMut { iter: self }
    }
}

impl Default for TcsList {
    #[inline]
    fn default() -> TcsList {
        TcsList::new()
    }
}

impl<'a> IntoIterator for &'a TcsList {
    type Item = NonNull<Tcs>;
    type IntoIter = TcsIter<'a>;

    fn into_iter(self) -> TcsIter<'a> {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut TcsList {
    type Item = NonNull<Tcs>;
    type IntoIter = TcsIterMut<'a>;

    fn into_iter(self) -> TcsIterMut<'a> {
        self.iter_mut()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TcsNode(usize);

impl TcsNode {
    pub fn new(tcs: NonNull<Tcs>, cookie: NonZeroUsize) -> TcsNode {
        TcsNode(tcs.as_ptr() as usize ^ cookie.get())
    }

    pub fn as_tcs(&self, cookie: NonZeroUsize) -> NonNull<Tcs> {
        NonNull::new((self.0 ^ cookie.get()) as *mut Tcs).unwrap()
    }
}

#[derive(Clone)]
pub struct TcsIter<'a> {
    iter: Iter<'a, TcsNode>,
    len: usize,
    cookie: NonZeroUsize,
}

impl fmt::Debug for TcsIter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TcsIter").field(&self.len).finish()
    }
}

impl<'a> Iterator for TcsIter<'a> {
    type Item = NonNull<Tcs>;

    #[inline]
    fn next(&mut self) -> Option<NonNull<Tcs>> {
        self.iter.next().map(|node| node.as_tcs(self.cookie))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn last(mut self) -> Option<NonNull<Tcs>> {
        self.iter.next_back().map(|node| node.as_tcs(self.cookie))
    }
}

pub struct TcsIterMut<'a> {
    iter: &'a mut TcsList,
}

impl fmt::Debug for TcsIterMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TcsIterMut").field(&self.iter.len()).finish()
    }
}

impl<'a> Iterator for TcsIterMut<'a> {
    type Item = NonNull<Tcs>;

    #[inline]
    fn next(&mut self) -> Option<NonNull<Tcs>> {
        self.iter
            .list
            .pop_front()
            .map(|node| node.as_tcs(self.iter.cookie()))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.iter.len(), Some(self.iter.len()))
    }
}
