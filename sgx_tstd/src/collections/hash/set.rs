// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
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

use core::borrow::Borrow;
use core::fmt;
use core::hash::{Hash, BuildHasher};
use core::iter::{Chain, FromIterator, FusedIterator};
use core::ops::{BitOr, BitAnd, BitXor, Sub};

use super::Recover;
use super::map::{self, HashMap, Keys, RandomState};

// Future Optimization (FIXME!)
// =============================
//
// Iteration over zero sized values is a noop. There is no need
// for `bucket.val` in the case of HashSet. I suppose we would need HKT
// to get rid of it properly.

/// A hash set implemented as a `HashMap` where the value is `()`.
///
/// As with the [`HashMap`] type, a `HashSet` requires that the elements
/// implement the [`Eq`] and [`Hash`] traits. This can frequently be achieved by
/// using `#[derive(PartialEq, Eq, Hash)]`. If you implement these yourself,
/// it is important that the following property holds:
///
/// ```text
/// k1 == k2 -> hash(k1) == hash(k2)
/// ```
///
/// In other words, if two keys are equal, their hashes must be equal.
///
///
/// It is a logic error for an item to be modified in such a way that the
/// item's hash, as determined by the [`Hash`] trait, or its equality, as
/// determined by the [`Eq`] trait, changes while it is in the set. This is
/// normally only possible through [`Cell`], [`RefCell`], global state, I/O, or
/// unsafe code.
///
#[derive(Clone)]
pub struct HashSet<T, S = RandomState> {
    map: HashMap<T, (), S>,
}

impl<T: Hash + Eq> HashSet<T, RandomState> {
    /// Creates an empty `HashSet`.
    ///
    /// The hash set is initially created with a capacity of 0, so it will not allocate until it
    /// is first inserted into.
    ///
    #[inline]
    pub fn new() -> HashSet<T, RandomState> {
        HashSet { map: HashMap::new() }
    }

    /// Creates an empty `HashSet` with the specified capacity.
    ///
    /// The hash set will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash set will not allocate.
    ///
    #[inline]
    pub fn with_capacity(capacity: usize) -> HashSet<T, RandomState> {
        HashSet { map: HashMap::with_capacity(capacity) }
    }
}

