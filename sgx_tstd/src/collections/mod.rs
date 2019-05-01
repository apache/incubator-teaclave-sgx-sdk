// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

//! Collection types.
//!
//! Rust's standard collection library provides efficient implementations of the
//! most common general purpose programming data structures. By using the
//! standard implementations, it should be possible for two libraries to
//! communicate without significant data conversion.
//!

pub use ops::Bound;
pub use alloc_crate::collections::{BinaryHeap, BTreeMap, BTreeSet};
pub use alloc_crate::collections::{LinkedList, VecDeque};
pub use alloc_crate::collections::{binary_heap, btree_map, btree_set};
pub use alloc_crate::collections::{linked_list, vec_deque};

pub use self::hash_map::HashMap;
pub use self::hash_set::HashSet;

pub use alloc_crate::collections::CollectionAllocErr;

mod hash;

pub mod hash_map {
    //! A hash map implemented with linear probing and Robin Hood bucket stealing.
    pub use super::hash::map::*;
}

pub mod hash_set {
    //! A hash set implemented as a `HashMap` where the value is `()`.
    pub use super::hash::set::*;
}
