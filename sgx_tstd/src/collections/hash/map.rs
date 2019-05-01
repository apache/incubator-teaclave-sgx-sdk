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

use self::Entry::*;
use self::VacantEntryState::*;

use intrinsics::unlikely;
use collections::CollectionAllocErr;
use core::cell::Cell;
use core::borrow::Borrow;
use core::cmp::max;
use core::fmt::{self, Debug};
#[allow(deprecated)]
use core::hash::{Hash, Hasher, BuildHasher, SipHasher13};
use core::iter::{FromIterator, FusedIterator};
use core::mem::{self, replace};
use core::ops::{Deref, DerefMut, Index};
use sys;

use super::table::{self, Bucket, EmptyBucket, Fallibility, FullBucket, FullBucketMut, RawTable,
                   SafeHash};
use super::table::BucketState::{Empty, Full};
use super::table::Fallibility::{Fallible, Infallible};

const MIN_NONZERO_RAW_CAPACITY: usize = 32;     // must be a power of two

/// The default behavior of HashMap implements a maximum load factor of 90.9%.
#[derive(Clone)]
struct DefaultResizePolicy;

impl DefaultResizePolicy {
    #[inline]
    fn new() -> DefaultResizePolicy {
        DefaultResizePolicy
    }

    /// A hash map's "capacity" is the number of elements it can hold without
    /// being resized. Its "raw capacity" is the number of slots required to
    /// provide that capacity, accounting for maximum loading. The raw capacity
    /// is always zero or a power of two.
    #[inline]
    fn try_raw_capacity(&self, len: usize) -> Result<usize, CollectionAllocErr> {
        if len == 0 {
            Ok(0)
        } else {
            // 1. Account for loading: `raw_capacity >= len * 1.1`.
            // 2. Ensure it is a power of two.
            // 3. Ensure it is at least the minimum size.
            let mut raw_cap = len.checked_mul(11)
                .map(|l| l / 10)
                .and_then(|l| l.checked_next_power_of_two())
                .ok_or(CollectionAllocErr::CapacityOverflow)?;

            raw_cap = max(MIN_NONZERO_RAW_CAPACITY, raw_cap);
            Ok(raw_cap)
        }
    }

    #[inline]
    fn raw_capacity(&self, len: usize) -> usize {
        self.try_raw_capacity(len).expect("raw_capacity overflow")
    }

    /// The capacity of the given raw capacity.
    #[inline]
    fn capacity(&self, raw_cap: usize) -> usize {
        // This doesn't have to be checked for overflow since allocation size
        // in bytes will overflow earlier than multiplication by 10.
        //
        // As per https://github.com/rust-lang/rust/pull/30991 this is updated
        // to be: (raw_cap * den + den - 1) / num
        (raw_cap * 10 + 10 - 1) / 11
    }
}

// The main performance trick in this hashmap is called Robin Hood Hashing.
// It gains its excellent performance from one essential operation:
//
//    If an insertion collides with an existing element, and that element's
//    "probe distance" (how far away the element is from its ideal location)
//    is higher than how far we've already probed, swap the elements.
//
// This massively lowers variance in probe distance, and allows us to get very
// high load factors with good performance. The 90% load factor I use is rather
// conservative.
//
// > Why a load factor of approximately 90%?
//
// In general, all the distances to initial buckets will converge on the mean.
// At a load factor of α, the odds of finding the target bucket after k
// probes is approximately 1-α^k. If we set this equal to 50% (since we converge
// on the mean) and set k=8 (64-byte cache line / 8-byte hash), α=0.92. I round
// this down to make the math easier on the CPU and avoid its FPU.
// Since on average we start the probing in the middle of a cache line, this
// strategy pulls in two cache lines of hashes on every lookup. I think that's
// pretty good, but if you want to trade off some space, it could go down to one
// cache line on average with an α of 0.84.
//
// > Wait, what? Where did you get 1-α^k from?
//
// On the first probe, your odds of a collision with an existing element is α.
// The odds of doing this twice in a row is approximately α^2. For three times,
// α^3, etc. Therefore, the odds of colliding k times is α^k. The odds of NOT
// colliding after k tries is 1-α^k.
//
// The paper from 1986 cited below mentions an implementation which keeps track
// of the distance-to-initial-bucket histogram. This approach is not suitable
// for modern architectures because it requires maintaining an internal data
// structure. This allows very good first guesses, but we are most concerned
// with guessing entire cache lines, not individual indexes. Furthermore, array
// accesses are no longer linear and in one direction, as we have now. There
// is also memory and cache pressure that this would entail that would be very
// difficult to properly see in a microbenchmark.
//
// ## Future Improvements (FIXME!)
//
// Allow the load factor to be changed dynamically and/or at initialization.
//
// Also, would it be possible for us to reuse storage when growing the
// underlying table? This is exactly the use case for 'realloc', and may
// be worth exploring.
//
// ## Future Optimizations (FIXME!)
//
// Another possible design choice that I made without any real reason is
// parameterizing the raw table over keys and values. Technically, all we need
// is the size and alignment of keys and values, and the code should be just as
// efficient (well, we might need one for power-of-two size and one for not...).
// This has the potential to reduce code bloat in rust executables, without
// really losing anything except 4 words (key size, key alignment, val size,
// val alignment) which can be passed in to every call of a `RawTable` function.
// This would definitely be an avenue worth exploring if people start complaining
// about the size of rust executables.
//
// Annotate exceedingly likely branches in `table::make_hash`
// and `search_hashed` to reduce instruction cache pressure
// and mispredictions once it becomes possible (blocked on issue #11092).
//
// Shrinking the table could simply reallocate in place after moving buckets
// to the first half.
//
// The growth algorithm (fragment of the Proof of Correctness)
// --------------------
//
// The growth algorithm is basically a fast path of the naive reinsertion-
// during-resize algorithm. Other paths should never be taken.
//
// Consider growing a robin hood hashtable of capacity n. Normally, we do this
// by allocating a new table of capacity `2n`, and then individually reinsert
// each element in the old table into the new one. This guarantees that the
// new table is a valid robin hood hashtable with all the desired statistical
// properties. Remark that the order we reinsert the elements in should not
// matter. For simplicity and efficiency, we will consider only linear
// reinsertions, which consist of reinserting all elements in the old table
// into the new one by increasing order of index. However we will not be
// starting our reinsertions from index 0 in general. If we start from index
// i, for the purpose of reinsertion we will consider all elements with real
// index j < i to have virtual index n + j.
//
// Our hash generation scheme consists of generating a 64-bit hash and
// truncating the most significant bits. When moving to the new table, we
// simply introduce a new bit to the front of the hash. Therefore, if an
// element has ideal index i in the old table, it can have one of two ideal
// locations in the new table. If the new bit is 0, then the new ideal index
// is i. If the new bit is 1, then the new ideal index is n + i. Intuitively,
// we are producing two independent tables of size n, and for each element we
// independently choose which table to insert it into with equal probability.
// However, rather than wrapping around themselves on overflowing their
// indexes, the first table overflows into the second, and the second into the
// first. Visually, our new table will look something like:
//
// [yy_xxx_xxxx_xxx|xx_yyy_yyyy_yyy]
//
// Where x's are elements inserted into the first table, y's are elements
// inserted into the second, and _'s are empty sections. We now define a few
// key concepts that we will use later. Note that this is a very abstract
// perspective of the table. A real resized table would be at least half
// empty.
//
// Theorem: A linear robin hood reinsertion from the first ideal element
// produces identical results to a linear naive reinsertion from the same
// element.
//
// FIXME(Gankro, pczarn): review the proof and put it all in a separate README.md
//
// Adaptive early resizing
// ----------------------
// To protect against degenerate performance scenarios (including DOS attacks),
// the implementation includes an adaptive behavior that can resize the map
// early (before its capacity is exceeded) when suspiciously long probe sequences
// are encountered.
//
// With this algorithm in place it would be possible to turn a CPU attack into
// a memory attack due to the aggressive resizing. To prevent that the
// adaptive behavior only triggers when the map is at least half full.
// This reduces the effectiveness of the algorithm but also makes it completely safe.
//
// The previous safety measure also prevents degenerate interactions with
// really bad quality hash algorithms that can make normal inputs look like a
// DOS attack.
//
const DISPLACEMENT_THRESHOLD: usize = 128;
//
// The threshold of 128 is chosen to minimize the chance of exceeding it.
// In particular, we want that chance to be less than 10^-8 with a load of 90%.
// For displacement, the smallest constant that fits our needs is 90,
// so we round that up to 128.
//
// At a load factor of α, the odds of finding the target bucket after exactly n
// unsuccessful probes[1] are
//
// Pr_α{displacement = n} =
// (1 - α) / α * ∑_{k≥1} e^(-kα) * (kα)^(k+n) / (k + n)! * (1 - kα / (k + n + 1))
//
// We use this formula to find the probability of triggering the adaptive behavior
//
// Pr_0.909{displacement > 128} = 1.601 * 10^-11
//
// 1. Alfredo Viola (2005). Distributional analysis of Robin Hood linear probing
//    hashing with buckets.

