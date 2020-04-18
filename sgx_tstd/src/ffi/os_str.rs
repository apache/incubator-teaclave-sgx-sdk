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

use crate::sys::os_str::{Buf, Slice};
use crate::sys_common::{AsInner, FromInner, IntoInner};
use core::ops;
use core::cmp;
use core::hash::{Hash, Hasher};
use core::fmt;
use alloc_crate::borrow::{Borrow, Cow, ToOwned};
use alloc_crate::string::String;
use alloc_crate::boxed::Box;
use alloc_crate::rc::Rc;
use alloc_crate::sync::Arc;


/// A type that can represent owned, mutable platform-native strings, but is
/// cheaply inter-convertible with Rust strings.
///
#[derive(Clone)]
pub struct OsString {
    inner: Buf,
}

/// Borrowed reference to an OS string (see [`OsString`]).
///
/// This type represents a borrowed reference to a string in the operating system's preferred
/// representation.
///
/// `&OsStr` is to [`OsString`] as [`&str`] is to [`String`]: the former in each pair are borrowed
/// references; the latter are owned strings.
///
/// See the [module's toplevel documentation about conversions][conversions] for a discussion on
/// the traits which `OsStr` implements for [conversions] from/to native representations.
///
/// [`OsString`]: struct.OsString.html
/// [`&str`]: ../primitive.str.html
/// [`String`]: ../string/struct.String.html
/// [conversions]: index.html#conversions
pub struct OsStr {
    inner: Slice,
}

impl OsString {
    /// Constructs a new empty `OsString`.
    ///
    pub fn new() -> OsString {
        OsString { inner: Buf::from_string(String::new()) }
    }

    /// Converts to an [`OsStr`] slice.
    ///
    /// [`OsStr`]: struct.OsStr.html
    ///
    pub fn as_os_str(&self) -> &OsStr {
        self
    }

    /// Converts the `OsString` into a [`String`] if it contains valid Unicode data.
    ///
    /// On failure, ownership of the original `OsString` is returned.
    ///
    /// [`String`]: ../../std/string/struct.String.html
    ///
    pub fn into_string(self) -> Result<String, OsString> {
        self.inner.into_string().map_err(|buf| OsString { inner: buf })
    }

    /// Extends the string with the given [`&OsStr`] slice.
    ///
    /// [`&OsStr`]: struct.OsStr.html
    ///
    pub fn push<T: AsRef<OsStr>>(&mut self, s: T) {
        self.inner.push_slice(&s.as_ref().inner)
    }

    /// Creates a new `OsString` with the given capacity.
    ///
    /// The string will be able to hold exactly `capacity` length units of other
    /// OS strings without reallocating. If `capacity` is 0, the string will not
    /// allocate.
    ///
    /// See main `OsString` documentation information about encoding.
    ///
    pub fn with_capacity(capacity: usize) -> OsString {
        OsString { inner: Buf::with_capacity(capacity) }
    }

    /// Truncates the `OsString` to zero length.
    ///
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// Returns the capacity this `OsString` can hold without reallocating.
    ///
    /// See `OsString` introduction for information about encoding.
    ///
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Reserves capacity for at least `additional` more capacity to be inserted
    /// in the given `OsString`.
    ///
    /// The collection may reserve more space to avoid frequent reallocations.
    ///
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional)
    }

    /// Reserves the minimum capacity for exactly `additional` more capacity to
    /// be inserted in the given `OsString`. Does nothing if the capacity is
    /// already sufficient.
    ///
    /// Note that the allocator may give the collection more space than it
    /// requests. Therefore, capacity can not be relied upon to be precisely
    /// minimal. Prefer reserve if future insertions are expected.
    ///
    pub fn reserve_exact(&mut self, additional: usize) {
        self.inner.reserve_exact(additional)
    }

    /// Shrinks the capacity of the `OsString` to match its length.
    ///
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit()
    }

    /// Shrinks the capacity of the `OsString` with a lower bound.
    ///
    /// The capacity will remain at least as large as both the length
    /// and the supplied value.
    ///
    /// Panics if the current capacity is smaller than the supplied
    /// minimum capacity.
    ///
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.inner.shrink_to(min_capacity)
    }

    /// Converts this `OsString` into a boxed [`OsStr`].
    ///
    /// [`OsStr`]: struct.OsStr.html
    ///
    pub fn into_boxed_os_str(self) -> Box<OsStr> {
        let rw = Box::into_raw(self.inner.into_box()) as *mut OsStr;
        unsafe { Box::from_raw(rw) }
    }
}

