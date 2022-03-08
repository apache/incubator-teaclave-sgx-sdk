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

use super::{const_io_error, Custom, Error, ErrorData, ErrorKind, Repr};
use crate::assert_matches::assert_matches;
use crate::error;
use crate::fmt;
use crate::mem::size_of;
use crate::sys::decode_error_kind;
use crate::sys::os::error_string;

use sgx_test_utils::test_case;

#[test_case]
fn test_size() {
    assert!(size_of::<Error>() <= size_of::<[usize; 2]>());
}

#[test_case]
fn test_debug_error() {
    let code = 6;
    let msg = error_string(code);
    let kind = decode_error_kind(code);
    let err = Error {
        repr: Repr::new_custom(box Custom {
            kind: ErrorKind::InvalidInput,
            error: box Error { repr: super::Repr::new_os(code) },
        }),
    };
    let expected = format!(
        "Custom {{ \
         kind: InvalidInput, \
         error: Os {{ \
         code: {:?}, \
         kind: {:?}, \
         message: {:?} \
         }} \
         }}",
        code, kind, msg
    );
    assert_eq!(format!("{:?}", err), expected);
}

#[test_case]
fn test_downcasting() {
    #[derive(Debug)]
    struct TestError;

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("asdf")
        }
    }

    impl error::Error for TestError {}

    // we have to call all of these UFCS style right now since method
    // resolution won't implicitly drop the Send+Sync bounds
    let mut err = Error::new(ErrorKind::Other, TestError);
    assert!(err.get_ref().unwrap().is::<TestError>());
    assert_eq!("asdf", err.get_ref().unwrap().to_string());
    assert!(err.get_mut().unwrap().is::<TestError>());
    let extracted = err.into_inner().unwrap();
    extracted.downcast::<TestError>().unwrap();
}

#[test_case]
fn test_const() {
    const E: Error = const_io_error!(ErrorKind::NotFound, "hello");

    assert_eq!(E.kind(), ErrorKind::NotFound);
    assert_eq!(E.to_string(), "hello");
    assert!(format!("{:?}", E).contains("\"hello\""));
    assert!(format!("{:?}", E).contains("NotFound"));
}

#[test_case]
fn test_os_packing() {
    for code in -20i32..20i32 {
        let e = Error::from_raw_os_error(code);
        assert_eq!(e.raw_os_error(), Some(code));
        assert_matches!(
            e.repr.data(),
            ErrorData::Os(c) if c == code,
        );
    }
}

#[test_case]
fn test_errorkind_packing() {
    assert_eq!(Error::from(ErrorKind::NotFound).kind(), ErrorKind::NotFound);
    assert_eq!(Error::from(ErrorKind::PermissionDenied).kind(), ErrorKind::PermissionDenied);
    assert_eq!(Error::from(ErrorKind::Uncategorized).kind(), ErrorKind::Uncategorized);
    // Check that the innards look like like what we want.
    assert_matches!(
        Error::from(ErrorKind::OutOfMemory).repr.data(),
        ErrorData::Simple(ErrorKind::OutOfMemory),
    );
}

#[test_case]
fn test_simple_message_packing() {
    use super::{ErrorKind::*, SimpleMessage};
    macro_rules! check_simple_msg {
        ($err:expr, $kind:ident, $msg:literal) => {{
            let e = &$err;
            // Check that the public api is right.
            assert_eq!(e.kind(), $kind);
            assert!(format!("{:?}", e).contains($msg));
            // and we got what we expected
            assert_matches!(
                e.repr.data(),
                ErrorData::SimpleMessage(SimpleMessage { kind: $kind, message: $msg })
            );
        }};
    }

    let not_static = const_io_error!(Uncategorized, "not a constant!");
    check_simple_msg!(not_static, Uncategorized, "not a constant!");

    const CONST: Error = const_io_error!(NotFound, "definitely a constant!");
    check_simple_msg!(CONST, NotFound, "definitely a constant!");

    static STATIC: Error = const_io_error!(BrokenPipe, "a constant, sort of!");
    check_simple_msg!(STATIC, BrokenPipe, "a constant, sort of!");
}

#[derive(Debug, PartialEq)]
struct Bojji(bool);
impl error::Error for Bojji {}
impl fmt::Display for Bojji {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ah! {:?}", self)
    }
}

#[test_case]
fn test_custom_error_packing() {
    use super::Custom;
    let test = Error::new(ErrorKind::Uncategorized, Bojji(true));
    assert_matches!(
        test.repr.data(),
        ErrorData::Custom(Custom {
            kind: ErrorKind::Uncategorized,
            error,
        }) if error.downcast_ref::<Bojji>() == Some(&Bojji(true)),
    );
}