impl<T, S> HashSet<T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
    /// Creates a new empty hash set which will use the given hasher to hash
    /// keys.
    ///
    /// The hash set is also created with the default initial capacity.
    ///
    /// Warning: `hasher` is normally randomly generated, and
    /// is designed to allow `HashSet`s to be resistant to attacks that
    /// cause many collisions and very poor performance. Setting it
    /// manually using this function can expose a DoS attack vector.
    ///
    #[inline]
    pub fn with_hasher(hasher: S) -> HashSet<T, S> {
        HashSet { map: HashMap::with_hasher(hasher) }
    }

    /// Creates an empty `HashSet` with with the specified capacity, using
    /// `hasher` to hash the keys.
    ///
    /// The hash set will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash set will not allocate.
    ///
    /// Warning: `hasher` is normally randomly generated, and
    /// is designed to allow `HashSet`s to be resistant to attacks that
    /// cause many collisions and very poor performance. Setting it
    /// manually using this function can expose a DoS attack vector.
    ///
    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> HashSet<T, S> {
        HashSet { map: HashMap::with_capacity_and_hasher(capacity, hasher) }
    }

    /// Returns a reference to the set's [`BuildHasher`].
    ///
    /// [`BuildHasher`]: ../../std/hash/trait.BuildHasher.html
    pub fn hasher(&self) -> &S {
        self.map.hasher()
    }

    /// Returns the number of elements the set can hold without reallocating.
    ///
    #[inline]
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the `HashSet`. The collection may reserve more space to avoid
    /// frequent reallocations.
    ///
    /// # Panics
    ///
    /// Panics if the new allocation size overflows `usize`.
    ///
    pub fn reserve(&mut self, additional: usize) {
        self.map.reserve(additional)
    }

    /// Shrinks the capacity of the set as much as possible. It will drop
    /// down as much as possible while maintaining the internal rules
    /// and possibly leaving some space in accordance with the resize policy.
    ///
    pub fn shrink_to_fit(&mut self) {
        self.map.shrink_to_fit()
    }

    /// An iterator visiting all elements in arbitrary order.
    /// The iterator element type is `&'a T`.
    ///
    pub fn iter(&self) -> Iter<T> {
        Iter { iter: self.map.keys() }
    }

    /// Visits the values representing the difference,
    /// i.e. the values that are in `self` but not in `other`.
    ///
    pub fn difference<'a>(&'a self, other: &'a HashSet<T, S>) -> Difference<'a, T, S> {
        Difference {
            iter: self.iter(),
            other: other,
        }
    }

    /// Visits the values representing the symmetric difference,
    /// i.e. the values that are in `self` or in `other` but not in both.
    ///
    pub fn symmetric_difference<'a>(&'a self,
                                    other: &'a HashSet<T, S>)
                                    -> SymmetricDifference<'a, T, S> {
        SymmetricDifference { iter: self.difference(other).chain(other.difference(self)) }
    }

    /// Visits the values representing the intersection,
    /// i.e. the values that are both in `self` and `other`.
    ///
    pub fn intersection<'a>(&'a self, other: &'a HashSet<T, S>) -> Intersection<'a, T, S> {
        Intersection {
            iter: self.iter(),
            other: other,
        }
    }

    /// Visits the values representing the union,
    /// i.e. all the values in `self` or `other`, without duplicates.
    ///
    pub fn union<'a>(&'a self, other: &'a HashSet<T, S>) -> Union<'a, T, S> {
        Union { iter: self.iter().chain(other.difference(self)) }
    }

    /// Returns the number of elements in the set.
    ///
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if the set contains no elements.
    ///
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clears the set, returning all elements in an iterator.
    #[inline]
    pub fn drain(&mut self) -> Drain<T> {
        Drain { iter: self.map.drain() }
    }

    /// Clears the set, removing all values.
    ///
    pub fn clear(&mut self) {
        self.map.clear()
    }

    /// Returns `true` if the set contains a value.
    ///
    /// The value may be any borrowed form of the set's value type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the value type.
    ///
    pub fn contains<Q: ?Sized>(&self, value: &Q) -> bool
        where T: Borrow<Q>,
              Q: Hash + Eq
    {
        self.map.contains_key(value)
    }

    /// Returns a reference to the value in the set, if any, that is equal to the given value.
    ///
    /// The value may be any borrowed form of the set's value type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the value type.
    ///
    /// [`Eq`]: ../../std/cmp/trait.Eq.html
    /// [`Hash`]: ../../std/hash/trait.Hash.html
    pub fn get<Q: ?Sized>(&self, value: &Q) -> Option<&T>
        where T: Borrow<Q>,
              Q: Hash + Eq
    {
        Recover::get(&self.map, value)
    }

    /// Returns `true` if `self` has no elements in common with `other`.
    /// This is equivalent to checking for an empty intersection.
    ///
    pub fn is_disjoint(&self, other: &HashSet<T, S>) -> bool {
        self.iter().all(|v| !other.contains(v))
    }

    /// Returns `true` if the set is a subset of another,
    /// i.e. `other` contains at least all the values in `self`.
    ///
    pub fn is_subset(&self, other: &HashSet<T, S>) -> bool {
        self.iter().all(|v| other.contains(v))
    }

    /// Returns `true` if the set is a superset of another,
    /// i.e. `self` contains at least all the values in `other`.
    ///
    #[inline]
    pub fn is_superset(&self, other: &HashSet<T, S>) -> bool {
        other.is_subset(self)
    }

    /// Adds a value to the set.
    ///
    /// If the set did not have this value present, `true` is returned.
    ///
    /// If the set did have this value present, `false` is returned.
    ///
    pub fn insert(&mut self, value: T) -> bool {
        self.map.insert(value, ()).is_none()
    }

    /// Adds a value to the set, replacing the existing value, if any, that is equal to the given
    /// one. Returns the replaced value.
    pub fn replace(&mut self, value: T) -> Option<T> {
        Recover::replace(&mut self.map, value)
    }

    /// Removes a value from the set. Returns `true` if the value was
    /// present in the set.
    ///
    /// The value may be any borrowed form of the set's value type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the value type.
    ///
    pub fn remove<Q: ?Sized>(&mut self, value: &Q) -> bool
        where T: Borrow<Q>,
              Q: Hash + Eq
    {
        self.map.remove(value).is_some()
    }

    /// Removes and returns the value in the set, if any, that is equal to the given one.
    ///
    /// The value may be any borrowed form of the set's value type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the value type.
    ///
    /// [`Eq`]: ../../std/cmp/trait.Eq.html
    /// [`Hash`]: ../../std/hash/trait.Hash.html
    pub fn take<Q: ?Sized>(&mut self, value: &Q) -> Option<T>
        where T: Borrow<Q>,
              Q: Hash + Eq
    {
        Recover::take(&mut self.map, value)
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` such that `f(&e)` returns `false`.
    ///
    pub fn retain<F>(&mut self, mut f: F)
        where F: FnMut(&T) -> bool
    {
        self.map.retain(|k, _| f(k));
    }
}

impl<T, S> PartialEq for HashSet<T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
    fn eq(&self, other: &HashSet<T, S>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().all(|key| other.contains(key))
    }
}

impl<T, S> Eq for HashSet<T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
}

impl<T, S> fmt::Debug for HashSet<T, S>
    where T: Eq + Hash + fmt::Debug,
          S: BuildHasher
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<T, S> FromIterator<T> for HashSet<T, S>
    where T: Eq + Hash,
          S: BuildHasher + Default
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> HashSet<T, S> {
        let mut set = HashSet::with_hasher(Default::default());
        set.extend(iter);
        set
    }
}

impl<T, S> Extend<T> for HashSet<T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.map.extend(iter.into_iter().map(|k| (k, ())));
    }
}

impl<'a, T, S> Extend<&'a T> for HashSet<T, S>
    where T: 'a + Eq + Hash + Copy,
          S: BuildHasher
{
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}

impl<T, S> Default for HashSet<T, S>
    where T: Eq + Hash,
          S: BuildHasher + Default
{
    /// Creates an empty `HashSet<T, S>` with the `Default` value for the hasher.
    fn default() -> HashSet<T, S> {
        HashSet { map: HashMap::default() }
    }
}

impl<'a, 'b, T, S> BitOr<&'b HashSet<T, S>> for &'a HashSet<T, S>
    where T: Eq + Hash + Clone,
          S: BuildHasher + Default
{
    type Output = HashSet<T, S>;

    /// Returns the union of `self` and `rhs` as a new `HashSet<T, S>`.
    ///
    fn bitor(self, rhs: &HashSet<T, S>) -> HashSet<T, S> {
        self.union(rhs).cloned().collect()
    }
}

impl<'a, 'b, T, S> BitAnd<&'b HashSet<T, S>> for &'a HashSet<T, S>
    where T: Eq + Hash + Clone,
          S: BuildHasher + Default
{
    type Output = HashSet<T, S>;

    /// Returns the intersection of `self` and `rhs` as a new `HashSet<T, S>`.
    ///
    fn bitand(self, rhs: &HashSet<T, S>) -> HashSet<T, S> {
        self.intersection(rhs).cloned().collect()
    }
}

impl<'a, 'b, T, S> BitXor<&'b HashSet<T, S>> for &'a HashSet<T, S>
    where T: Eq + Hash + Clone,
          S: BuildHasher + Default
{
    type Output = HashSet<T, S>;

    /// Returns the symmetric difference of `self` and `rhs` as a new `HashSet<T, S>`.
    ///
    fn bitxor(self, rhs: &HashSet<T, S>) -> HashSet<T, S> {
        self.symmetric_difference(rhs).cloned().collect()
    }
}

impl<'a, 'b, T, S> Sub<&'b HashSet<T, S>> for &'a HashSet<T, S>
    where T: Eq + Hash + Clone,
          S: BuildHasher + Default
{
    type Output = HashSet<T, S>;

    /// Returns the difference of `self` and `rhs` as a new `HashSet<T, S>`.
    ///
    fn sub(self, rhs: &HashSet<T, S>) -> HashSet<T, S> {
        self.difference(rhs).cloned().collect()
    }
}

/// An iterator over the items of a `HashSet`.
///
/// This `struct` is created by the [`iter`] method on [`HashSet`].
/// See its documentation for more.
///
/// [`HashSet`]: struct.HashSet.html
/// [`iter`]: struct.HashSet.html#method.iter
pub struct Iter<'a, K: 'a> {
    iter: Keys<'a, K, ()>,
}

/// An owning iterator over the items of a `HashSet`.
///
/// This `struct` is created by the [`into_iter`] method on [`HashSet`][`HashSet`]
/// (provided by the `IntoIterator` trait). See its documentation for more.
///
/// [`HashSet`]: struct.HashSet.html
/// [`into_iter`]: struct.HashSet.html#method.into_iter
pub struct IntoIter<K> {
    iter: map::IntoIter<K, ()>,
}

/// A draining iterator over the items of a `HashSet`.
///
/// This `struct` is created by the [`drain`] method on [`HashSet`].
/// See its documentation for more.
///
/// [`HashSet`]: struct.HashSet.html
/// [`drain`]: struct.HashSet.html#method.drain
pub struct Drain<'a, K: 'a> {
    iter: map::Drain<'a, K, ()>,
}

/// A lazy iterator producing elements in the intersection of `HashSet`s.
///
/// This `struct` is created by the [`intersection`] method on [`HashSet`].
/// See its documentation for more.
///
/// [`HashSet`]: struct.HashSet.html
/// [`intersection`]: struct.HashSet.html#method.intersection
pub struct Intersection<'a, T: 'a, S: 'a> {
    // iterator of the first set
    iter: Iter<'a, T>,
    // the second set
    other: &'a HashSet<T, S>,
}

/// A lazy iterator producing elements in the difference of `HashSet`s.
///
/// This `struct` is created by the [`difference`] method on [`HashSet`].
/// See its documentation for more.
///
/// [`HashSet`]: struct.HashSet.html
/// [`difference`]: struct.HashSet.html#method.difference
pub struct Difference<'a, T: 'a, S: 'a> {
    // iterator of the first set
    iter: Iter<'a, T>,
    // the second set
    other: &'a HashSet<T, S>,
}

/// A lazy iterator producing elements in the symmetric difference of `HashSet`s.
///
/// This `struct` is created by the [`symmetric_difference`] method on
/// [`HashSet`]. See its documentation for more.
///
/// [`HashSet`]: struct.HashSet.html
/// [`symmetric_difference`]: struct.HashSet.html#method.symmetric_difference
pub struct SymmetricDifference<'a, T: 'a, S: 'a> {
    iter: Chain<Difference<'a, T, S>, Difference<'a, T, S>>,
}

/// A lazy iterator producing elements in the union of `HashSet`s.
///
/// This `struct` is created by the [`union`] method on [`HashSet`].
/// See its documentation for more.
///
/// [`HashSet`]: struct.HashSet.html
/// [`union`]: struct.HashSet.html#method.union
pub struct Union<'a, T: 'a, S: 'a> {
    iter: Chain<Iter<'a, T>, Difference<'a, T, S>>,
}

impl<'a, T, S> IntoIterator for &'a HashSet<T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<T, S> IntoIterator for HashSet<T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
    type Item = T;
    type IntoIter = IntoIter<T>;

    /// Creates a consuming iterator, that is, one that moves each value out
    /// of the set in arbitrary order. The set cannot be used after calling
    /// this.
    ///
    fn into_iter(self) -> IntoIter<T> {
        IntoIter { iter: self.map.into_iter() }
    }
}

impl<'a, K> Clone for Iter<'a, K> {
    fn clone(&self) -> Iter<'a, K> {
        Iter { iter: self.iter.clone() }
    }
}

impl<'a, K> Iterator for Iter<'a, K> {
    type Item = &'a K;

    fn next(&mut self) -> Option<&'a K> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K> ExactSizeIterator for Iter<'a, K> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, K> FusedIterator for Iter<'a, K> {}

impl<'a, K: fmt::Debug> fmt::Debug for Iter<'a, K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<K> Iterator for IntoIter<K> {
    type Item = K;

    fn next(&mut self) -> Option<K> {
        self.iter.next().map(|(k, _)| k)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K> ExactSizeIterator for IntoIter<K> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K> FusedIterator for IntoIter<K> {}

impl<K: fmt::Debug> fmt::Debug for IntoIter<K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let entries_iter = self.iter
            .inner
            .iter()
            .map(|(k, _)| k);
        f.debug_list().entries(entries_iter).finish()
    }
}

impl<'a, K> Iterator for Drain<'a, K> {
    type Item = K;

    fn next(&mut self) -> Option<K> {
        self.iter.next().map(|(k, _)| k)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K> ExactSizeIterator for Drain<'a, K> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, K> FusedIterator for Drain<'a, K> {}

impl<'a, K: fmt::Debug> fmt::Debug for Drain<'a, K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let entries_iter = self.iter
            .inner
            .iter()
            .map(|(k, _)| k);
        f.debug_list().entries(entries_iter).finish()
    }
}

impl<'a, T, S> Clone for Intersection<'a, T, S> {
    fn clone(&self) -> Intersection<'a, T, S> {
        Intersection { iter: self.iter.clone(), ..*self }
    }
}

impl<'a, T, S> Iterator for Intersection<'a, T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        loop {
            let elt = self.iter.next()?;
            if self.other.contains(elt) {
                return Some(elt);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }
}

impl<'a, T, S> fmt::Debug for Intersection<'a, T, S>
    where T: fmt::Debug + Eq + Hash,
          S: BuildHasher
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, T, S> FusedIterator for Intersection<'a, T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
}

impl<'a, T, S> Clone for Difference<'a, T, S> {
    fn clone(&self) -> Difference<'a, T, S> {
        Difference { iter: self.iter.clone(), ..*self }
    }
}

impl<'a, T, S> Iterator for Difference<'a, T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        loop {
            let elt = self.iter.next()?;
            if !self.other.contains(elt) {
                return Some(elt);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }
}

impl<'a, T, S> FusedIterator for Difference<'a, T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
}

impl<'a, T, S> fmt::Debug for Difference<'a, T, S>
    where T: fmt::Debug + Eq + Hash,
          S: BuildHasher
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}


impl<'a, T, S> Clone for SymmetricDifference<'a, T, S> {
    fn clone(&self) -> SymmetricDifference<'a, T, S> {
        SymmetricDifference { iter: self.iter.clone() }
    }
}

impl<'a, T, S> Iterator for SymmetricDifference<'a, T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T, S> FusedIterator for SymmetricDifference<'a, T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
}

impl<'a, T, S> fmt::Debug for SymmetricDifference<'a, T, S>
    where T: fmt::Debug + Eq + Hash,
          S: BuildHasher
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, T, S> Clone for Union<'a, T, S> {
    fn clone(&self) -> Union<'a, T, S> {
        Union { iter: self.iter.clone() }
    }
}

impl<'a, T, S> FusedIterator for Union<'a, T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
}

impl<'a, T, S> fmt::Debug for Union<'a, T, S>
    where T: fmt::Debug + Eq + Hash,
          S: BuildHasher
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, T, S> Iterator for Union<'a, T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[allow(dead_code)]
fn assert_covariance() {
    fn set<'new>(v: HashSet<&'static str>) -> HashSet<&'new str> {
        v
    }
    fn iter<'a, 'new>(v: Iter<'a, &'static str>) -> Iter<'a, &'new str> {
        v
    }
    fn into_iter<'new>(v: IntoIter<&'static str>) -> IntoIter<&'new str> {
        v
    }
    fn difference<'a, 'new>(v: Difference<'a, &'static str, RandomState>)
                            -> Difference<'a, &'new str, RandomState> {
        v
    }
    fn symmetric_difference<'a, 'new>(v: SymmetricDifference<'a, &'static str, RandomState>)
                                      -> SymmetricDifference<'a, &'new str, RandomState> {
        v
    }
    fn intersection<'a, 'new>(v: Intersection<'a, &'static str, RandomState>)
                              -> Intersection<'a, &'new str, RandomState> {
        v
    }
    fn union<'a, 'new>(v: Union<'a, &'static str, RandomState>)
                       -> Union<'a, &'new str, RandomState> {
        v
    }
    fn drain<'new>(d: Drain<'static, &'static str>) -> Drain<'new, &'new str> {
        d
    }
}