/// A hash map implemented with linear probing and Robin Hood bucket stealing.
///
/// By default, `HashMap` uses a hashing algorithm selected to provide
/// resistance against HashDoS attacks. The algorithm is randomly seeded, and a
/// reasonable best-effort is made to generate this seed from a high quality,
/// secure source of randomness provided by the host without blocking the
/// program. Because of this, the randomness of the seed depends on the output
/// quality of the system's random number generator when the seed is created.
/// In particular, seeds generated when the system's entropy pool is abnormally
/// low such as during system boot may be of a lower quality.
///
/// The default hashing algorithm is currently SipHash 1-3, though this is
/// subject to change at any point in the future. While its performance is very
/// competitive for medium sized keys, other hashing algorithms will outperform
/// it for small keys such as integers as well as large keys such as long
/// strings, though those algorithms will typically *not* protect against
/// attacks such as HashDoS.
///
/// The hashing algorithm can be replaced on a per-`HashMap` basis using the
/// [`default`], [`with_hasher`], and [`with_capacity_and_hasher`] methods. Many
/// alternative algorithms are available on crates.io, such as the [`fnv`] crate.
///
/// It is required that the keys implement the [`Eq`] and [`Hash`] traits, although
/// this can frequently be achieved by using `#[derive(PartialEq, Eq, Hash)]`.
/// If you implement these yourself, it is important that the following
/// property holds:
///
/// ```text
/// k1 == k2 -> hash(k1) == hash(k2)
/// ```
///
/// In other words, if two keys are equal, their hashes must be equal.
///
/// It is a logic error for a key to be modified in such a way that the key's
/// hash, as determined by the [`Hash`] trait, or its equality, as determined by
/// the [`Eq`] trait, changes while it is in the map. This is normally only
/// possible through [`Cell`], [`RefCell`], global state, I/O, or unsafe code.
///
/// Relevant papers/articles:
///
/// 1. Pedro Celis. ["Robin Hood Hashing"](https://cs.uwaterloo.ca/research/tr/1986/CS-86-14.pdf)
/// 2. Emmanuel Goossaert. ["Robin Hood
///    hashing"](http://codecapsule.com/2013/11/11/robin-hood-hashing/)
/// 3. Emmanuel Goossaert. ["Robin Hood hashing: backward shift
///    deletion"](http://codecapsule.com/2013/11/17/robin-hood-hashing-backward-shift-deletion/)
///
#[derive(Clone)]
pub struct HashMap<K, V, S = RandomState> {
    // All hashes are keyed on these values, to prevent hash collision attacks.
    hash_builder: S,

    table: RawTable<K, V>,

    resize_policy: DefaultResizePolicy,
}

/// Search for a pre-hashed key.
/// If you don't already know the hash, use search or search_mut instead
#[inline]
fn search_hashed<K, V, M, F>(table: M, hash: SafeHash, is_match: F) -> InternalEntry<K, V, M>
    where M: Deref<Target = RawTable<K, V>>,
          F: FnMut(&K) -> bool
{
    // This is the only function where capacity can be zero. To avoid
    // undefined behavior when Bucket::new gets the raw bucket in this
    // case, immediately return the appropriate search result.
    if table.capacity() == 0 {
        return InternalEntry::TableIsEmpty;
    }

    search_hashed_nonempty(table, hash, is_match, true)
}

/// Search for a pre-hashed key when the hash map is known to be non-empty.
#[inline]
fn search_hashed_nonempty<K, V, M, F>(table: M, hash: SafeHash, mut is_match: F,
                                      compare_hashes: bool)
    -> InternalEntry<K, V, M>
    where M: Deref<Target = RawTable<K, V>>,
          F: FnMut(&K) -> bool
{
    // Do not check the capacity as an extra branch could slow the lookup.

    let size = table.size();
    let mut probe = Bucket::new(table, hash);
    let mut displacement = 0;

    loop {
        let full = match probe.peek() {
            Empty(bucket) => {
                // Found a hole!
                return InternalEntry::Vacant {
                    hash,
                    elem: NoElem(bucket, displacement),
                };
            }
            Full(bucket) => bucket,
        };

        let probe_displacement = full.displacement();

        if probe_displacement < displacement {
            // Found a luckier bucket than me.
            // We can finish the search early if we hit any bucket
            // with a lower distance to initial bucket than we've probed.
            return InternalEntry::Vacant {
                hash,
                elem: NeqElem(full, probe_displacement),
            };
        }

        // If the hash doesn't match, it can't be this one..
        if !compare_hashes || hash == full.hash() {
            // If the key doesn't match, it can't be this one..
            if is_match(full.read().0) {
                return InternalEntry::Occupied { elem: full };
            }
        }
        displacement += 1;
        probe = full.next();
        debug_assert!(displacement <= size);
    }
}

/// Same as `search_hashed_nonempty` but for mutable access.
#[inline]
fn search_hashed_nonempty_mut<K, V, M, F>(table: M, hash: SafeHash, mut is_match: F,
                                          compare_hashes: bool)
    -> InternalEntry<K, V, M>
    where M: DerefMut<Target = RawTable<K, V>>,
          F: FnMut(&K) -> bool
{
    // Do not check the capacity as an extra branch could slow the lookup.

    let size = table.size();
    let mut probe = Bucket::new(table, hash);
    let mut displacement = 0;

    loop {
        let mut full = match probe.peek() {
            Empty(bucket) => {
                // Found a hole!
                return InternalEntry::Vacant {
                    hash,
                    elem: NoElem(bucket, displacement),
                };
            }
            Full(bucket) => bucket,
        };

        let probe_displacement = full.displacement();

        if probe_displacement < displacement {
            // Found a luckier bucket than me.
            // We can finish the search early if we hit any bucket
            // with a lower distance to initial bucket than we've probed.
            return InternalEntry::Vacant {
                hash,
                elem: NeqElem(full, probe_displacement),
            };
        }

        // If the hash doesn't match, it can't be this one..
        if hash == full.hash() || !compare_hashes {
            // If the key doesn't match, it can't be this one..
            if is_match(full.read_mut().0) {
                return InternalEntry::Occupied { elem: full };
            }
        }
        displacement += 1;
        probe = full.next();
        debug_assert!(displacement <= size);
    }
}

fn pop_internal<K, V>(starting_bucket: FullBucketMut<K, V>)
    -> (K, V, &mut RawTable<K, V>)
{
    let (empty, retkey, retval) = starting_bucket.take();
    let mut gap = match empty.gap_peek() {
        Ok(b) => b,
        Err(b) => return (retkey, retval, b.into_table()),
    };

    while gap.full().displacement() != 0 {
        gap = match gap.shift() {
            Ok(b) => b,
            Err(b) => {
                return (retkey, retval, b.into_table());
            },
        };
    }

    // Now we've done all our shifting. Return the value we grabbed earlier.
    (retkey, retval, gap.into_table())
}

/// Performs robin hood bucket stealing at the given `bucket`. You must
/// also pass that bucket's displacement so we don't have to recalculate it.
///
/// `hash`, `key`, and `val` are the elements to "robin hood" into the hashtable.
fn robin_hood<'a, K: 'a, V: 'a>(bucket: FullBucketMut<'a, K, V>,
                                mut displacement: usize,
                                mut hash: SafeHash,
                                mut key: K,
                                mut val: V)
                                -> FullBucketMut<'a, K, V> {
    let size = bucket.table().size();
    let raw_capacity = bucket.table().capacity();
    // There can be at most `size - dib` buckets to displace, because
    // in the worst case, there are `size` elements and we already are
    // `displacement` buckets away from the initial one.
    let idx_end = (bucket.index() + size - bucket.displacement()) % raw_capacity;
    // Save the *starting point*.
    let mut bucket = bucket.stash();

    loop {
        let (old_hash, old_key, old_val) = bucket.replace(hash, key, val);
        hash = old_hash;
        key = old_key;
        val = old_val;

        loop {
            displacement += 1;
            let probe = bucket.next();
            debug_assert!(probe.index() != idx_end);

            let full_bucket = match probe.peek() {
                Empty(bucket) => {
                    // Found a hole!
                    let bucket = bucket.put(hash, key, val);
                    // Now that it's stolen, just read the value's pointer
                    // right out of the table! Go back to the *starting point*.
                    //
                    // This use of `into_table` is misleading. It turns the
                    // bucket, which is a FullBucket on top of a
                    // FullBucketMut, into just one FullBucketMut. The "table"
                    // refers to the inner FullBucketMut in this context.
                    return bucket.into_table();
                }
                Full(bucket) => bucket,
            };

            let probe_displacement = full_bucket.displacement();

            bucket = full_bucket;

            // Robin hood! Steal the spot.
            if probe_displacement < displacement {
                displacement = probe_displacement;
                break;
            }
        }
    }
}

