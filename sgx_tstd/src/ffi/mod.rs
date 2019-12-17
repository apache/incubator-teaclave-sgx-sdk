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

//! Utilities related to FFI bindings.
//!
//! This module provides utilities to handle data across non-Rust
//! interfaces, like other programming languages and the underlying
//! operating system. It is mainly of use for FFI (Foreign Function
//! Interface) bindings and code that needs to exchange C-like strings
//! with other languages.

pub use self::c_str::{CString, CStr, NulError, IntoStringError};
pub use self::c_str::{FromBytesWithNulError};
pub use self::os_str::{OsString, OsStr};

pub use core::ffi::c_void;
pub use core::ffi::{VaList, VaListImpl};

mod c_str;
mod os_str;