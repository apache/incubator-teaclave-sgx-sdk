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

/// A type representing an owned, C-compatible, nul-terminated string with no nul bytes in the
/// middle.
///
/// This type serves the purpose of being able to safely generate a
/// C-compatible string from a Rust byte slice or vector. An instance of this
/// type is a static guarantee that the underlying bytes contain no interior 0
/// bytes ("nul characters") and that the final byte is 0 ("nul terminator").
///
use sgx_types::c_char;
use libc;
use memchr;
use ascii;
//use error::Error;
//use io;

use core::ops;
use core::cmp::Ordering;
use core::mem;
use core::ptr;
use core::fmt::{self, Write};
use alloc::boxed::Box;
use alloc::borrow::{Cow, Borrow, ToOwned};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::slice;
use alloc::rc::Rc;
use alloc::arc::Arc;

use alloc::str::{self, Utf8Error};

/// A type representing an owned C-compatible string
///
#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Clone)]
pub struct CString {
    inner: Box<[u8]>,
}

/// Representation of a borrowed C string.
///
#[derive(Hash)]
pub struct CStr {
    inner: [c_char]
}

/// An error returned from `CString::new` to indicate that a nul byte was found
/// in the vector provided.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NulError(usize, Vec<u8>);

/// An error returned from `CStr::from_bytes_with_nul` to indicate that a nul
/// byte was found too early in the slice provided or one wasn't found at all.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FromBytesWithNulError {
    kind: FromBytesWithNulErrorKind,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum FromBytesWithNulErrorKind {
    InteriorNul(usize),
    NotNulTerminated,
}

impl FromBytesWithNulError {
    fn interior_nul(pos: usize) -> FromBytesWithNulError {
        FromBytesWithNulError {
            kind: FromBytesWithNulErrorKind::InteriorNul(pos),
        }
    }
    fn not_nul_terminated() -> FromBytesWithNulError {
        FromBytesWithNulError {
            kind: FromBytesWithNulErrorKind::NotNulTerminated,
        }
    }

    pub fn __description(&self) -> &str {
        match self.kind {
            FromBytesWithNulErrorKind::InteriorNul(..) =>
                "data provided contains an interior nul byte",
            FromBytesWithNulErrorKind::NotNulTerminated =>
                "data provided is not nul terminated",
        }
    }
}

/// An error returned from `CString::into_string` to indicate that a UTF-8 error
/// was encountered during the conversion.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct IntoStringError {
    inner: CString,
    error: Utf8Error,
}

impl CString {
    /// Creates a new C-compatible string from a container of bytes.
    ///
    /// This method will consume the provided data and use the underlying bytes
    /// to construct a new string, ensuring that there is a trailing 0 byte.
    ///
    /// # Errors
    ///
    /// This function will return an error if the bytes yielded contain an
    /// internal 0 byte. The error returned will contain the bytes as well as
    /// the position of the nul byte.
    pub fn new<T: Into<Vec<u8>>>(t: T) -> Result<CString, NulError> {
        Self::_new(t.into())
    }

    fn _new(bytes: Vec<u8>) -> Result<CString, NulError> {
        match memchr::memchr(0, &bytes) {
            Some(i) => Err(NulError(i, bytes)),
            None => Ok(unsafe { CString::from_vec_unchecked(bytes) }),
        }
    }

    /// Creates a C-compatible string from a byte vector without checking for
    /// interior 0 bytes.
    ///
    /// This method is equivalent to `new` except that no runtime assertion
    /// is made that `v` contains no 0 bytes, and it requires an actual
    /// byte vector, not anything that can be converted to one with Into.
    ///
    pub unsafe fn from_vec_unchecked(mut v: Vec<u8>) -> CString {
        v.reserve_exact(1);
        v.push(0);
        CString { inner: v.into_boxed_slice() }
    }