impl<K, V, S> HashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher
{
    fn make_hash<X: ?Sized>(&self, x: &X) -> SafeHash
        where X: Hash
    {
        table::make_hash(&self.hash_builder, x)
    }

    /// Search for a key, yielding the index if it's found in the hashtable.
    /// If you already have the hash for the key lying around, or if you need an
    /// InternalEntry, use search_hashed or search_hashed_nonempty.
    #[inline]
    fn search<'a, Q: ?Sized>(&'a self, q: &Q)
        -> Option<FullBucket<K, V, &'a RawTable<K, V>>>
        where K: Borrow<Q>,
              Q: Eq + Hash
    {
        if self.is_empty() {
            return None;
        }

        let hash = self.make_hash(q);
        search_hashed_nonempty(&self.table, hash, |k| q.eq(k.borrow()), true)
            .into_occupied_bucket()
    }

    #[inline]
    fn search_mut<'a, Q: ?Sized>(&'a mut self, q: &Q)
        -> Option<FullBucket<K, V, &'a mut RawTable<K, V>>>
        where K: Borrow<Q>,
              Q: Eq + Hash
    {
        if self.is_empty() {
            return None;
        }

        let hash = self.make_hash(q);
        search_hashed_nonempty(&mut self.table, hash, |k| q.eq(k.borrow()), true)
            .into_occupied_bucket()
    }

    // The caller should ensure that invariants by Robin Hood Hashing hold
    // and that there's space in the underlying table.
    fn insert_hashed_ordered(&mut self, hash: SafeHash, k: K, v: V) {
        let mut buckets = Bucket::new(&mut self.table, hash);
        let start_index = buckets.index();

        loop {
            // We don't need to compare hashes for value swap.
            // Not even DIBs for Robin Hood.
            buckets = match buckets.peek() {
                Empty(empty) => {
                    empty.put(hash, k, v);
                    return;
                }
                Full(b) => b.into_bucket(),
            };
            buckets.next();
            debug_assert!(buckets.index() != start_index);
        }
    }
}

impl<K: Hash + Eq, V> HashMap<K, V, RandomState> {
    /// Creates an empty `HashMap`.
    ///
    /// The hash map is initially created with a capacity of 0, so it will not allocate until it
    /// is first inserted into.
    ///
    #[inline]
    pub fn new() -> HashMap<K, V, RandomState> {
        Default::default()
    }

    /// Creates an empty `HashMap` with the specified capacity.
    ///
    /// The hash map will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash map will not allocate.
    #[inline]
    pub fn with_capacity(capacity: usize) -> HashMap<K, V, RandomState> {
        HashMap::with_capacity_and_hasher(capacity, Default::default())
    }
}

impl<K, V, S> HashMap<K, V, S> {
    /// Returns the number of elements the map can hold without reallocating.
    ///
    /// This number is a lower bound; the `HashMap<K, V>` might be able to hold
    /// more, but is guaranteed to be able to hold at least this many.
    ///
    #[inline]
    pub fn capacity(&self) -> usize {
        self.resize_policy.capacity(self.raw_capacity())
    }

    /// Returns the hash map's raw capacity.
    #[inline]
    fn raw_capacity(&self) -> usize {
        self.table.capacity()
    }

    /// An iterator visiting all keys in arbitrary order.
    /// The iterator element type is `&'a K`.
    ///
    pub fn keys(&self) -> Keys<K, V> {
        Keys { inner: self.iter() }
    }

    /// An iterator visiting all values in arbitrary order.
    /// The iterator element type is `&'a V`.
    ///
    pub fn values(&self) -> Values<K, V> {
        Values { inner: self.iter() }
    }

    /// An iterator visiting all values mutably in arbitrary order.
    /// The iterator element type is `&'a mut V`.
    ///
    pub fn values_mut(&mut self) -> ValuesMut<K, V> {
        ValuesMut { inner: self.iter_mut() }
    }

    /// An iterator visiting all key-value pairs in arbitrary order.
    /// The iterator element type is `(&'a K, &'a V)`.
    ///
    pub fn iter(&self) -> Iter<K, V> {
        Iter { inner: self.table.iter() }
    }

    /// An iterator visiting all key-value pairs in arbitrary order,
    /// with mutable references to the values.
    /// The iterator element type is `(&'a K, &'a mut V)`.
    ///
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut { inner: self.table.iter_mut() }
    }

    /// Returns the number of elements in the map.
    ///
    pub fn len(&self) -> usize {
        self.table.size()
    }

    /// Returns `true` if the map contains no elements.
    ///
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears the map, returning all key-value pairs as an iterator. Keeps the
    /// allocated memory for reuse.
    ///
    #[inline]
    pub fn drain(&mut self) -> Drain<K, V> {
        Drain { inner: self.table.drain() }
    }

    /// Clears the map, removing all key-value pairs. Keeps the allocated memory
    /// for reuse.
    ///
    #[inline]
    pub fn clear(&mut self) {
        self.drain();
    }
}

impl<K, V, S> HashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher
{
    /// Creates an empty `HashMap` which will use the given hash builder to hash
    /// keys.
    ///
    /// The created map has the default initial capacity.
    ///
    /// Warning: `hash_builder` is normally randomly generated, and
    /// is designed to allow HashMaps to be resistant to attacks that
    /// cause many collisions and very poor performance. Setting it
    /// manually using this function can expose a DoS attack vector.
    ///
    #[inline]
    pub fn with_hasher(hash_builder: S) -> HashMap<K, V, S> {
        HashMap {
            hash_builder,
            resize_policy: DefaultResizePolicy::new(),
            table: RawTable::new(0),
        }
    }

    /// Creates an empty `HashMap` with the specified capacity, using `hash_builder`
    /// to hash the keys.
    ///
    /// The hash map will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash map will not allocate.
    ///
    /// Warning: `hash_builder` is normally randomly generated, and
    /// is designed to allow HashMaps to be resistant to attacks that
    /// cause many collisions and very poor performance. Setting it
    /// manually using this function can expose a DoS attack vector.
    ///
    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> HashMap<K, V, S> {
        let resize_policy = DefaultResizePolicy::new();
        let raw_cap = resize_policy.raw_capacity(capacity);
        HashMap {
            hash_builder,
            resize_policy,
            table: RawTable::new(raw_cap),
        }
    }

    /// Returns a reference to the map's [`BuildHasher`].
    ///
    pub fn hasher(&self) -> &S {
        &self.hash_builder
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the `HashMap`. The collection may reserve more space to avoid
    /// frequent reallocations.
    ///
    /// # Panics
    ///
    /// Panics if the new allocation size overflows [`usize`].
    ///
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        match self.reserve_internal(additional, Infallible) {
            Err(CollectionAllocErr::CapacityOverflow) => panic!("capacity overflow"),
            Err(CollectionAllocErr::AllocErr) => unreachable!(),
            Ok(()) => { /* yay */ }
        }
    }

