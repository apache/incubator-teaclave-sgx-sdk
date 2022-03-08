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

#![allow(dead_code)]

use list::{LinkedList, Node};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr::NonNull;
use std::sync::Arc;

mod list;

pub type NodeRef<T> = Arc<RefCell<T>>;

#[derive(Debug)]
struct LruEntry<T> {
    node_ref: NonNull<Node<u64>>,
    value: NodeRef<T>,
}

impl<T> LruEntry<T> {
    fn new(node_ref: NonNull<Node<u64>>, value: NodeRef<T>) -> LruEntry<T> {
        LruEntry { node_ref, value }
    }
}

pub struct Iter<'a, T: 'a> {
    iter: list::Iter<'a, u64>,
    map: &'a HashMap<u64, LruEntry<T>>,
}

#[derive(Debug)]
pub struct LruCache<T> {
    list: LinkedList<u64>,
    map: HashMap<u64, LruEntry<T>>,
    max_size: usize,
}

impl<T> LruCache<T> {
    #[inline]
    pub fn new(capacity: usize) -> LruCache<T> {
        LruCache {
            list: LinkedList::new(),
            map: HashMap::with_capacity(capacity),
            max_size: capacity,
        }
    }

    /// Return the maximum number of key-value pairs the cache can hold.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.max_size
    }

    /// Returns `true` if the cache is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// Returns the length of the cache.
    #[inline]
    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// Adds an element first in the cache.
    pub fn push(&mut self, key: u64, value: NodeRef<T>) -> bool {
        let is_none = self.map.get(&key).is_none();
        if is_none {
            self.list.push_front(key);
            let node_ref = unsafe { self.list.head_node_ref().unwrap() };
            self.map.insert(key, LruEntry::new(node_ref, value));
        }
        is_none
    }

    /// Returns the value corresponding to the key.
    #[inline]
    pub fn find(&self, key: u64) -> Option<NodeRef<T>> {
        self.map.get(&key).map(|entry| entry.value.clone())
    }

    /// Removes the first element and returns it, or `None` if the cache is emptry.
    pub fn pop_front(&mut self) -> Option<NodeRef<T>> {
        let key = self.list.pop_front()?;
        self.map.remove(&key).map(|entry| entry.value)
    }

    /// Removes the last element and returns it, or `None` if the cache is emptry.
    pub fn pop_back(&mut self) -> Option<NodeRef<T>> {
        let key = self.list.pop_back()?;
        self.map.remove(&key).map(|entry| entry.value)
    }

    /// Provides a reference to the front element, or `None` if the cache is empty.
    pub fn front(&self) -> Option<&NodeRef<T>> {
        let key = self.list.front()?;
        self.map.get(key).map(|entry| &entry.value)
    }

    /// Provides a reference to the back element, or `None` if the cache is empty.
    pub fn back(&self) -> Option<&NodeRef<T>> {
        let key = self.list.back()?;
        self.map.get(key).map(|entry| &entry.value)
    }

    /// Move the element corresponding to the key to the head.
    pub fn move_to_head(&mut self, key: u64) {
        if let Some(entry) = self.map.get_mut(&key) {
            unsafe {
                self.list.move_to_head(entry.node_ref);
            }
        }
    }

    /// Move the element corresponding to the key to the tail.
    pub fn move_to_tail(&mut self, key: u64) {
        if let Some(entry) = self.map.get_mut(&key) {
            unsafe {
                self.list.move_to_tail(entry.node_ref);
            }
        }
    }

    /// Change the number of key-value pairs the cache can hold.
    pub fn change_capacity(&mut self, capacity: usize) {
        if capacity < self.len() {
            for _ in capacity..self.len() {
                let _ = self.pop_back();
            }
        }
        self.max_size = capacity;
    }

    /// Removes all elements from the cache.
    #[inline]
    pub fn clear(&mut self) {
        self.list.clear();
        self.map.clear();
    }

    /// Provides a forward iterator.
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            iter: self.list.iter(),
            map: &self.map,
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a NodeRef<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let key = self.iter.next()?;
        self.map.get(key).map(|entry| &entry.value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T> DoubleEndedIterator for Iter<'_, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let (len, _) = self.iter.size_hint();
        if len == 0 {
            None
        } else {
            let key = self.iter.next_back()?;
            self.map.get(key).map(|entry| &entry.value)
        }
    }
}

pub struct IntoIter<T> {
    cache: LruCache<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = NodeRef<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.cache.pop_front()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.cache.len(), Some(self.cache.len()))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.cache.pop_back()
    }
}

impl<T> IntoIterator for LruCache<T> {
    type Item = NodeRef<T>;
    type IntoIter = IntoIter<T>;

    /// Consumes the list into an iterator yielding elements by value.
    #[inline]
    fn into_iter(self) -> IntoIter<T> {
        IntoIter { cache: self }
    }
}
