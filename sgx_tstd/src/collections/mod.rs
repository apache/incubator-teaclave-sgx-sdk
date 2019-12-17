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

//! Collection types.
//!
//! Rust's standard collection library provides efficient implementations of the
//! most common general purpose programming data structures. By using the
//! standard implementations, it should be possible for two libraries to
//! communicate without significant data conversion.
//!

pub use crate::ops::Bound;
pub use alloc_crate::collections::{BinaryHeap, BTreeMap, BTreeSet};
pub use alloc_crate::collections::{LinkedList, VecDeque};
pub use alloc_crate::collections::{binary_heap, btree_map, btree_set};
pub use alloc_crate::collections::{linked_list, vec_deque};

pub use self::hash_map::HashMap;
pub use self::hash_set::HashSet;

pub use alloc_crate::collections::TryReserveError;

mod hash;

pub mod hash_map {
    //! A hash map implemented with linear probing and Robin Hood bucket stealing.
    pub use super::hash::map::*;
}

pub mod hash_set {
    //! A hash set implemented as a `HashMap` where the value is `()`.
    pub use super::hash::set::*;
}