    /// Tries to reserve capacity for at least `additional` more elements to be inserted
    /// in the given `HashMap<K,V>`. The collection may reserve more space to avoid
    /// frequent reallocations.
    ///
    /// # Errors
    ///
    /// If the capacity overflows, or the allocator reports a failure, then an error
    /// is returned.
    ///
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), CollectionAllocErr> {
        self.reserve_internal(additional, Fallible)
    }

    #[inline]
    fn reserve_internal(&mut self, additional: usize, fallibility: Fallibility)
        -> Result<(), CollectionAllocErr> {

        let remaining = self.capacity() - self.len(); // this can't overflow
        if remaining < additional {
            let min_cap = self.len()
                .checked_add(additional)
                .ok_or(CollectionAllocErr::CapacityOverflow)?;
            let raw_cap = self.resize_policy.try_raw_capacity(min_cap)?;
            self.try_resize(raw_cap, fallibility)?;
        } else if self.table.tag() && remaining <= self.len() {
            // Probe sequence is too long and table is half full,
            // resize early to reduce probing length.
            let new_capacity = self.table.capacity() * 2;
            self.try_resize(new_capacity, fallibility)?;
        }
        Ok(())
    }

    /// Resizes the internal vectors to a new capacity. It's your
    /// responsibility to:
    ///   1) Ensure `new_raw_cap` is enough for all the elements, accounting
    ///      for the load factor.
    ///   2) Ensure `new_raw_cap` is a power of two or zero.
    #[inline(never)]
    #[cold]
    fn try_resize(
        &mut self,
        new_raw_cap: usize,
        fallibility: Fallibility,
    ) -> Result<(), CollectionAllocErr> {
        assert!(self.table.size() <= new_raw_cap);
        assert!(new_raw_cap.is_power_of_two() || new_raw_cap == 0);

        let mut old_table = replace(
            &mut self.table,
            match fallibility {
                Infallible => RawTable::new(new_raw_cap),
                Fallible => RawTable::try_new(new_raw_cap)?,
            }
        );
        let old_size = old_table.size();

        if old_table.size() == 0 {
            return Ok(());
        }

        let mut bucket = Bucket::head_bucket(&mut old_table);

        // This is how the buckets might be laid out in memory:
        // ($ marks an initialized bucket)
        //  ________________
        // |$$$_$$$$$$_$$$$$|
        //
        // But we've skipped the entire initial cluster of buckets
        // and will continue iteration in this order:
        //  ________________
        //     |$$$$$$_$$$$$
        //                  ^ wrap around once end is reached
        //  ________________
        //  $$$_____________|
        //    ^ exit once table.size == 0
        loop {
            bucket = match bucket.peek() {
                Full(bucket) => {
                    let h = bucket.hash();
                    let (b, k, v) = bucket.take();
                    self.insert_hashed_ordered(h, k, v);
                    if b.table().size() == 0 {
                        break;
                    }
                    b.into_bucket()
                }
                Empty(b) => b.into_bucket(),
            };
            bucket.next();
        }

        assert_eq!(self.table.size(), old_size);
        Ok(())
    }

    /// Shrinks the capacity of the map as much as possible. It will drop
    /// down as much as possible while maintaining the internal rules
    /// and possibly leaving some space in accordance with the resize policy.
    ///
    pub fn shrink_to_fit(&mut self) {
        let new_raw_cap = self.resize_policy.raw_capacity(self.len());
        if self.raw_capacity() != new_raw_cap {
            let old_table = replace(&mut self.table, RawTable::new(new_raw_cap));
            let old_size = old_table.size();

            // Shrink the table. Naive algorithm for resizing:
            for (h, k, v) in old_table.into_iter() {
                self.insert_hashed_nocheck(h, k, v);
            }

            debug_assert_eq!(self.table.size(), old_size);
        }
    }

    /// Shrinks the capacity of the map with a lower limit. It will drop
    /// down no lower than the supplied limit while maintaining the internal rules
    /// and possibly leaving some space in accordance with the resize policy.
    ///
    /// Panics if the current capacity is smaller than the supplied
    /// minimum capacity.
    ///
    pub fn shrink_to(&mut self, min_capacity: usize) {
        assert!(self.capacity() >= min_capacity, "Tried to shrink to a larger capacity");

        let new_raw_cap = self.resize_policy.raw_capacity(max(self.len(), min_capacity));
        if self.raw_capacity() != new_raw_cap {
            let old_table = replace(&mut self.table, RawTable::new(new_raw_cap));
            let old_size = old_table.size();

            // Shrink the table. Naive algorithm for resizing:
            for (h, k, v) in old_table.into_iter() {
                self.insert_hashed_nocheck(h, k, v);
            }

            debug_assert_eq!(self.table.size(), old_size);
        }
    }

    /// Insert a pre-hashed key-value pair, without first checking
    /// that there's enough room in the buckets. Returns a reference to the
    /// newly insert value.
    ///
    /// If the key already exists, the hashtable will be returned untouched
    /// and a reference to the existing element will be returned.
    fn insert_hashed_nocheck(&mut self, hash: SafeHash, k: K, v: V) -> Option<V> {
        let entry = search_hashed(&mut self.table, hash, |key| *key == k).into_entry(k);
        match entry {
            Some(Occupied(mut elem)) => Some(elem.insert(v)),
            Some(Vacant(elem)) => {
                elem.insert(v);
                None
            }
            None => unreachable!(),
        }
    }

    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    ///
    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        // Gotta resize now.
        self.reserve(1);
        let hash = self.make_hash(&key);
        search_hashed(&mut self.table, hash, |q| q.eq(&key))
            .into_entry(key).expect("unreachable")
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// [`Eq`]: ../../std/cmp/trait.Eq.html
    /// [`Hash`]: ../../std/hash/trait.Hash.html
    ///
    #[inline]
    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
        where K: Borrow<Q>,
              Q: Hash + Eq
    {
        self.search(k).map(|bucket| bucket.into_refs().1)
    }

    /// Returns the key-value pair corresponding to the supplied key.
    ///
    /// The supplied key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// [`Eq`]: ../../std/cmp/trait.Eq.html
    /// [`Hash`]: ../../std/hash/trait.Hash.html
    ///
    pub fn get_key_value<Q: ?Sized>(&self, k: &Q) -> Option<(&K, &V)>
        where K: Borrow<Q>,
              Q: Hash + Eq
    {
        self.search(k).map(|bucket| bucket.into_refs())
    }

    /// Returns true if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// [`Eq`]: ../../std/cmp/trait.Eq.html
    /// [`Hash`]: ../../std/hash/trait.Hash.html
    ///
    pub fn contains_key<Q: ?Sized>(&self, k: &Q) -> bool
        where K: Borrow<Q>,
              Q: Hash + Eq
    {
        self.search(k).is_some()
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// [`Eq`]: ../../std/cmp/trait.Eq.html
    /// [`Hash`]: ../../std/hash/trait.Hash.html
    ///
    pub fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut V>
        where K: Borrow<Q>,
              Q: Hash + Eq
    {
        self.search_mut(k).map(|bucket| bucket.into_mut_refs().1)
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though; this matters for
    /// types that can be `==` without being identical. See the [module-level
    /// documentation] for more.
    ///
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    /// [module-level documentation]: index.html#insert-and-complex-keys
    ///
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        let hash = self.make_hash(&k);
        self.reserve(1);
        self.insert_hashed_nocheck(hash, k, v)
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// [`Eq`]: ../../std/cmp/trait.Eq.html
    /// [`Hash`]: ../../std/hash/trait.Hash.html
    ///
    pub fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<V>
        where K: Borrow<Q>,
              Q: Hash + Eq
    {
        self.search_mut(k).map(|bucket| pop_internal(bucket).1)
    }

    /// Removes a key from the map, returning the stored key and value if the
    /// key was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// [`Eq`]: ../../std/cmp/trait.Eq.html
    /// [`Hash`]: ../../std/hash/trait.Hash.html
    ///
    pub fn remove_entry<Q: ?Sized>(&mut self, k: &Q) -> Option<(K, V)>
        where K: Borrow<Q>,
              Q: Hash + Eq
    {
        self.search_mut(k)
            .map(|bucket| {
                let (k, v, _) = pop_internal(bucket);
                (k, v)
            })
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs `(k, v)` such that `f(&k,&mut v)` returns `false`.
    ///
    pub fn retain<F>(&mut self, mut f: F)
        where F: FnMut(&K, &mut V) -> bool
    {
        if self.table.size() == 0 {
            return;
        }
        let mut elems_left = self.table.size();
        let mut bucket = Bucket::head_bucket(&mut self.table);
        bucket.prev();
        let start_index = bucket.index();
        while elems_left != 0 {
            bucket = match bucket.peek() {
                Full(mut full) => {
                    elems_left -= 1;
                    let should_remove = {
                        let (k, v) = full.read_mut();
                        !f(k, v)
                    };
                    if should_remove {
                        let prev_raw = full.raw();
                        let (_, _, t) = pop_internal(full);
                        Bucket::new_from(prev_raw, t)
                    } else {
                        full.into_bucket()
                    }
                },
                Empty(b) => {
                    b.into_bucket()
                }
            };
            bucket.prev();  // reverse iteration
            debug_assert!(elems_left == 0 || bucket.index() != start_index);
        }
    }
}

impl<K, V, S> HashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher
{
    /// Creates a raw entry builder for the HashMap.
    ///
    /// Raw entries provide the lowest level of control for searching and
    /// manipulating a map. They must be manually initialized with a hash and
    /// then manually searched. After this, insertions into a vacant entry
    /// still require an owned key to be provided.
    ///
    /// Raw entries are useful for such exotic situations as:
    ///
    /// * Hash memoization
    /// * Deferring the creation of an owned key until it is known to be required
    /// * Using a search key that doesn't work with the Borrow trait
    /// * Using custom comparison logic without newtype wrappers
    ///
    /// Because raw entries provide much more low-level control, it's much easier
    /// to put the HashMap into an inconsistent state which, while memory-safe,
    /// will cause the map to produce seemingly random results. Higher-level and
    /// more foolproof APIs like `entry` should be preferred when possible.
    ///
    /// In particular, the hash used to initialized the raw entry must still be
    /// consistent with the hash of the key that is ultimately stored in the entry.
    /// This is because implementations of HashMap may need to recompute hashes
    /// when resizing, at which point only the keys are available.
    ///
    /// Raw entries give mutable access to the keys. This must not be used
    /// to modify how the key would compare or hash, as the map will not re-evaluate
    /// where the key should go, meaning the keys may become "lost" if their
    /// location does not reflect their state. For instance, if you change a key
    /// so that the map now contains keys which compare equal, search may start
    /// acting erratically, with two keys randomly masking each other. Implementations
    /// are free to assume this doesn't happen (within the limits of memory-safety).
    #[inline(always)]
    pub fn raw_entry_mut(&mut self) -> RawEntryBuilderMut<K, V, S> {
        self.reserve(1);
        RawEntryBuilderMut { map: self }
    }

    /// Creates a raw immutable entry builder for the HashMap.
    ///
    /// Raw entries provide the lowest level of control for searching and
    /// manipulating a map. They must be manually initialized with a hash and
    /// then manually searched.
    ///
    /// This is useful for
    /// * Hash memoization
    /// * Using a search key that doesn't work with the Borrow trait
    /// * Using custom comparison logic without newtype wrappers
    ///
    /// Unless you are in such a situation, higher-level and more foolproof APIs like
    /// `get` should be preferred.
    ///
    /// Immutable raw entries have very limited use; you might instead want `raw_entry_mut`.
    pub fn raw_entry(&self) -> RawEntryBuilder<K, V, S> {
        RawEntryBuilder { map: self }
    }
}

impl<K, V, S> PartialEq for HashMap<K, V, S>
    where K: Eq + Hash,
          V: PartialEq,
          S: BuildHasher
{
    fn eq(&self, other: &HashMap<K, V, S>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().all(|(key, value)| other.get(key).map_or(false, |v| *value == *v))
    }
}

impl<K, V, S> Eq for HashMap<K, V, S>
    where K: Eq + Hash,
          V: Eq,
          S: BuildHasher
{
}

impl<K, V, S> Debug for HashMap<K, V, S>
    where K: Eq + Hash + Debug,
          V: Debug,
          S: BuildHasher
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K, V, S> Default for HashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher + Default
{
    /// Creates an empty `HashMap<K, V, S>`, with the `Default` value for the hasher.
    fn default() -> HashMap<K, V, S> {
        HashMap::with_hasher(Default::default())
    }
}

impl<K, Q: ?Sized, V, S> Index<&Q> for HashMap<K, V, S>
    where K: Eq + Hash + Borrow<Q>,
          Q: Eq + Hash,
          S: BuildHasher
{
    type Output = V;

    /// Returns a reference to the value corresponding to the supplied key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not present in the `HashMap`.
    #[inline]
    fn index(&self, key: &Q) -> &V {
        self.get(key).expect("no entry found for key")
    }
}

/// An iterator over the entries of a `HashMap`.
///
/// This `struct` is created by the [`iter`] method on [`HashMap`]. See its
/// documentation for more.
///
/// [`iter`]: struct.HashMap.html#method.iter
/// [`HashMap`]: struct.HashMap.html
pub struct Iter<'a, K: 'a, V: 'a> {
    inner: table::Iter<'a, K, V>,
}

impl<K, V> Clone for Iter<'_, K, V> {
    fn clone(&self) -> Self {
        Iter { inner: self.inner.clone() }
    }
}

