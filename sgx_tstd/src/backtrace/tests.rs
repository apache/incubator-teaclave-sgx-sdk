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
// under the License.

use super::*;
use crate::panic::{RefUnwindSafe, UnwindSafe};

use sgx_test_utils::test_case;

fn generate_fake_frames() -> Vec<BacktraceFrame> {
    vec![
        BacktraceFrame {
            frame: RawFrame::Fake,
            symbols: vec![BacktraceSymbol {
                name: Some(b"std::backtrace::Backtrace::create".to_vec()),
                filename: Some(BytesOrWide::Bytes(b"rust/backtrace.rs".to_vec())),
                lineno: Some(100),
                colno: None,
            }],
        },
        BacktraceFrame {
            frame: RawFrame::Fake,
            symbols: vec![BacktraceSymbol {
                name: Some(b"__rust_maybe_catch_panic".to_vec()),
                filename: None,
                lineno: None,
                colno: None,
            }],
        },
        BacktraceFrame {
            frame: RawFrame::Fake,
            symbols: vec![
                BacktraceSymbol {
                    name: Some(b"std::rt::lang_start_internal".to_vec()),
                    filename: Some(BytesOrWide::Bytes(b"rust/rt.rs".to_vec())),
                    lineno: Some(300),
                    colno: Some(5),
                },
                BacktraceSymbol {
                    name: Some(b"std::rt::lang_start".to_vec()),
                    filename: Some(BytesOrWide::Bytes(b"rust/rt.rs".to_vec())),
                    lineno: Some(400),
                    colno: None,
                },
            ],
        },
    ]
}

#[test_case]
fn test_debug() {
    let backtrace = Backtrace {
        inner: Inner::Captured(LazyLock::preinit(Capture {
            actual_start: 1,
            frames: generate_fake_frames(),
        })),
    };

    #[rustfmt::skip]
    let expected = "Backtrace [\
    \n    { fn: \"__rust_maybe_catch_panic\" },\
    \n    { fn: \"std::rt::lang_start_internal\", file: \"rust/rt.rs\", line: 300 },\
    \n    { fn: \"std::rt::lang_start\", file: \"rust/rt.rs\", line: 400 },\
    \n]";

    assert_eq!(format!("{backtrace:#?}"), expected);

    // Format the backtrace a second time, just to make sure lazily resolved state is stable
    assert_eq!(format!("{backtrace:#?}"), expected);
}

#[test_case]
#[allow(clippy::useless_vec)]
fn test_frames() {
    let backtrace = Backtrace {
        inner: Inner::Captured(LazyLock::preinit(Capture {
            actual_start: 1,
            frames: generate_fake_frames(),
        })),
    };

    let frames = backtrace.frames();

    #[rustfmt::skip]
    let expected = vec![
        "[
    { fn: \"std::backtrace::Backtrace::create\", file: \"rust/backtrace.rs\", line: 100 },
]",
        "[
    { fn: \"__rust_maybe_catch_panic\" },
]",
        "[
    { fn: \"std::rt::lang_start_internal\", file: \"rust/rt.rs\", line: 300 },
    { fn: \"std::rt::lang_start\", file: \"rust/rt.rs\", line: 400 },
]"
    ];

    let mut iter = frames.iter().zip(expected.iter());

    assert!(iter.all(|(f, e)| format!("{f:#?}") == *e));
}

#[test_case]
fn backtrace_unwind_safe() {
    fn assert_unwind_safe<T: UnwindSafe + RefUnwindSafe>() {}
    assert_unwind_safe::<Backtrace>();
}