impl From<String> for OsString {
    /// Converts a [`String`] into a [`OsString`].
    ///
    /// The conversion copies the data, and includes an allocation on the heap.
    ///
    /// [`OsString`]: ../../std/ffi/struct.OsString.html
    fn from(s: String) -> OsString {
        OsString { inner: Buf::from_string(s) }
    }
}

impl<T: ?Sized + AsRef<OsStr>> From<&T> for OsString {
    fn from(s: &T) -> OsString {
        s.as_ref().to_os_string()
    }
}

impl ops::Index<ops::RangeFull> for OsString {
    type Output = OsStr;

    #[inline]
    fn index(&self, _index: ops::RangeFull) -> &OsStr {
        OsStr::from_inner(self.inner.as_slice())
    }
}

impl ops::IndexMut<ops::RangeFull> for OsString {
    #[inline]
    fn index_mut(&mut self, _index: ops::RangeFull) -> &mut OsStr {
        OsStr::from_inner_mut(self.inner.as_mut_slice())
    }
}

impl ops::Deref for OsString {
    type Target = OsStr;

    #[inline]
    fn deref(&self) -> &OsStr {
        &self[..]
    }
}

impl ops::DerefMut for OsString {
    #[inline]
    fn deref_mut(&mut self) -> &mut OsStr {
        &mut self[..]
    }
}

impl Default for OsString {
    /// Constructs an empty `OsString`.
    #[inline]
    fn default() -> OsString {
        OsString::new()
    }
}

impl fmt::Debug for OsString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, formatter)
    }
}

impl PartialEq for OsString {
    fn eq(&self, other: &OsString) -> bool {
        &**self == &**other
    }
}

impl PartialEq<str> for OsString {
    fn eq(&self, other: &str) -> bool {
        &**self == other
    }
}

impl PartialEq<OsString> for str {
    fn eq(&self, other: &OsString) -> bool {
        &**other == self
    }
}

impl PartialEq<&str> for OsString {
    fn eq(&self, other: &&str) -> bool {
        **self == **other
    }
}

impl<'a> PartialEq<OsString> for &'a str {
    fn eq(&self, other: &OsString) -> bool {
        **other == **self
    }
}

impl Eq for OsString {}

impl PartialOrd for OsString {
    #[inline]
    fn partial_cmp(&self, other: &OsString) -> Option<cmp::Ordering> {
        (&**self).partial_cmp(&**other)
    }
    #[inline]
    fn lt(&self, other: &OsString) -> bool {
        &**self < &**other
    }
    #[inline]
    fn le(&self, other: &OsString) -> bool {
        &**self <= &**other
    }
    #[inline]
    fn gt(&self, other: &OsString) -> bool {
        &**self > &**other
    }
    #[inline]
    fn ge(&self, other: &OsString) -> bool {
        &**self >= &**other
    }
}

impl PartialOrd<str> for OsString {
    #[inline]
    fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
        (&**self).partial_cmp(other)
    }
}

impl Ord for OsString {
    #[inline]
    fn cmp(&self, other: &OsString) -> cmp::Ordering {
        (&**self).cmp(&**other)
    }
}

impl Hash for OsString {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        (&**self).hash(state)
    }
}

impl OsStr {
    /// Coerces into an `OsStr` slice.
    ///
    pub fn new<S: AsRef<OsStr> + ?Sized>(s: &S) -> &OsStr {
        s.as_ref()
    }

    #[inline]
    fn from_inner(inner: &Slice) -> &OsStr {
        // Safety: OsStr is just a wrapper of Slice,
        // therefore converting &Slice to &OsStr is safe.
        unsafe { &*(inner as *const Slice as *const OsStr) }
    }

    #[inline]
    fn from_inner_mut(inner: &mut Slice) -> &mut OsStr {
        // Safety: OsStr is just a wrapper of Slice,
        // therefore converting &mut Slice to &mut OsStr is safe.
        // Any method that mutates OsStr must be careful not to
        // break platform-specific encoding, in particular Wtf8 on Windows.
        unsafe { &mut *(inner as *mut Slice as *mut OsStr) }
    }

    /// Yields a [`&str`] slice if the `OsStr` is valid Unicode.
    ///
    /// This conversion may entail doing a check for UTF-8 validity.
    ///
    /// [`&str`]: ../../std/primitive.str.html
    ///
    pub fn to_str(&self) -> Option<&str> {
        self.inner.to_str()
    }