impl<K: Debug, V: Debug> fmt::Debug for Iter<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list()
            .entries(self.clone())
            .finish()
    }
}

/// A mutable iterator over the entries of a `HashMap`.
///
/// This `struct` is created by the [`iter_mut`] method on [`HashMap`]. See its
/// documentation for more.
///
/// [`iter_mut`]: struct.HashMap.html#method.iter_mut
/// [`HashMap`]: struct.HashMap.html
pub struct IterMut<'a, K: 'a, V: 'a> {
    inner: table::IterMut<'a, K, V>,
}

/// An owning iterator over the entries of a `HashMap`.
///
/// This `struct` is created by the [`into_iter`] method on [`HashMap`][`HashMap`]
/// (provided by the `IntoIterator` trait). See its documentation for more.
///
/// [`into_iter`]: struct.HashMap.html#method.into_iter
/// [`HashMap`]: struct.HashMap.html
pub struct IntoIter<K, V> {
    pub(super) inner: table::IntoIter<K, V>,
}

/// An iterator over the keys of a `HashMap`.
///
/// This `struct` is created by the [`keys`] method on [`HashMap`]. See its
/// documentation for more.
///
/// [`keys`]: struct.HashMap.html#method.keys
/// [`HashMap`]: struct.HashMap.html
pub struct Keys<'a, K: 'a, V: 'a> {
    inner: Iter<'a, K, V>,
}

impl<K, V> Clone for Keys<'_, K, V> {
    fn clone(&self) -> Self {
        Keys { inner: self.inner.clone() }
    }
}

impl<K: Debug, V> fmt::Debug for Keys<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list()
            .entries(self.clone())
            .finish()
    }
}

/// An iterator over the values of a `HashMap`.
///
/// This `struct` is created by the [`values`] method on [`HashMap`]. See its
/// documentation for more.
///
/// [`values`]: struct.HashMap.html#method.values
/// [`HashMap`]: struct.HashMap.html
pub struct Values<'a, K: 'a, V: 'a> {
    inner: Iter<'a, K, V>,
}

impl<K, V> Clone for Values<'_, K, V> {
    fn clone(&self) -> Self {
        Values { inner: self.inner.clone() }
    }
}

impl<K, V: Debug> fmt::Debug for Values<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list()
            .entries(self.clone())
            .finish()
    }
}

/// A draining iterator over the entries of a `HashMap`.
///
/// This `struct` is created by the [`drain`] method on [`HashMap`]. See its
/// documentation for more.
///
/// [`drain`]: struct.HashMap.html#method.drain
/// [`HashMap`]: struct.HashMap.html
pub struct Drain<'a, K: 'a, V: 'a> {
    pub(super) inner: table::Drain<'a, K, V>,
}

/// A mutable iterator over the values of a `HashMap`.
///
/// This `struct` is created by the [`values_mut`] method on [`HashMap`]. See its
/// documentation for more.
///
/// [`values_mut`]: struct.HashMap.html#method.values_mut
/// [`HashMap`]: struct.HashMap.html
pub struct ValuesMut<'a, K: 'a, V: 'a> {
    inner: IterMut<'a, K, V>,
}

enum InternalEntry<K, V, M> {
    Occupied { elem: FullBucket<K, V, M> },
    Vacant {
        hash: SafeHash,
        elem: VacantEntryState<K, V, M>,
    },
    TableIsEmpty,
}

impl<K, V, M> InternalEntry<K, V, M> {
    #[inline]
    fn into_occupied_bucket(self) -> Option<FullBucket<K, V, M>> {
        match self {
            InternalEntry::Occupied { elem } => Some(elem),
            _ => None,
        }
    }
}

impl<'a, K, V> InternalEntry<K, V, &'a mut RawTable<K, V>> {
    #[inline]
    fn into_entry(self, key: K) -> Option<Entry<'a, K, V>> {
        match self {
            InternalEntry::Occupied { elem } => {
                Some(Occupied(OccupiedEntry {
                    key: Some(key),
                    elem,
                }))
            }
            InternalEntry::Vacant { hash, elem } => {
                Some(Vacant(VacantEntry {
                    hash,
                    key,
                    elem,
                }))
            }
            InternalEntry::TableIsEmpty => None,
        }
    }
}

/// A builder for computing where in a HashMap a key-value pair would be stored.
///
/// See the [`HashMap::raw_entry_mut`] docs for usage examples.
///
/// [`HashMap::raw_entry_mut`]: struct.HashMap.html#method.raw_entry_mut
pub struct RawEntryBuilderMut<'a, K: 'a, V: 'a, S: 'a> {
    map: &'a mut HashMap<K, V, S>,
}