    /// Retakes ownership of a `CString` that was transferred to C.
    ///
    /// Additionally, the length of the string will be recalculated from the pointer.
    ///
    /// # Safety
    ///
    /// This should only ever be called with a pointer that was earlier
    /// obtained by calling `into_raw` on a `CString`. Other usage (e.g. trying to take
    /// ownership of a string that was allocated by foreign code) is likely to lead
    /// to undefined behavior or allocator corruption.
    pub unsafe fn from_raw(ptr: *mut c_char) -> CString {
        let len = libc::strlen(ptr) + 1; // Including the NUL byte
        let slice = slice::from_raw_parts_mut(ptr, len as usize);
        CString { inner: Box::from_raw(slice as *mut [c_char] as *mut [u8]) }
    }

    /// Transfers ownership of the string to a C caller.
    ///
    /// The pointer must be returned to Rust and reconstituted using
    /// `from_raw` to be properly deallocated. Specifically, one
    /// should *not* use the standard C `free` function to deallocate
    /// this string.
    ///
    /// Failure to call `from_raw` will lead to a memory leak.
    pub fn into_raw(self) -> *mut c_char {
        Box::into_raw(self.into_inner()) as *mut c_char
    }

    /// Converts the `CString` into a `String` if it contains valid Unicode data.
    ///
    /// On failure, ownership of the original `CString` is returned.
    pub fn into_string(self) -> Result<String, IntoStringError> {
        String::from_utf8(self.into_bytes())
            .map_err(|e| IntoStringError {
                error: e.utf8_error(),
                inner: unsafe { CString::from_vec_unchecked(e.into_bytes()) },
            })
    }

    /// Returns the underlying byte buffer.
    ///
    /// The returned buffer does **not** contain the trailing nul separator and
    /// it is guaranteed to not have any interior nul bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        let mut vec = self.into_inner().into_vec();
        let _nul = vec.pop();
        debug_assert_eq!(_nul, Some(0u8));
        vec
    }

    /// Equivalent to the `into_bytes` function except that the returned vector
    /// includes the trailing nul byte.
    pub fn into_bytes_with_nul(self) -> Vec<u8> {
        self.into_inner().into_vec()
    }

    /// Returns the contents of this `CString` as a slice of bytes.
    ///
    /// The returned slice does **not** contain the trailing nul separator and
    /// it is guaranteed to not have any interior nul bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.inner[..self.inner.len() - 1]
    }

    /// Equivalent to the `as_bytes` function except that the returned slice
    /// includes the trailing nul byte.
    pub fn as_bytes_with_nul(&self) -> &[u8] {
        &self.inner
    }

    /// Extracts a `CStr` slice containing the entire string.
    pub fn as_c_str(&self) -> &CStr {
        &*self
    }

    /// Converts this `CString` into a boxed `CStr`.
    pub fn into_boxed_c_str(self) -> Box<CStr> {
        unsafe { Box::from_raw(Box::into_raw(self.into_inner()) as *mut CStr) }
    }

    // Bypass "move out of struct which implements `Drop` trait" restriction.
    fn into_inner(self) -> Box<[u8]> {
        unsafe {
            let result = ptr::read(&self.inner);
            mem::forget(self);
            result
        }
    }
}

// Turns this `CString` into an empty string to prevent
// memory unsafe code from working by accident. Inline
// to prevent LLVM from optimizing it away in debug builds.
impl Drop for CString {
    #[inline]
    fn drop(&mut self) {
        unsafe { *self.inner.get_unchecked_mut(0) = 0; }
    }
}

impl ops::Deref for CString {
    type Target = CStr;

    #[inline]
    fn deref(&self) -> &CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(self.as_bytes_with_nul()) }
    }
}

impl fmt::Debug for CString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl From<CString> for Vec<u8> {
    #[inline]
    fn from(s: CString) -> Vec<u8> {
        s.into_bytes()
    }
}

impl fmt::Debug for CStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\"")?;
        for byte in self.to_bytes().iter().flat_map(|&b| ascii::escape_default(b)) {
            f.write_char(byte as char)?;
        }
        write!(f, "\"")
    }
}