    /// Converts an `OsStr` to a [`Cow`]`<`[`str`]`>`.
    ///
    /// Any non-Unicode sequences are replaced with
    /// [`U+FFFD REPLACEMENT CHARACTER`][U+FFFD].
    ///
    /// [`Cow`]: ../../std/borrow/enum.Cow.html
    /// [`str`]: ../../std/primitive.str.html
    /// [U+FFFD]: ../../std/char/constant.REPLACEMENT_CHARACTER.html
    ///
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        self.inner.to_string_lossy()
    }

    /// Copies the slice into an owned [`OsString`].
    ///
    /// [`OsString`]: struct.OsString.html
    ///
    pub fn to_os_string(&self) -> OsString {
        OsString { inner: self.inner.to_owned() }
    }

    /// Checks whether the `OsStr` is empty.
    ///
    pub fn is_empty(&self) -> bool {
        self.inner.inner.is_empty()
    }

    /// Returns the length of this `OsStr`.
    ///
    /// Note that this does **not** return the number of bytes in the string in
    /// OS string form.
    ///
    /// The length returned is that of the underlying storage used by `OsStr`.
    /// As discussed in the [`OsString`] introduction, [`OsString`] and `OsStr`
    /// store strings in a form best suited for cheap inter-conversion between
    /// native-platform and Rust string forms, which may differ significantly
    /// from both of them, including in storage size and encoding.
    ///
    /// This number is simply useful for passing to other methods, like
    /// [`OsString::with_capacity`] to avoid reallocations.
    ///
    /// [`OsString`]: struct.OsString.html
    /// [`OsString::with_capacity`]: struct.OsString.html#method.with_capacity
    ///
    pub fn len(&self) -> usize {
        self.inner.inner.len()
    }

    /// Converts a [`Box`]`<OsStr>` into an [`OsString`] without copying or allocating.
    ///
    /// [`Box`]: ../boxed/struct.Box.html
    /// [`OsString`]: struct.OsString.html
    pub fn into_os_string(self: Box<OsStr>) -> OsString {
        let boxed = unsafe { Box::from_raw(Box::into_raw(self) as *mut Slice) };
        OsString { inner: Buf::from_box(boxed) }
    }

    /// Gets the underlying byte representation.
    ///
    /// Note: it is *crucial* that this API is private, to avoid
    /// revealing the internal, platform-specific encodings.
    #[inline]
    fn bytes(&self) -> &[u8] {
        unsafe { &*(&self.inner as *const _ as *const [u8]) }
    }

    /// Converts this string to its ASCII lower case equivalent in-place.
    ///
    /// ASCII letters 'A' to 'Z' are mapped to 'a' to 'z',
    /// but non-ASCII letters are unchanged.
    ///
    /// To return a new lowercased value without modifying the existing one, use
    /// [`to_ascii_lowercase`].
    ///
    /// [`to_ascii_lowercase`]: #method.to_ascii_lowercase
    ///
    pub fn make_ascii_lowercase(&mut self) {
        self.inner.make_ascii_lowercase()
    }

    /// Converts this string to its ASCII upper case equivalent in-place.
    ///
    /// ASCII letters 'a' to 'z' are mapped to 'A' to 'Z',
    /// but non-ASCII letters are unchanged.
    ///
    /// To return a new uppercased value without modifying the existing one, use
    /// [`to_ascii_uppercase`].
    ///
    /// [`to_ascii_uppercase`]: #method.to_ascii_uppercase
    ///
    pub fn make_ascii_uppercase(&mut self) {
        self.inner.make_ascii_uppercase()
    }

    /// Returns a copy of this string where each character is mapped to its
    /// ASCII lower case equivalent.
    ///
    /// ASCII letters 'A' to 'Z' are mapped to 'a' to 'z',
    /// but non-ASCII letters are unchanged.
    ///
    /// To lowercase the value in-place, use [`make_ascii_lowercase`].
    ///
    /// [`make_ascii_lowercase`]: #method.make_ascii_lowercase
    ///
    pub fn to_ascii_lowercase(&self) -> OsString {
        OsString::from_inner(self.inner.to_ascii_lowercase())
    }

    /// Returns a copy of this string where each character is mapped to its
    /// ASCII upper case equivalent.
    ///
    /// ASCII letters 'a' to 'z' are mapped to 'A' to 'Z',
    /// but non-ASCII letters are unchanged.
    ///
    /// To uppercase the value in-place, use [`make_ascii_uppercase`].
    ///
    /// [`make_ascii_uppercase`]: #method.make_ascii_uppercase
    ///
    pub fn to_ascii_uppercase(&self) -> OsString {
        OsString::from_inner(self.inner.to_ascii_uppercase())
    }

    /// Checks if all characters in this string are within the ASCII range.
    ///
    pub fn is_ascii(&self) -> bool {
        self.inner.is_ascii()
    }

    /// Checks that two strings are an ASCII case-insensitive match.
    ///
    /// Same as `to_ascii_lowercase(a) == to_ascii_lowercase(b)`,
    /// but without allocating and copying temporaries.
    ///
    pub fn eq_ignore_ascii_case<S: ?Sized + AsRef<OsStr>>(&self, other: &S) -> bool {
        self.inner.eq_ignore_ascii_case(&other.as_ref().inner)
    }
}