/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This is a lower-level version of [`Entry`].
///
/// This `enum` is constructed from the [`raw_entry`] method on [`HashMap`].
///
/// [`HashMap`]: struct.HashMap.html
/// [`Entry`]: enum.Entry.html
/// [`raw_entry`]: struct.HashMap.html#method.raw_entry
pub enum RawEntryMut<'a, K: 'a, V: 'a, S: 'a> {
    /// An occupied entry.
    Occupied(RawOccupiedEntryMut<'a, K, V>),
    /// A vacant entry.
    Vacant(RawVacantEntryMut<'a, K, V, S>),
}

/// A view into an occupied entry in a `HashMap`.
/// It is part of the [`RawEntryMut`] enum.
///
/// [`RawEntryMut`]: enum.RawEntryMut.html
pub struct RawOccupiedEntryMut<'a, K: 'a, V: 'a> {
    elem: FullBucket<K, V, &'a mut RawTable<K, V>>,
}

/// A view into a vacant entry in a `HashMap`.
/// It is part of the [`RawEntryMut`] enum.
///
/// [`RawEntryMut`]: enum.RawEntryMut.html
pub struct RawVacantEntryMut<'a, K: 'a, V: 'a, S: 'a> {
    elem: VacantEntryState<K, V, &'a mut RawTable<K, V>>,
    hash_builder: &'a S,
}

/// A builder for computing where in a HashMap a key-value pair would be stored.
///
/// See the [`HashMap::raw_entry`] docs for usage examples.
///
/// [`HashMap::raw_entry`]: struct.HashMap.html#method.raw_entry
pub struct RawEntryBuilder<'a, K: 'a, V: 'a, S: 'a> {
    map: &'a HashMap<K, V, S>,
}

impl<'a, K, V, S> RawEntryBuilderMut<'a, K, V, S>
    where S: BuildHasher,
          K: Eq + Hash,
{
    /// Create a `RawEntryMut` from the given key.
    pub fn from_key<Q: ?Sized>(self, k: &Q) -> RawEntryMut<'a, K, V, S>
        where K: Borrow<Q>,
              Q: Hash + Eq
    {
        let mut hasher = self.map.hash_builder.build_hasher();
        k.hash(&mut hasher);
        self.from_key_hashed_nocheck(hasher.finish(), k)
    }

    /// Create a `RawEntryMut` from the given key and its hash.
    #[inline]
    pub fn from_key_hashed_nocheck<Q: ?Sized>(self, hash: u64, k: &Q) -> RawEntryMut<'a, K, V, S>
        where K: Borrow<Q>,
              Q: Eq
    {
        self.from_hash(hash, |q| q.borrow().eq(k))
    }

    #[inline]
    fn search<F>(self, hash: u64, is_match: F, compare_hashes: bool)  -> RawEntryMut<'a, K, V, S>
        where for<'b> F: FnMut(&'b K) -> bool,
    {
        match search_hashed_nonempty_mut(&mut self.map.table,
                                         SafeHash::new(hash),
                                         is_match,
                                         compare_hashes) {
            InternalEntry::Occupied { elem } => {
                RawEntryMut::Occupied(RawOccupiedEntryMut { elem })
            }
            InternalEntry::Vacant { elem, .. } => {
                RawEntryMut::Vacant(RawVacantEntryMut {
                    elem,
                    hash_builder: &self.map.hash_builder,
                })
            }
            InternalEntry::TableIsEmpty => {
                unreachable!()
            }
        }
    }
    /// Create a `RawEntryMut` from the given hash.
    #[inline]
    pub fn from_hash<F>(self, hash: u64, is_match: F) -> RawEntryMut<'a, K, V, S>
        where for<'b> F: FnMut(&'b K) -> bool,
    {
        self.search(hash, is_match, true)
    }

    /// Search possible locations for an element with hash `hash` until `is_match` returns true for
    /// one of them. There is no guarantee that all keys passed to `is_match` will have the provided
    /// hash.
    pub fn search_bucket<F>(self, hash: u64, is_match: F) -> RawEntryMut<'a, K, V, S>
        where for<'b> F: FnMut(&'b K) -> bool,
    {
        self.search(hash, is_match, false)
    }
}

impl<'a, K, V, S> RawEntryBuilder<'a, K, V, S>
    where S: BuildHasher,
{
    /// Access an entry by key.
    pub fn from_key<Q: ?Sized>(self, k: &Q) -> Option<(&'a K, &'a V)>
        where K: Borrow<Q>,
              Q: Hash + Eq
    {
        let mut hasher = self.map.hash_builder.build_hasher();
        k.hash(&mut hasher);
        self.from_key_hashed_nocheck(hasher.finish(), k)
    }

    /// Access an entry by a key and its hash.
    pub fn from_key_hashed_nocheck<Q: ?Sized>(self, hash: u64, k: &Q) -> Option<(&'a K, &'a V)>
        where K: Borrow<Q>,
              Q: Hash + Eq

    {
        self.from_hash(hash, |q| q.borrow().eq(k))
    }

    fn search<F>(self, hash: u64, is_match: F, compare_hashes: bool) -> Option<(&'a K, &'a V)>
        where F: FnMut(&K) -> bool
    {
        if unsafe { unlikely(self.map.table.size() == 0) } {
            return None;
        }
        match search_hashed_nonempty(&self.map.table,
                                     SafeHash::new(hash),
                                     is_match,
                                     compare_hashes) {
            InternalEntry::Occupied { elem } => Some(elem.into_refs()),
            InternalEntry::Vacant { .. } => None,
            InternalEntry::TableIsEmpty => unreachable!(),
        }
    }

    /// Access an entry by hash.
    pub fn from_hash<F>(self, hash: u64, is_match: F) -> Option<(&'a K, &'a V)>
        where F: FnMut(&K) -> bool
    {
        self.search(hash, is_match, true)
    }

    /// Search possible locations for an element with hash `hash` until `is_match` returns true for
    /// one of them. There is no guarantee that all keys passed to `is_match` will have the provided
    /// hash.
    pub fn search_bucket<F>(self, hash: u64, is_match: F) -> Option<(&'a K, &'a V)>
        where F: FnMut(&K) -> bool
    {
        self.search(hash, is_match, false)
    }
}

impl<'a, K, V, S> RawEntryMut<'a, K, V, S> {
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// mutable references to the key and value in the entry.
    ///
    pub fn or_insert(self, default_key: K, default_val: V) -> (&'a mut K, &'a mut V)
        where K: Hash,
              S: BuildHasher,
    {
        match self {
            RawEntryMut::Occupied(entry) => entry.into_key_value(),
            RawEntryMut::Vacant(entry) => entry.insert(default_key, default_val),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns mutable references to the key and value in the entry.
    ///
    pub fn or_insert_with<F>(self, default: F) -> (&'a mut K, &'a mut V)
        where F: FnOnce() -> (K, V),
              K: Hash,
              S: BuildHasher,
    {
        match self {
            RawEntryMut::Occupied(entry) => entry.into_key_value(),
            RawEntryMut::Vacant(entry) => {
                let (k, v) = default();
                entry.insert(k, v)
            }
        }
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    ///
    pub fn and_modify<F>(self, f: F) -> Self
        where F: FnOnce(&mut K, &mut V)
    {
        match self {
            RawEntryMut::Occupied(mut entry) => {
                {
                    let (k, v) = entry.get_key_value_mut();
                    f(k, v);
                }
                RawEntryMut::Occupied(entry)
            },
            RawEntryMut::Vacant(entry) => RawEntryMut::Vacant(entry),
        }
    }
}

impl<'a, K, V> RawOccupiedEntryMut<'a, K, V> {
    /// Gets a reference to the key in the entry.
    pub fn key(&self) -> &K {
        self.elem.read().0
    }

    /// Gets a mutable reference to the key in the entry.
    pub fn key_mut(&mut self) -> &mut K {
        self.elem.read_mut().0
    }

    /// Converts the entry into a mutable reference to the key in the entry
    /// with a lifetime bound to the map itself.
    pub fn into_key(self) -> &'a mut K {
        self.elem.into_mut_refs().0
    }

    /// Gets a reference to the value in the entry.
    pub fn get(&self) -> &V {
        self.elem.read().1
    }

    /// Converts the OccupiedEntry into a mutable reference to the value in the entry
    /// with a lifetime bound to the map itself.
    pub fn into_mut(self) -> &'a mut V {
        self.elem.into_mut_refs().1
    }

    /// Gets a mutable reference to the value in the entry.
    pub fn get_mut(&mut self) -> &mut V {
        self.elem.read_mut().1
    }

    /// Gets a reference to the key and value in the entry.
    pub fn get_key_value(&mut self) -> (&K, &V) {
        self.elem.read()
    }

    /// Gets a mutable reference to the key and value in the entry.
    pub fn get_key_value_mut(&mut self) -> (&mut K, &mut V) {
        self.elem.read_mut()
    }

    /// Converts the OccupiedEntry into a mutable reference to the key and value in the entry
    /// with a lifetime bound to the map itself.
    pub fn into_key_value(self) -> (&'a mut K, &'a mut V) {
        self.elem.into_mut_refs()
    }

    /// Sets the value of the entry, and returns the entry's old value.
    pub fn insert(&mut self, value: V) -> V {
        mem::replace(self.get_mut(), value)
    }

    /// Sets the value of the entry, and returns the entry's old value.
    pub fn insert_key(&mut self, key: K) -> K {
        mem::replace(self.key_mut(), key)
    }

    /// Takes the value out of the entry, and returns it.
    pub fn remove(self) -> V {
        pop_internal(self.elem).1
    }

    /// Take the ownership of the key and value from the map.
    pub fn remove_entry(self) -> (K, V) {
        let (k, v, _) = pop_internal(self.elem);
        (k, v)
    }
}

impl<'a, K, V, S> RawVacantEntryMut<'a, K, V, S> {
    /// Sets the value of the entry with the VacantEntry's key,
    /// and returns a mutable reference to it.
    pub fn insert(self, key: K, value: V) -> (&'a mut K, &'a mut V)
        where K: Hash,
              S: BuildHasher,
    {
        let mut hasher = self.hash_builder.build_hasher();
        key.hash(&mut hasher);
        self.insert_hashed_nocheck(hasher.finish(), key, value)
    }