impl<'a> Default for &'a CStr {
    fn default() -> &'a CStr {
        const SLICE: &'static [c_char] = &[0];
        unsafe { CStr::from_ptr(SLICE.as_ptr()) }
    }
}

impl Default for CString {
    /// Creates an empty `CString`.
    fn default() -> CString {
        let a: &CStr = Default::default();
        a.to_owned()
    }
}

impl Borrow<CStr> for CString {
    #[inline]
    fn borrow(&self) -> &CStr { self }
}

impl<'a> From<&'a CStr> for Box<CStr> {
    fn from(s: &'a CStr) -> Box<CStr> {
        let boxed: Box<[u8]> = Box::from(s.to_bytes_with_nul());
        unsafe { Box::from_raw(Box::into_raw(boxed) as *mut CStr) }
    }
}

impl From<Box<CStr>> for CString {
    #[inline]
    fn from(s: Box<CStr>) -> CString {
        s.into_c_string()
    }
}

impl From<CString> for Box<CStr> {
    #[inline]
    fn from(s: CString) -> Box<CStr> {
        s.into_boxed_c_str()
    }
}

impl From<CString> for Arc<CStr> {
    #[inline]
    fn from(s: CString) -> Arc<CStr> {
        let arc: Arc<[u8]> = Arc::from(s.into_inner());
        unsafe { Arc::from_raw(Arc::into_raw(arc) as *const CStr) }
    }
}

impl<'a> From<&'a CStr> for Arc<CStr> {
    #[inline]
    fn from(s: &CStr) -> Arc<CStr> {
        let arc: Arc<[u8]> = Arc::from(s.to_bytes_with_nul());
        unsafe { Arc::from_raw(Arc::into_raw(arc) as *const CStr) }
    }
}

impl From<CString> for Rc<CStr> {
    #[inline]
    fn from(s: CString) -> Rc<CStr> {
        let rc: Rc<[u8]> = Rc::from(s.into_inner());
        unsafe { Rc::from_raw(Rc::into_raw(rc) as *const CStr) }
    }
}

impl<'a> From<&'a CStr> for Rc<CStr> {
    #[inline]
    fn from(s: &CStr) -> Rc<CStr> {
        let rc: Rc<[u8]> = Rc::from(s.to_bytes_with_nul());
        unsafe { Rc::from_raw(Rc::into_raw(rc) as *const CStr) }
    }
}

impl Default for Box<CStr> {
    fn default() -> Box<CStr> {
        let boxed: Box<[u8]> = Box::from([0]);
        unsafe { Box::from_raw(Box::into_raw(boxed) as *mut CStr) }
    }
}

impl NulError {
    /// Returns the position of the nul byte in the slice that was provided to
    /// `CString::new`.
    ///
    pub fn nul_position(&self) -> usize { self.0 }

    /// Consumes this error, returning the underlying vector of bytes which
    /// generated the error in the first place.
    ///
    pub fn into_vec(self) -> Vec<u8> { self.1 }
}

impl fmt::Display for NulError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "nul byte found in provided data at position: {}", self.0)
    }
}

impl fmt::Display for FromBytesWithNulError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.__description())?;
        if let FromBytesWithNulErrorKind::InteriorNul(pos) = self.kind {
            write!(f, " at byte pos {}", pos)?;
        }
        Ok(())
    }
}

impl IntoStringError {
    /// Consumes this error, returning original `CString` which generated the
    /// error.
    pub fn into_cstring(self) -> CString {
        self.inner
    }

    /// Access the underlying UTF-8 error that was the cause of this error.
    pub fn utf8_error(&self) -> Utf8Error {
        self.error
    }

    pub fn __description(&self) -> &str {
        "C string contained non-utf8 bytes"
    }

    pub fn __cause(&self) -> Option<&Utf8Error> {
        Some(&self.error)
    }
}

