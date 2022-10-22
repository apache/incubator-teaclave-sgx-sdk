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

//! This module contains the implementation of the `eh_personality` lang item.
//!
//! The actual implementation is heavily dependent on the target since Rust
//! tries to use the native stack unwinding mechanism whenever possible.
//!
//! This personality function is still required with `-C panic=abort` because
//! it is used to catch foreign exceptions from `extern "C-unwind"` and turn
//! them into aborts.
//!
//! Additionally, ARM EHABI uses the personality function when generating
//! backtraces.

mod dwarf;
mod gcc;