    /// Sets the value of the entry with the VacantEntry's key,
    /// and returns a mutable reference to it.
    #[inline]
    pub fn insert_hashed_nocheck(self, hash: u64, key: K, value: V) -> (&'a mut K, &'a mut V) {
        let hash = SafeHash::new(hash);
        let b = match self.elem {
            NeqElem(mut bucket, disp) => {
                if disp >= DISPLACEMENT_THRESHOLD {
                    bucket.table_mut().set_tag(true);
                }
                robin_hood(bucket, disp, hash, key, value)
            },
            NoElem(mut bucket, disp) => {
                if disp >= DISPLACEMENT_THRESHOLD {
                    bucket.table_mut().set_tag(true);
                }
                bucket.put(hash, key, value)
            },
        };
        b.into_mut_refs()
    }
}

impl<K, V, S> Debug for RawEntryBuilderMut<'_, K, V, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RawEntryBuilder")
         .finish()
    }
}

impl<K: Debug, V: Debug, S> Debug for RawEntryMut<'_, K, V, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RawEntryMut::Vacant(ref v) => {
                f.debug_tuple("RawEntry")
                    .field(v)
                    .finish()
            }
            RawEntryMut::Occupied(ref o) => {
                f.debug_tuple("RawEntry")
                    .field(o)
                    .finish()
            }
        }
    }
}

impl<K: Debug, V: Debug> Debug for RawOccupiedEntryMut<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RawOccupiedEntryMut")
         .field("key", self.key())
         .field("value", self.get())
         .finish()
    }
}

impl<K, V, S> Debug for RawVacantEntryMut<'_, K, V, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RawVacantEntryMut")
         .finish()
    }
}

impl<K, V, S> Debug for RawEntryBuilder<'_, K, V, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RawEntryBuilder")
         .finish()
    }
}

/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This `enum` is constructed from the [`entry`] method on [`HashMap`].
///
/// [`HashMap`]: struct.HashMap.html
/// [`entry`]: struct.HashMap.html#method.entry
pub enum Entry<'a, K: 'a, V: 'a> {
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, K, V>),

    /// A vacant entry.
    Vacant(VacantEntry<'a, K, V>),
}

impl<K: Debug, V: Debug> Debug for Entry<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Vacant(ref v) => {
                f.debug_tuple("Entry")
                    .field(v)
                    .finish()
            }
            Occupied(ref o) => {
                f.debug_tuple("Entry")
                    .field(o)
                    .finish()
            }
        }
    }
}

/// A view into an occupied entry in a `HashMap`.
/// It is part of the [`Entry`] enum.
///
/// [`Entry`]: enum.Entry.html
pub struct OccupiedEntry<'a, K: 'a, V: 'a> {
    key: Option<K>,
    elem: FullBucket<K, V, &'a mut RawTable<K, V>>,
}

impl<K: Debug, V: Debug> Debug for OccupiedEntry<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("OccupiedEntry")
            .field("key", self.key())
            .field("value", self.get())
            .finish()
    }
}

/// A view into a vacant entry in a `HashMap`.
/// It is part of the [`Entry`] enum.
///
/// [`Entry`]: enum.Entry.html
pub struct VacantEntry<'a, K: 'a, V: 'a> {
    hash: SafeHash,
    key: K,
    elem: VacantEntryState<K, V, &'a mut RawTable<K, V>>,
}

impl<K: Debug, V> Debug for VacantEntry<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("VacantEntry")
            .field(self.key())
            .finish()
    }
}

/// Possible states of a VacantEntry.
enum VacantEntryState<K, V, M> {
    /// The index is occupied, but the key to insert has precedence,
    /// and will kick the current one out on insertion.
    NeqElem(FullBucket<K, V, M>, usize),
    /// The index is genuinely vacant.
    NoElem(EmptyBucket<K, V, M>, usize),
}

impl<'a, K, V, S> IntoIterator for &'a HashMap<K, V, S> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Iter<'a, K, V> {
        self.iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut HashMap<K, V, S> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> IterMut<'a, K, V> {
        self.iter_mut()
    }
}

impl<K, V, S> IntoIterator for HashMap<K, V, S> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    /// Creates a consuming iterator, that is, one that moves each key-value
    /// pair out of the map in arbitrary order. The map cannot be used after
    /// calling this.
    ///
    fn into_iter(self) -> IntoIter<K, V> {
        IntoIter { inner: self.table.into_iter() }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        self.inner.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> FusedIterator for Iter<'_, K, V> {}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<(&'a K, &'a mut V)> {
        self.inner.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V> ExactSizeIterator for IterMut<'_, K, V> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> FusedIterator for IterMut<'_, K, V> {}

impl<K, V> fmt::Debug for IterMut<'_, K, V>
    where K: fmt::Debug,
          V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list()
            .entries(self.inner.iter())
            .finish()
    }
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<(K, V)> {
        self.inner.next().map(|(_, k, v)| (k, v))
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> FusedIterator for IntoIter<K, V> {}

impl<K: Debug, V: Debug> fmt::Debug for IntoIter<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list()
            .entries(self.inner.iter())
            .finish()
    }
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    #[inline]
    fn next(&mut self) -> Option<(&'a K)> {
        self.inner.next().map(|(k, _)| k)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V> ExactSizeIterator for Keys<'_, K, V> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> FusedIterator for Keys<'_, K, V> {}


impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    #[inline]
    fn next(&mut self) -> Option<(&'a V)> {
        self.inner.next().map(|(_, v)| v)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V> ExactSizeIterator for Values<'_, K, V> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> FusedIterator for Values<'_, K, V> {}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    #[inline]
    fn next(&mut self) -> Option<(&'a mut V)> {
        self.inner.next().map(|(_, v)| v)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V> ExactSizeIterator for ValuesMut<'_, K, V> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> FusedIterator for ValuesMut<'_, K, V> {}

impl<K, V> fmt::Debug for ValuesMut<'_, K, V>
    where K: fmt::Debug,
          V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list()
            .entries(self.inner.inner.iter())
            .finish()
    }
}

impl<'a, K, V> Iterator for Drain<'a, K, V> {
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<(K, V)> {
        self.inner.next().map(|(_, k, v)| (k, v))
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V> ExactSizeIterator for Drain<'_, K, V> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> FusedIterator for Drain<'_, K, V> {}

impl<K, V> fmt::Debug for Drain<'_, K, V>
    where K: fmt::Debug,
          V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list()
            .entries(self.inner.iter())
            .finish()
    }
}

impl<'a, K, V> Entry<'a, K, V> {
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    ///
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(default),
        }
    }


    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    ///
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(default()),
        }
    }

    /// Returns a reference to this entry's key.
    ///
    pub fn key(&self) -> &K {
        match *self {
            Occupied(ref entry) => entry.key(),
            Vacant(ref entry) => entry.key(),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    ///
    pub fn and_modify<F>(self, f: F) -> Self
        where F: FnOnce(&mut V)
    {
        match self {
            Occupied(mut entry) => {
                f(entry.get_mut());
                Occupied(entry)
            },
            Vacant(entry) => Vacant(entry),
        }
    }
}

impl<'a, K, V: Default> Entry<'a, K, V> {
    /// Ensures a value is in the entry by inserting the default value if empty,
    /// and returns a mutable reference to the value in the entry.
    ///
    pub fn or_default(self) -> &'a mut V {
        match self {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(Default::default()),
        }
    }
}

impl<'a, K, V> OccupiedEntry<'a, K, V> {
    /// Gets a reference to the key in the entry.
    ///
    pub fn key(&self) -> &K {
        self.elem.read().0
    }

    /// Take the ownership of the key and value from the map.
    ///
    pub fn remove_entry(self) -> (K, V) {
        let (k, v, _) = pop_internal(self.elem);
        (k, v)
    }

    /// Gets a reference to the value in the entry.
    ///
    pub fn get(&self) -> &V {
        self.elem.read().1
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// If you need a reference to the `OccupiedEntry` which may outlive the
    /// destruction of the `Entry` value, see [`into_mut`].
    ///
    pub fn get_mut(&mut self) -> &mut V {
        self.elem.read_mut().1
    }

    /// Converts the OccupiedEntry into a mutable reference to the value in the entry
    /// with a lifetime bound to the map itself.
    ///
    /// If you need multiple references to the `OccupiedEntry`, see [`get_mut`].
    ///
    pub fn into_mut(self) -> &'a mut V {
        self.elem.into_mut_refs().1
    }

    /// Sets the value of the entry, and returns the entry's old value.
    ///
    pub fn insert(&mut self, mut value: V) -> V {
        let old_value = self.get_mut();
        mem::swap(&mut value, old_value);
        value
    }

