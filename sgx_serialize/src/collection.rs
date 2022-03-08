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

//! Implementations of serialization for structures found in liballoc

use crate::{Decodable, Decoder, Encodable, Encoder};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque};
use std::hash::{BuildHasher, Hash};
use std::rc::Rc;
use std::sync::Arc;
use std::vec::Vec;

impl<T: Encodable> Encodable for LinkedList<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_seq(self.len(), |s| {
            for (i, e) in self.iter().enumerate() {
                s.emit_seq_elt(i, |s| e.encode(s))?;
            }
            Ok(())
        })
    }
}

impl<T: Decodable> Decodable for LinkedList<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<LinkedList<T>, D::Error> {
        d.read_seq(|d, len| {
            let mut list = LinkedList::new();
            for _ in 0..len {
                list.push_back(d.read_seq_elt(|d| Decodable::decode(d))?);
            }
            Ok(list)
        })
    }
}

impl<T: Encodable> Encodable for VecDeque<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_seq(self.len(), |s| {
            for (i, e) in self.iter().enumerate() {
                s.emit_seq_elt(i, |s| e.encode(s))?;
            }
            Ok(())
        })
    }
}

impl<T: Decodable> Decodable for VecDeque<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<VecDeque<T>, D::Error> {
        d.read_seq(|d, len| {
            let mut deque: VecDeque<T> = VecDeque::with_capacity(len);
            for _ in 0..len {
                deque.push_back(d.read_seq_elt(|d| Decodable::decode(d))?);
            }
            Ok(deque)
        })
    }
}

impl<K, V> Encodable for BTreeMap<K, V>
where
    K: Encodable + PartialEq + Ord,
    V: Encodable,
{
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        e.emit_map(self.len(), |e| {
            for (i, (key, val)) in self.iter().enumerate() {
                e.emit_map_elt_key(i, |e| key.encode(e))?;
                e.emit_map_elt_val(|e| val.encode(e))?;
            }
            Ok(())
        })
    }
}

impl<K, V> Decodable for BTreeMap<K, V>
where
    K: Decodable + PartialEq + Ord,
    V: Decodable,
{
    fn decode<D: Decoder>(d: &mut D) -> Result<BTreeMap<K, V>, D::Error> {
        d.read_map(|d, len| {
            let mut map = BTreeMap::new();
            for i in 0..len {
                let key = d.read_map_elt_key(i, |d| Decodable::decode(d))?;
                let val = d.read_map_elt_val(i, |d| Decodable::decode(d))?;
                map.insert(key, val);
            }
            Ok(map)
        })
    }
}

impl<T> Encodable for BTreeSet<T>
where
    T: Encodable + PartialEq + Ord,
{
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_seq(self.len(), |s| {
            for (i, e) in self.iter().enumerate() {
                s.emit_seq_elt(i, |s| e.encode(s))?;
            }
            Ok(())
        })
    }
}

impl<T> Decodable for BTreeSet<T>
where
    T: Decodable + PartialEq + Ord,
{
    fn decode<D: Decoder>(d: &mut D) -> Result<BTreeSet<T>, D::Error> {
        d.read_seq(|d, len| {
            let mut set = BTreeSet::new();
            for _ in 0..len {
                set.insert(d.read_seq_elt(|d| Decodable::decode(d))?);
            }
            Ok(set)
        })
    }
}

impl<K, V, S> Encodable for HashMap<K, V, S>
where
    K: Encodable + Eq,
    V: Encodable,
    S: BuildHasher,
{
    fn encode<E: Encoder>(&self, e: &mut E) -> Result<(), E::Error> {
        e.emit_map(self.len(), |e| {
            for (i, (key, val)) in self.iter().enumerate() {
                e.emit_map_elt_key(i, |e| key.encode(e))?;
                e.emit_map_elt_val(|e| val.encode(e))?;
            }
            Ok(())
        })
    }
}

impl<K, V, S> Decodable for HashMap<K, V, S>
where
    K: Decodable + Hash + Eq,
    V: Decodable,
    S: BuildHasher + Default,
{
    fn decode<D: Decoder>(d: &mut D) -> Result<HashMap<K, V, S>, D::Error> {
        d.read_map(|d, len| {
            let state = Default::default();
            let mut map = HashMap::with_capacity_and_hasher(len, state);
            for i in 0..len {
                let key = d.read_map_elt_key(i, |d| Decodable::decode(d))?;
                let val = d.read_map_elt_val(i, |d| Decodable::decode(d))?;
                map.insert(key, val);
            }
            Ok(map)
        })
    }
}

impl<T, S> Encodable for HashSet<T, S>
where
    T: Encodable + Eq,
    S: BuildHasher,
{
    fn encode<E: Encoder>(&self, s: &mut E) -> Result<(), E::Error> {
        s.emit_seq(self.len(), |s| {
            for (i, e) in self.iter().enumerate() {
                s.emit_seq_elt(i, |s| e.encode(s))?;
            }
            Ok(())
        })
    }
}

impl<T, S> Encodable for &HashSet<T, S>
where
    T: Encodable + Eq,
    S: BuildHasher,
{
    fn encode<E: Encoder>(&self, s: &mut E) -> Result<(), E::Error> {
        (**self).encode(s)
    }
}

impl<T, S> Decodable for HashSet<T, S>
where
    T: Decodable + Hash + Eq,
    S: BuildHasher + Default,
{
    fn decode<D: Decoder>(d: &mut D) -> Result<HashSet<T, S>, D::Error> {
        d.read_seq(|d, len| {
            let state = Default::default();
            let mut set = HashSet::with_capacity_and_hasher(len, state);
            for _ in 0..len {
                set.insert(d.read_seq_elt(|d| Decodable::decode(d))?);
            }
            Ok(set)
        })
    }
}

impl<T: Encodable> Encodable for Rc<[T]> {
    fn encode<E: Encoder>(&self, s: &mut E) -> Result<(), E::Error> {
        let slice: &[T] = self;
        slice.encode(s)
    }
}

impl<T: Decodable> Decodable for Rc<[T]> {
    fn decode<D: Decoder>(d: &mut D) -> Result<Rc<[T]>, D::Error> {
        let vec: Vec<T> = Decodable::decode(d)?;
        Ok(vec.into())
    }
}

impl<T: Encodable> Encodable for Arc<[T]> {
    fn encode<E: Encoder>(&self, s: &mut E) -> Result<(), E::Error> {
        let slice: &[T] = self;
        slice.encode(s)
    }
}

impl<T: Decodable> Decodable for Arc<[T]> {
    fn decode<D: Decoder>(d: &mut D) -> Result<Arc<[T]>, D::Error> {
        let vec: Vec<T> = Decodable::decode(d)?;
        Ok(vec.into())
    }
}
