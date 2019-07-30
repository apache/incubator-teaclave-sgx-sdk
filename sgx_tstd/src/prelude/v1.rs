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

// Reexported core operators
#[doc(no_inline)] pub use crate::marker::{Copy, Send, Sized, Sync};
#[doc(no_inline)] pub use crate::ops::{Drop, Fn, FnMut, FnOnce};

// Reexported functions
#[doc(no_inline)] pub use crate::mem::drop;

// Reexported types and traits
#[doc(no_inline)] pub use crate::boxed::Box;
#[doc(no_inline)] pub use crate::borrow::ToOwned;
#[doc(no_inline)] pub use crate::clone::Clone;
#[doc(no_inline)] pub use crate::cmp::{PartialEq, PartialOrd, Eq, Ord};
#[doc(no_inline)] pub use crate::convert::{AsRef, AsMut, Into, From};
#[doc(no_inline)] pub use crate::default::Default;
#[doc(no_inline)] pub use crate::iter::{Iterator, Extend, IntoIterator};
#[doc(no_inline)] pub use crate::iter::{DoubleEndedIterator, ExactSizeIterator};
#[doc(no_inline)] pub use crate::option::Option::{self, Some, None};
#[doc(no_inline)] pub use crate::result::Result::{self, Ok, Err};
#[doc(no_inline)] pub use crate::slice::SliceConcatExt;
#[doc(no_inline)] pub use crate::string::{String, ToString};
#[doc(no_inline)] pub use crate::vec::Vec;