    /// Takes the value out of the entry, and returns it.
    ///
    pub fn remove(self) -> V {
        pop_internal(self.elem).1
    }

    /// Returns a key that was used for search.
    ///
    /// The key was retained for further use.
    fn take_key(&mut self) -> Option<K> {
        self.key.take()
    }

    /// Replaces the entry, returning the old key and value. The new key in the hash map will be
    /// the key used to create this entry.
    ///
    pub fn replace_entry(mut self, value: V) -> (K, V) {
        let (old_key, old_value) = self.elem.read_mut();

        let old_key = mem::replace(old_key, self.key.unwrap());
        let old_value = mem::replace(old_value, value);

        (old_key, old_value)
    }

    /// Replaces the key in the hash map with the key used to create this entry.
    ///
    pub fn replace_key(mut self) -> K {
        let (old_key, _) = self.elem.read_mut();
        mem::replace(old_key, self.key.unwrap())
    }
}

impl<'a, K: 'a, V: 'a> VacantEntry<'a, K, V> {
    /// Gets a reference to the key that would be used when inserting a value
    /// through the `VacantEntry`.
    ///
    pub fn key(&self) -> &K {
        &self.key
    }

    /// Take ownership of the key.
    ///
    pub fn into_key(self) -> K {
        self.key
    }

    /// Sets the value of the entry with the VacantEntry's key,
    /// and returns a mutable reference to it.
    ///
    pub fn insert(self, value: V) -> &'a mut V {
        let b = match self.elem {
            NeqElem(mut bucket, disp) => {
                if disp >= DISPLACEMENT_THRESHOLD {
                    bucket.table_mut().set_tag(true);
                }
                robin_hood(bucket, disp, self.hash, self.key, value)
            },
            NoElem(mut bucket, disp) => {
                if disp >= DISPLACEMENT_THRESHOLD {
                    bucket.table_mut().set_tag(true);
                }
                bucket.put(self.hash, self.key, value)
            },
        };
        b.into_mut_refs().1
    }
}

impl<K, V, S> FromIterator<(K, V)> for HashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher + Default
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> HashMap<K, V, S> {
        let mut map = HashMap::with_hasher(Default::default());
        map.extend(iter);
        map
    }
}

impl<K, V, S> Extend<(K, V)> for HashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        // Keys may be already present or show multiple times in the iterator.
        // Reserve the entire hint lower bound if the map is empty.
        // Otherwise reserve half the hint (rounded up), so the map
        // will only resize twice in the worst case.
        let iter = iter.into_iter();
        let reserve = if self.is_empty() {
            iter.size_hint().0
        } else {
            (iter.size_hint().0 + 1) / 2
        };
        self.reserve(reserve);
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<'a, K, V, S> Extend<(&'a K, &'a V)> for HashMap<K, V, S>
    where K: Eq + Hash + Copy,
          V: Copy,
          S: BuildHasher
{
    fn extend<T: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: T) {
        self.extend(iter.into_iter().map(|(&key, &value)| (key, value)));
    }
}

/// `RandomState` is the default state for [`HashMap`] types.
///
/// A particular instance `RandomState` will create the same instances of
/// [`Hasher`], but the hashers created by two different `RandomState`
/// instances are unlikely to produce the same result for the same values.
///
/// [`HashMap`]: struct.HashMap.html
/// [`Hasher`]: ../../hash/trait.Hasher.html
///
#[derive(Clone)]
pub struct RandomState {
    k0: u64,
    k1: u64,
}

impl RandomState {
    /// Constructs a new `RandomState` that is initialized with random keys.
    ///
    #[inline]
    #[allow(deprecated)]
    // rand
    pub fn new() -> RandomState {
        // Historically this function did not cache keys from the OS and instead
        // simply always called `rand::thread_rng().gen()` twice. In #31356 it
        // was discovered, however, that because we re-seed the thread-local RNG
        // from the OS periodically that this can cause excessive slowdown when
        // many hash maps are created on a thread. To solve this performance
        // trap we cache the first set of randomly generated keys per-thread.
        //
        // Later in #36481 it was discovered that exposing a deterministic
        // iteration order allows a form of DOS attack. To counter that we
        // increment one of the seeds on every RandomState creation, giving
        // every corresponding HashMap a different iteration order.
        thread_local!(static KEYS: Cell<(u64, u64)> = {
            Cell::new(sys::hashmap_random_keys())
        });

        KEYS.with(|keys| {
            let (k0, k1) = keys.get();
            keys.set((k0.wrapping_add(1), k1));
            RandomState { k0: k0, k1: k1 }
        })
    }
}

impl BuildHasher for RandomState {
    type Hasher = DefaultHasher;
    #[inline]
    #[allow(deprecated)]
    fn build_hasher(&self) -> DefaultHasher {
        DefaultHasher(SipHasher13::new_with_keys(self.k0, self.k1))
    }
}

/// The default [`Hasher`] used by [`RandomState`].
///
/// The internal algorithm is not specified, and so it and its hashes should
/// not be relied upon over releases.
///
/// [`RandomState`]: struct.RandomState.html
/// [`Hasher`]: ../../hash/trait.Hasher.html
#[allow(deprecated)]
#[derive(Clone, Debug)]
pub struct DefaultHasher(SipHasher13);

impl DefaultHasher {
    /// Creates a new `DefaultHasher`.
    ///
    /// This hasher is not guaranteed to be the same as all other
    /// `DefaultHasher` instances, but is the same as all other `DefaultHasher`
    /// instances created through `new` or `default`.
    #[allow(deprecated)]
    pub fn new() -> DefaultHasher {
        DefaultHasher(SipHasher13::new_with_keys(0, 0))
    }
}

impl Default for DefaultHasher {
    /// Creates a new `DefaultHasher` using [`new`][DefaultHasher::new].
    /// See its documentation for more.
    fn default() -> DefaultHasher {
        DefaultHasher::new()
    }
}

impl Hasher for DefaultHasher {
    #[inline]
    fn write(&mut self, msg: &[u8]) {
        self.0.write(msg)
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0.finish()
    }
}

impl Default for RandomState {
    /// Constructs a new `RandomState`.
    #[inline]
    fn default() -> RandomState {
        RandomState::new()
    }
}

impl fmt::Debug for RandomState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("RandomState { .. }")
    }
}

impl<K, S, Q: ?Sized> super::Recover<Q> for HashMap<K, (), S>
    where K: Eq + Hash + Borrow<Q>,
          S: BuildHasher,
          Q: Eq + Hash
{
    type Key = K;

    #[inline]
    fn get(&self, key: &Q) -> Option<&K> {
        self.search(key).map(|bucket| bucket.into_refs().0)
    }

    fn take(&mut self, key: &Q) -> Option<K> {
        self.search_mut(key).map(|bucket| pop_internal(bucket).0)
    }

    #[inline]
    fn replace(&mut self, key: K) -> Option<K> {
        self.reserve(1);

        match self.entry(key) {
            Occupied(mut occupied) => {
                let key = occupied.take_key().unwrap();
                Some(mem::replace(occupied.elem.read_mut().0, key))
            }
            Vacant(vacant) => {
                vacant.insert(());
                None
            }
        }
    }
}

#[allow(dead_code)]
fn assert_covariance() {
    fn map_key<'new>(v: HashMap<&'static str, u8>) -> HashMap<&'new str, u8> {
        v
    }
    fn map_val<'new>(v: HashMap<u8, &'static str>) -> HashMap<u8, &'new str> {
        v
    }
    fn iter_key<'a, 'new>(v: Iter<'a, &'static str, u8>) -> Iter<'a, &'new str, u8> {
        v
    }
    fn iter_val<'a, 'new>(v: Iter<'a, u8, &'static str>) -> Iter<'a, u8, &'new str> {
        v
    }
    fn into_iter_key<'new>(v: IntoIter<&'static str, u8>) -> IntoIter<&'new str, u8> {
        v
    }
    fn into_iter_val<'new>(v: IntoIter<u8, &'static str>) -> IntoIter<u8, &'new str> {
        v
    }
    fn keys_key<'a, 'new>(v: Keys<'a, &'static str, u8>) -> Keys<'a, &'new str, u8> {
        v
    }
    fn keys_val<'a, 'new>(v: Keys<'a, u8, &'static str>) -> Keys<'a, u8, &'new str> {
        v
    }
    fn values_key<'a, 'new>(v: Values<'a, &'static str, u8>) -> Values<'a, &'new str, u8> {
        v
    }
    fn values_val<'a, 'new>(v: Values<'a, u8, &'static str>) -> Values<'a, u8, &'new str> {
        v
    }
    fn drain<'new>(d: Drain<'static, &'static str, &'static str>)
                   -> Drain<'new, &'new str, &'new str> {
        d
    }
}