impl From<&OsStr> for Box<OsStr> {
    fn from(s: &OsStr) -> Box<OsStr> {
        let rw = Box::into_raw(s.inner.into_box()) as *mut OsStr;
        unsafe { Box::from_raw(rw) }
    }
}

impl From<Box<OsStr>> for OsString {
    /// Converts a [`Box`]`<`[`OsStr`]`>` into a `OsString` without copying or
    /// allocating.
    ///
    /// [`Box`]: ../boxed/struct.Box.html
    /// [`OsStr`]: ../ffi/struct.OsStr.html
    fn from(boxed: Box<OsStr>) -> OsString {
        boxed.into_os_string()
    }
}

impl From<OsString> for Box<OsStr> {
    /// Converts a [`OsString`] into a [`Box`]`<OsStr>` without copying or allocating.
    ///
    /// [`Box`]: ../boxed/struct.Box.html
    /// [`OsString`]: ../ffi/struct.OsString.html
    fn from(s: OsString) -> Box<OsStr> {
        s.into_boxed_os_str()
    }
}

impl Clone for Box<OsStr> {
    #[inline]
    fn clone(&self) -> Self {
        self.to_os_string().into_boxed_os_str()
    }
}

impl From<OsString> for Arc<OsStr> {
    /// Converts a [`OsString`] into a [`Arc`]`<OsStr>` without copying or allocating.
    ///
    /// [`Arc`]: ../sync/struct.Arc.html
    /// [`OsString`]: ../ffi/struct.OsString.html
    #[inline]
    fn from(s: OsString) -> Arc<OsStr> {
        let arc = s.inner.into_arc();
        unsafe { Arc::from_raw(Arc::into_raw(arc) as *const OsStr) }
    }
}

impl From<&OsStr> for Arc<OsStr> {
    #[inline]
    fn from(s: &OsStr) -> Arc<OsStr> {
        let arc = s.inner.into_arc();
        unsafe { Arc::from_raw(Arc::into_raw(arc) as *const OsStr) }
    }
}

impl From<OsString> for Rc<OsStr> {
    /// Converts a [`OsString`] into a [`Rc`]`<OsStr>` without copying or allocating.
    ///
    /// [`Rc`]: ../rc/struct.Rc.html
    /// [`OsString`]: ../ffi/struct.OsString.html
    #[inline]
    fn from(s: OsString) -> Rc<OsStr> {
        let rc = s.inner.into_rc();
        unsafe { Rc::from_raw(Rc::into_raw(rc) as *const OsStr) }
    }
}

impl From<&OsStr> for Rc<OsStr> {
    #[inline]
    fn from(s: &OsStr) -> Rc<OsStr> {
        let rc = s.inner.into_rc();
        unsafe { Rc::from_raw(Rc::into_raw(rc) as *const OsStr) }
    }
}

impl<'a> From<OsString> for Cow<'a, OsStr> {
    #[inline]
    fn from(s: OsString) -> Cow<'a, OsStr> {
        Cow::Owned(s)
    }
}

impl<'a> From<&'a OsStr> for Cow<'a, OsStr> {
    #[inline]
    fn from(s: &'a OsStr) -> Cow<'a, OsStr> {
        Cow::Borrowed(s)
    }
}

impl<'a> From<&'a OsString> for Cow<'a, OsStr> {
    #[inline]
    fn from(s: &'a OsString) -> Cow<'a, OsStr> {
        Cow::Borrowed(s.as_os_str())
    }
}

impl<'a> From<Cow<'a, OsStr>> for OsString {
    #[inline]
    fn from(s: Cow<'a, OsStr>) -> Self {
        s.into_owned()
    }
}