impl fmt::Display for IntoStringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.__description().fmt(f)
    }
}

impl CStr {
    /// Wraps a raw C string with a safe C string wrapper.
    ///
    /// This function will wrap the provided `ptr` with a `CStr` wrapper, which
    /// allows inspection and interoperation of non-owned C strings. This method
    /// is unsafe for a number of reasons:
    ///
    /// * There is no guarantee to the validity of `ptr`.
    /// * The returned lifetime is not guaranteed to be the actual lifetime of
    ///   `ptr`.
    /// * There is no guarantee that the memory pointed to by `ptr` contains a
    ///   valid nul terminator byte at the end of the string.
    /// * It is not guaranteed that the memory pointed by `ptr` won't change
    ///   before the `CStr` has been destroyed.
    ///
    /// > **Note**: This operation is intended to be a 0-cost cast but it is
    /// > currently implemented with an up-front calculation of the length of
    /// > the string. This is not guaranteed to always be the case.
    ///
    pub unsafe fn from_ptr<'a>(ptr: *const c_char) -> &'a CStr {
        let len = libc::strlen(ptr);
        let ptr = ptr as *const u8;
        CStr::from_bytes_with_nul_unchecked(slice::from_raw_parts(ptr, len as usize + 1))
    }

    /// Creates a C string wrapper from a byte slice.
    ///
    /// This function will cast the provided `bytes` to a `CStr`
    /// wrapper after ensuring that the byte slice is nul-terminated
    /// and does not contain any interior nul bytes.
    ///
    pub fn from_bytes_with_nul(bytes: &[u8])
                               -> Result<&CStr, FromBytesWithNulError> {
        let nul_pos = memchr::memchr(0, bytes);
        if let Some(nul_pos) = nul_pos {
            if nul_pos + 1 != bytes.len() {
                return Err(FromBytesWithNulError::interior_nul(nul_pos));
            }
            Ok(unsafe { CStr::from_bytes_with_nul_unchecked(bytes) })
        } else {
            Err(FromBytesWithNulError::not_nul_terminated())
        }
    }

    /// Unsafely creates a C string wrapper from a byte slice.
    ///
    /// This function will cast the provided `bytes` to a `CStr` wrapper without
    /// performing any sanity checks. The provided slice **must** be nul-terminated
    /// and not contain any interior nul bytes.
    ///
    pub unsafe fn from_bytes_with_nul_unchecked(bytes: &[u8]) -> &CStr {
        &*(bytes as *const [u8] as *const CStr)
    }

    /// Returns the inner pointer to this C string.
    ///
    /// The returned pointer will be valid for as long as `self` is, and points
    /// to a contiguous region of memory terminated with a 0 byte to represent
    /// the end of the string.
    ///
    pub fn as_ptr(&self) -> *const c_char {
        self.inner.as_ptr()
    }

    /// Converts this C string to a byte slice.
    ///
    /// The returned slice will **not** contain the trailing nul terminator that this C
    /// string has.
    ///
    /// > **Note**: This method is currently implemented as a constant-time
    /// > cast, but it is planned to alter its definition in the future to
    /// > perform the length calculation whenever this method is called.
    ///
    pub fn to_bytes(&self) -> &[u8] {
        let bytes = self.to_bytes_with_nul();
        &bytes[..bytes.len() - 1]
    }

    /// Converts this C string to a byte slice containing the trailing 0 byte.
    ///
    /// This function is the equivalent of [`to_bytes`] except that it will retain
    /// the trailing nul terminator instead of chopping it off.
    ///
    /// > **Note**: This method is currently implemented as a 0-cost cast, but
    /// > it is planned to alter its definition in the future to perform the
    /// > length calculation whenever this method is called.
    ///
    /// [`to_bytes`]: #method.to_bytes
    ///
    pub fn to_bytes_with_nul(&self) -> &[u8] {
        unsafe { &*(&self.inner as *const [c_char] as *const [u8]) }
    }

