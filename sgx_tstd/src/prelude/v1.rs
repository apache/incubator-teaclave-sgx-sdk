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

#[doc(no_inline)]
pub use crate::marker::{Send, Sized, Sync, Unpin};
#[doc(no_inline)]
pub use crate::ops::{Drop, Fn, FnMut, FnOnce};

// Re-exported functions
#[doc(no_inline)]
pub use crate::mem::drop;

// Re-exported types and traits
#[doc(no_inline)]
pub use crate::convert::{AsRef, AsMut, Into, From};
#[doc(no_inline)]
pub use crate::iter::{Iterator, Extend, IntoIterator};
#[doc(no_inline)]
pub use crate::iter::{DoubleEndedIterator, ExactSizeIterator};
#[doc(no_inline)]
pub use crate::option::Option::{self, Some, None};
#[doc(no_inline)]
pub use crate::result::Result::{self, Ok, Err};

#[doc(no_inline)]
pub use core::prelude::v1::{
    asm,
    assert,
    cfg,
    column,
    compile_error,
    concat,
    concat_idents,
    env,
    file,
    format_args,
    format_args_nl,
    global_asm,
    include,
    include_bytes,
    include_str,
    line,
    log_syntax,
    module_path,
    option_env,
    stringify,
    trace_macros,
};

// FIXME: Attribute and derive macros are not documented because for them rustdoc generates
// dead links which fail link checker testing.
#[allow(deprecated)]
#[doc(hidden)]
pub use core::prelude::v1::{
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    RustcDecodable,
    RustcEncodable,
//    bench,
    global_allocator,
    test,
    test_case,
};

// The file so far is equivalent to src/libcore/prelude/v1.rs,
// and below to src/liballoc/prelude.rs.
// Those files are duplicated rather than using glob imports
// because we want docs to show these re-exports as pointing to within `std`.


#[doc(no_inline)]
pub use crate::boxed::Box;
#[doc(no_inline)]
pub use crate::borrow::ToOwned;
#[doc(no_inline)]
pub use crate::string::{String, ToString};
#[doc(no_inline)]
pub use crate::vec::Vec;