impl Default for Box<OsStr> {
    fn default() -> Box<OsStr> {
        let rw = Box::into_raw(Slice::empty_box()) as *mut OsStr;
        unsafe { Box::from_raw(rw) }
    }
}

impl Default for &OsStr {
    /// Creates an empty `OsStr`.
    #[inline]
    fn default() -> Self {
        OsStr::new("")
    }
}

impl PartialEq for OsStr {
    fn eq(&self, other: &OsStr) -> bool {
        self.bytes().eq(other.bytes())
    }
}

impl PartialEq<str> for OsStr {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        *self == *OsStr::new(other)
    }
}

impl PartialEq<OsStr> for str {
    #[inline]
    fn eq(&self, other: &OsStr) -> bool {
        *other == *OsStr::new(self)
    }
}

impl Eq for OsStr {}

impl PartialOrd for OsStr {
    #[inline]
    fn partial_cmp(&self, other: &OsStr) -> Option<cmp::Ordering> {
        self.bytes().partial_cmp(other.bytes())
    }
    #[inline]
    fn lt(&self, other: &OsStr) -> bool {
        self.bytes().lt(other.bytes())
    }
    #[inline]
    fn le(&self, other: &OsStr) -> bool {
        self.bytes().le(other.bytes())
    }
    #[inline]
    fn gt(&self, other: &OsStr) -> bool {
        self.bytes().gt(other.bytes())
    }
    #[inline]
    fn ge(&self, other: &OsStr) -> bool {
        self.bytes().ge(other.bytes())
    }
}

impl PartialOrd<str> for OsStr {
    #[inline]
    fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
        self.partial_cmp(OsStr::new(other))
    }
}

// FIXME (#19470): cannot provide PartialOrd<OsStr> for str until we
// have more flexible coherence rules.

impl Ord for OsStr {
    #[inline]
    fn cmp(&self, other: &OsStr) -> cmp::Ordering {
        self.bytes().cmp(other.bytes())
    }
}

macro_rules! impl_cmp {
    ($lhs:ty, $rhs: ty) => {
        impl<'a, 'b> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                <OsStr as PartialEq>::eq(self, other)
            }
        }

        impl<'a, 'b> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                <OsStr as PartialEq>::eq(self, other)
            }
        }

        impl<'a, 'b> PartialOrd<$rhs> for $lhs {
            #[inline]
            fn partial_cmp(&self, other: &$rhs) -> Option<cmp::Ordering> {
                <OsStr as PartialOrd>::partial_cmp(self, other)
            }
        }

        impl<'a, 'b> PartialOrd<$lhs> for $rhs {
            #[inline]
            fn partial_cmp(&self, other: &$lhs) -> Option<cmp::Ordering> {
                <OsStr as PartialOrd>::partial_cmp(self, other)
            }
        }
    };
}

impl_cmp!(OsString, OsStr);
impl_cmp!(OsString, &'a OsStr);
impl_cmp!(Cow<'a, OsStr>, OsStr);
impl_cmp!(Cow<'a, OsStr>, &'b OsStr);
impl_cmp!(Cow<'a, OsStr>, OsString);

impl Hash for OsStr {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes().hash(state)
    }
}

impl fmt::Debug for OsStr {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, formatter)
    }
}

impl OsStr {
    pub(crate) fn display(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, formatter)
    }
}

impl Borrow<OsStr> for OsString {
    fn borrow(&self) -> &OsStr {
        &self[..]
    }
}

impl ToOwned for OsStr {
    type Owned = OsString;
    fn to_owned(&self) -> OsString {
        self.to_os_string()
    }
    fn clone_into(&self, target: &mut OsString) {
        self.inner.clone_into(&mut target.inner)
    }
}

impl AsRef<OsStr> for OsStr {
    fn as_ref(&self) -> &OsStr {
        self
    }
}

impl AsRef<OsStr> for OsString {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self
    }
}

impl AsRef<OsStr> for str {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        OsStr::from_inner(Slice::from_str(self))
    }
}

impl AsRef<OsStr> for String {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        (&**self).as_ref()
    }
}

impl FromInner<Buf> for OsString {
    fn from_inner(buf: Buf) -> OsString {
        OsString { inner: buf }
    }
}

impl IntoInner<Buf> for OsString {
    fn into_inner(self) -> Buf {
        self.inner
    }
}

impl AsInner<Slice> for OsStr {
    #[inline]
    fn as_inner(&self) -> &Slice {
        &self.inner
    }
}