    /// Yields a [`&str`] slice if the `CStr` contains valid UTF-8.
    ///
    /// If the contents of the `CStr` are valid UTF-8 data, this
    /// function will return the corresponding [`&str`] slice. Otherwise,
    /// it will return an error with details of where UTF-8 validation failed.
    ///
    /// > **Note**: This method is currently implemented to check for validity
    /// > after a constant-time cast, but it is planned to alter its definition
    /// > in the future to perform the length calculation in addition to the
    /// > UTF-8 check whenever this method is called.
    ///
    /// [`&str`]: ../primitive.str.html
    ///
    pub fn to_str(&self) -> Result<&str, str::Utf8Error> {
        // NB: When CStr is changed to perform the length check in .to_bytes()
        // instead of in from_ptr(), it may be worth considering if this should
        // be rewritten to do the UTF-8 check inline with the length calculation
        // instead of doing it afterwards.
        str::from_utf8(self.to_bytes())
    }

    /// Converts a `CStr` into a [`Cow`]`<`[`str`]`>`.
    ///
    /// If the contents of the `CStr` are valid UTF-8 data, this
    /// function will return a [`Cow`]`::`[`Borrowed`]`(`[`&str`]`)`
    /// with the the corresponding [`&str`] slice. Otherwise, it will
    /// replace any invalid UTF-8 sequences with `U+FFFD REPLACEMENT
    /// CHARACTER` and return a [`Cow`]`::`[`Owned`]`(`[`String`]`)`
    /// with the result.
    ///
    /// > **Note**: This method is currently implemented to check for validity
    /// > after a constant-time cast, but it is planned to alter its definition
    /// > in the future to perform the length calculation in addition to the
    /// > UTF-8 check whenever this method is called.
    ///
    /// [`Cow`]: ../borrow/enum.Cow.html
    /// [`Borrowed`]: ../borrow/enum.Cow.html#variant.Borrowed
    /// [`str`]: ../primitive.str.html
    /// [`String`]: ../string/struct.String.html
    ///
    pub fn to_string_lossy(&self) -> Cow<str> {
        String::from_utf8_lossy(self.to_bytes())
    }

    /// Converts a [`Box`]`<CStr>` into a [`CString`] without copying or allocating.
    ///
    /// [`Box`]: ../boxed/struct.Box.html
    /// [`CString`]: struct.CString.html
    ///
    pub fn into_c_string(self: Box<CStr>) -> CString {
        let raw = Box::into_raw(self) as *mut [u8];
        CString { inner: unsafe { Box::from_raw(raw) } }
    }
}

impl PartialEq for CStr {
    fn eq(&self, other: &CStr) -> bool {
        self.to_bytes().eq(other.to_bytes())
    }
}

impl Eq for CStr {}

impl PartialOrd for CStr {
    fn partial_cmp(&self, other: &CStr) -> Option<Ordering> {
        self.to_bytes().partial_cmp(&other.to_bytes())
    }
}

impl Ord for CStr {
    fn cmp(&self, other: &CStr) -> Ordering {
        self.to_bytes().cmp(&other.to_bytes())
    }
}

impl ToOwned for CStr {
    type Owned = CString;

    fn to_owned(&self) -> CString {
        CString { inner: self.to_bytes_with_nul().into() }
    }
}

impl<'a> From<&'a CStr> for CString {
    fn from(s: &'a CStr) -> CString {
        s.to_owned()
    }
}

impl ops::Index<ops::RangeFull> for CString {
    type Output = CStr;

    #[inline]
    fn index(&self, _index: ops::RangeFull) -> &CStr {
        self
    }
}

impl AsRef<CStr> for CStr {
    #[inline]
    fn as_ref(&self) -> &CStr {
        self
    }
}

impl AsRef<CStr> for CString {
    #[inline]
    fn as_ref(&self) -> &CStr {
        self
    }
}

