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

use core::any::TypeId;
use core::cell;
use core::mem::transmute;
use core::num;
use core::array;
use core::fmt::{self, Debug, Display};
use alloc_crate::str;
use alloc_crate::string::{self, String};
use alloc_crate::boxed::Box;
use alloc_crate::borrow::Cow;
use alloc_crate::alloc;
use core::char;

/// Base functionality for all errors in Rust.
pub trait Error: Debug + Display {
    /// A short description of the error.
    #[deprecated]
    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    /// The lower-level cause of this error, if any.
    fn cause(&self) -> Option<&Error> { None }

    /// Get the `TypeId` of `self`
    /// Pending CVE number here
    /// https://github.com/rust-lang/rust/issues/60784
    /// Alex said:
    /// This leaves us, however, with the question of what to do about this
    /// API? Error::type_id has been present since the inception of the Error
    /// trait, all the way back to 1.0.0. It's unstable, however, and is pretty
    /// rare as well to have a manual implementation of the type_id function.
    /// Despite this we would ideally still like a path to stability which
    /// includes safety at some point.
    ///
    /// This tracking issue is intended to serve as a location to discuss this
    /// issue and determine the best way forward to fully removing Error::type_id
    /// (so even nightly users are not affected by this memory safety issue) and
    /// having a stable mechanism for the functionality.
    ///
    /// Yu: Mark as deprecated for compile-time warning
    #[deprecated]
    fn type_id(&self) -> TypeId where Self: 'static {
        TypeId::of::<Self>()
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> { None }
}

impl<'a, E: Error + 'a> From<E> for Box<Error + 'a> {
    fn from(err: E) -> Box<Error + 'a> {
        Box::new(err)
    }
}

impl<'a, E: Error + Send + Sync + 'a> From<E> for Box<Error + Send + Sync + 'a> {
    fn from(err: E) -> Box<Error + Send + Sync + 'a> {
        Box::new(err)
    }
}

impl From<String> for Box<Error + Send + Sync> {
    fn from(err: String) -> Box<Error + Send + Sync> {
        #[derive(Debug)]
        struct StringError(String);

        impl Error for StringError {
            fn description(&self) -> &str { &self.0 }
        }

        impl Display for StringError {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                Display::fmt(&self.0, f)
            }
        }

        Box::new(StringError(err))
    }
}

impl From<String> for Box<Error> {
    fn from(str_err: String) -> Box<Error> {
        let err1: Box<Error + Send + Sync> = From::from(str_err);
        let err2: Box<Error> = err1;
        err2
    }
}

impl<'a, 'b> From<&'b str> for Box<Error + Send + Sync + 'a> {
    fn from(err: &'b str) -> Box<Error + Send + Sync + 'a> {
        From::from(String::from(err))
    }
}

impl<'a> From<&'a str> for Box<Error> {
    fn from(err: &'a str) -> Box<Error> {
        From::from(String::from(err))
    }
}

impl<'a, 'b> From<Cow<'b, str>> for Box<Error + Send + Sync + 'a> {
    fn from(err: Cow<'b, str>) -> Box<Error + Send + Sync + 'a> {
        From::from(String::from(err))
    }
}

impl<'a> From<Cow<'a, str>> for Box<Error> {
    fn from(err: Cow<'a, str>) -> Box<Error> {
        From::from(String::from(err))
    }
}

impl Error for ! {
    fn description(&self) -> &str { *self }
}

impl Error for alloc::AllocErr {
    fn description(&self) -> &str {
        "memory allocation failed"
    }
}

impl Error for alloc::CannotReallocInPlace {
    fn description(&self) -> &str {
        alloc::CannotReallocInPlace::description(self)
    }
}

impl Error for str::ParseBoolError {
    fn description(&self) -> &str { "failed to parse bool" }
}

impl Error for str::Utf8Error {
    fn description(&self) -> &str {
        "invalid utf-8: corrupt contents"
    }
}

impl Error for num::ParseIntError {
    fn description(&self) -> &str {
        self.__description()
    }
}

impl Error for num::TryFromIntError {
    fn description(&self) -> &str {
        self.__description()
    }
}

impl Error for array::TryFromSliceError {
    fn description(&self) -> &str {
        self.__description()
    }
}

impl Error for num::ParseFloatError {
    fn description(&self) -> &str {
        self.__description()
    }
}

impl Error for string::FromUtf8Error {
    fn description(&self) -> &str {
        "invalid utf-8"
    }
}

impl Error for string::FromUtf16Error {
    fn description(&self) -> &str {
        "invalid utf-16"
    }
}

impl Error for string::ParseError {
    fn description(&self) -> &str {
        match *self {}
    }
}

impl Error for char::DecodeUtf16Error {
    fn description(&self) -> &str {
        "unpaired surrogate found"
    }
}

impl<T: Error> Error for Box<T> {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        Error::description(&**self)
    }

