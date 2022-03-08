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

use crate::rand::Rng;
use crate::sync::SpinMutex;
use crate::veh::{ExceptionHandler, Handle};
use alloc::collections::linked_list::{Iter, LinkedList};
use core::fmt;
use core::lazy::OnceCell;
use core::mem;
use core::num::NonZeroUsize;

pub static EXCEPTION_LIST: SpinMutex<ExceptionList> = SpinMutex::new(ExceptionList::new());

#[derive(Debug)]
pub struct ExceptionList {
    cookie: OnceCell<NonZeroUsize>,
    list: LinkedList<ExceptionNode>,
}

impl ExceptionList {
    pub const fn new() -> ExceptionList {
        ExceptionList {
            cookie: OnceCell::new(),
            list: LinkedList::new(),
        }
    }

    pub fn push_back(&mut self, f: ExceptionHandler) -> Handle {
        let node = ExceptionNode::new(f, self.cookie());
        self.list.push_back(node);
        node.id
    }

    pub fn push_front(&mut self, f: ExceptionHandler) -> Handle {
        let node = ExceptionNode::new(f, self.cookie());
        self.list.push_front(node);
        node.id
    }

    pub fn remove(&mut self, id: Handle) -> Option<ExceptionNode> {
        self.list.drain_filter(|node| node.id() == id).next()
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
    pub fn list(&self) -> &LinkedList<ExceptionNode> {
        &self.list
    }

    pub fn cookie(&self) -> NonZeroUsize {
        *self.cookie.get_or_init(|| loop {
            let r = Rng::new().next_usize();
            if r != 0 {
                break NonZeroUsize::new(r).unwrap();
            }
        })
    }

    pub fn iter(&self) -> ExceptionIter<'_> {
        ExceptionIter {
            iter: self.list.iter(),
            len: self.len(),
            cookie: self.cookie(),
        }
    }
}

impl Default for ExceptionList {
    #[inline]
    fn default() -> ExceptionList {
        ExceptionList::new()
    }
}

impl IntoIterator for ExceptionList {
    type Item = ExceptionHandler;
    type IntoIter = ExceptionIntoIter;

    #[inline]
    fn into_iter(self) -> ExceptionIntoIter {
        ExceptionIntoIter { list: self }
    }
}

impl<'a> IntoIterator for &'a ExceptionList {
    type Item = ExceptionHandler;
    type IntoIter = ExceptionIter<'a>;

    fn into_iter(self) -> ExceptionIter<'a> {
        self.iter()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ExceptionNode {
    id: Handle,
    f: usize,
}

impl ExceptionNode {
    pub fn new(f: ExceptionHandler, cookie: NonZeroUsize) -> ExceptionNode {
        ExceptionNode {
            f: f as usize ^ cookie.get(),
            id: Handle::new(),
        }
    }

    pub fn handler(&self, cookie: NonZeroUsize) -> ExceptionHandler {
        unsafe { mem::transmute(self.f ^ cookie.get()) }
    }

    pub fn id(&self) -> Handle {
        self.id
    }
}

#[derive(Clone)]
pub struct ExceptionIter<'a> {
    iter: Iter<'a, ExceptionNode>,
    len: usize,
    cookie: NonZeroUsize,
}

impl fmt::Debug for ExceptionIter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ExceptionIter").field(&self.len).finish()
    }
}

impl<'a> Iterator for ExceptionIter<'a> {
    type Item = ExceptionHandler;

    #[inline]
    fn next(&mut self) -> Option<ExceptionHandler> {
        self.iter.next().map(|node| node.handler(self.cookie))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn last(mut self) -> Option<ExceptionHandler> {
        self.iter.next_back().map(|node| node.handler(self.cookie))
    }
}

pub struct ExceptionIntoIter {
    list: ExceptionList,
}

impl Iterator for ExceptionIntoIter {
    type Item = ExceptionHandler;

    #[inline]
    fn next(&mut self) -> Option<ExceptionHandler> {
        self.list
            .list
            .pop_front()
            .map(|node| node.handler(self.list.cookie()))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.list.len(), Some(self.list.len()))
    }
}
