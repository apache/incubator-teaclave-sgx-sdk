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

//! The first version of the prelude of The Rust Standard Library.
//!
//! See the [module-level documentation](super) for more.

// Re-exported core operators
#[doc(no_inline)]
pub use crate::marker::{Send, Sized, Sync, Unpin};
#[doc(no_inline)]
pub use crate::ops::{Drop, Fn, FnMut, FnOnce};

// Re-exported functions
#[doc(no_inline)]
pub use crate::mem::drop;

// Re-exported types and traits
#[doc(no_inline)]
pub use crate::convert::{AsMut, AsRef, From, Into};
#[doc(no_inline)]
pub use crate::iter::{DoubleEndedIterator, ExactSizeIterator};
#[doc(no_inline)]
pub use crate::iter::{Extend, IntoIterator, Iterator};
#[doc(no_inline)]
pub use crate::option::Option::{self, None, Some};
#[doc(no_inline)]
pub use crate::result::Result::{self, Err, Ok};

// Re-exported built-in macros
#[allow(deprecated)]
#[doc(no_inline)]
pub use core::prelude::v1::{
    assert, cfg, column, compile_error, concat, concat_idents, env, file, format_args,
    format_args_nl, include, include_bytes, include_str, line, llvm_asm, log_syntax, module_path,
    option_env, stringify, trace_macros, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq,
    PartialOrd,
};

#[doc(no_inline)]
pub use core::prelude::v1::asm;

#[doc(no_inline)]
pub use core::prelude::v1::global_asm;

// FIXME: Attribute and internal derive macros are not documented because for them rustdoc generates
// dead links which fail link checker testing.
#[allow(deprecated, deprecated_in_future)]
#[doc(hidden)]
pub use core::prelude::v1::{
    global_allocator, test, test_case, RustcDecodable, RustcEncodable,
};

#[doc(hidden)]
pub use core::prelude::v1::derive;
#[doc(hidden)]
pub use core::prelude::v1::cfg_accessible;

#[doc(hidden)]
pub use core::prelude::v1::cfg_eval;

// The file so far is equivalent to src/libcore/prelude/v1.rs,
// and below to src/liballoc/prelude.rs.
// Those files are duplicated rather than using glob imports
// because we want docs to show these re-exports as pointing to within `std`.

#[doc(no_inline)]
pub use crate::borrow::ToOwned;
#[doc(no_inline)]
pub use crate::boxed::Box;
#[doc(no_inline)]
pub use crate::string::{String, ToString};
#[doc(no_inline)]
pub use crate::vec::Vec;