    fn cause(&self) -> Option<&Error> {
        Error::cause(&**self)
    }
}

impl Error for fmt::Error {
    fn description(&self) -> &str {
        "an error occurred when formatting an argument"
    }
}

impl Error for cell::BorrowError {
    fn description(&self) -> &str {
        "already mutably borrowed"
    }
}

impl Error for cell::BorrowMutError {
    fn description(&self) -> &str {
        "already borrowed"
    }
}

impl Error for char::CharTryFromError {
    fn description(&self) -> &str {
        "converted integer out of range for `char`"
    }
}

impl Error for char::ParseCharError {
    fn description(&self) -> &str {
        self.__description()
    }
}

// copied from any.rs
impl Error + 'static {
    /// Returns true if the boxed type is the same as `T`
    #[inline]
    pub fn is<T: Error + 'static>(&self) -> bool {
        // Get TypeId of the type this function is instantiated with
        let t = TypeId::of::<T>();

        // Get TypeId of the type in the trait object
        // Allow type_id here because the impl here is safe
        #[allow(deprecated)]
        let boxed = self.type_id();

        // Compare both TypeIds on equality
        t == boxed
    }

    /// Returns some reference to the boxed value if it is of type `T`, or
    /// `None` if it isn't.
    #[inline]
    pub fn downcast_ref<T: Error + 'static>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe {
                Some(&*(self as *const Error as *const T))
            }
        } else {
            None
        }
    }

    /// Returns some mutable reference to the boxed value if it is of type `T`, or
    /// `None` if it isn't.
    #[inline]
    pub fn downcast_mut<T: Error + 'static>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe {
                Some(&mut *(self as *mut Error as *mut T))
            }
        } else {
            None
        }
    }
}

impl Error + 'static + Send {
    /// Forwards to the method defined on the type `Any`.
    #[inline]
    pub fn is<T: Error + 'static>(&self) -> bool {
        <Error + 'static>::is::<T>(self)
    }

    /// Forwards to the method defined on the type `Any`.
    #[inline]
    pub fn downcast_ref<T: Error + 'static>(&self) -> Option<&T> {
        <Error + 'static>::downcast_ref::<T>(self)
    }

    /// Forwards to the method defined on the type `Any`.
    #[inline]
    pub fn downcast_mut<T: Error + 'static>(&mut self) -> Option<&mut T> {
        <Error + 'static>::downcast_mut::<T>(self)
    }
}

impl Error + 'static + Send + Sync {
    /// Forwards to the method defined on the type `Any`.
    #[inline]
    pub fn is<T: Error + 'static>(&self) -> bool {
        <Error + 'static>::is::<T>(self)
    }

    /// Forwards to the method defined on the type `Any`.
    #[inline]
    pub fn downcast_ref<T: Error + 'static>(&self) -> Option<&T> {
        <Error + 'static>::downcast_ref::<T>(self)
    }

    /// Forwards to the method defined on the type `Any`.
    #[inline]
    pub fn downcast_mut<T: Error + 'static>(&mut self) -> Option<&mut T> {
        <Error + 'static>::downcast_mut::<T>(self)
    }
}

impl Error {
    #[inline]
    /// Attempt to downcast the box to a concrete type.
    pub fn downcast<T: Error + 'static>(self: Box<Self>) -> Result<Box<T>, Box<Error>> {
        if self.is::<T>() {
            unsafe {
                let raw: *mut Error = Box::into_raw(self);
                Ok(Box::from_raw(raw as *mut T))
            }
        } else {
            Err(self)
        }
    }
}

impl Error + Send {
    #[inline]
    /// Attempt to downcast the box to a concrete type.
    pub fn downcast<T: Error + 'static>(self: Box<Self>)
                                        -> Result<Box<T>, Box<Error + Send>> {
        let err: Box<Error> = self;
        <Error>::downcast(err).map_err(|s| unsafe {
            // reapply the Send marker
            transmute::<Box<Error>, Box<Error + Send>>(s)
        })
    }
}

impl Error + Send + Sync {
    #[inline]
    /// Attempt to downcast the box to a concrete type.
    pub fn downcast<T: Error + 'static>(self: Box<Self>)
                                        -> Result<Box<T>, Box<Self>> {
        let err: Box<Error> = self;
        <Error>::downcast(err).map_err(|s| unsafe {
            // reapply the Send+Sync marker
            transmute::<Box<Error>, Box<Error + Send + Sync>>(s)
        })
    }
}
