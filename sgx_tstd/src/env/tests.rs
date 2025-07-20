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

#![allow(clippy::unnecessary_mut_passed)]

use super::*;

use crate::path::Path;

use sgx_test_utils::test_case;

#[test_case]
fn test_self_exe_path() {
    let path = current_exe();
    assert!(path.is_ok());
    let path = path.unwrap();

    // Hard to test this function
    assert!(path.is_absolute());
}

#[test_case]
fn test() {
    assert!((!Path::new("test-path").is_absolute()));
    current_dir().unwrap();
}

#[test_case]
fn split_paths_unix() {
    use crate::path::PathBuf;

    fn check_parse(unparsed: &str, parsed: &[&str]) -> bool {
        split_paths(unparsed).collect::<Vec<_>>()
            == parsed.iter().map(|s| PathBuf::from(*s)).collect::<Vec<_>>()
    }

    assert!(check_parse("", &mut [""]));
    assert!(check_parse("::", &mut ["", "", ""]));
    assert!(check_parse("/", &mut ["/"]));
    assert!(check_parse("/:", &mut ["/", ""]));
    assert!(check_parse("/:/usr/local", &mut ["/", "/usr/local"]));
}

#[test_case]
fn join_paths_unix() {
    use crate::ffi::OsStr;

    fn test_eq(input: &[&str], output: &str) -> bool {
        &*join_paths(input.iter().cloned()).unwrap() == OsStr::new(output)
    }

    assert!(test_eq(&[], ""));
    assert!(test_eq(&["/bin", "/usr/bin", "/usr/local/bin"], "/bin:/usr/bin:/usr/local/bin"));
    assert!(test_eq(&["", "/bin", "", "", "/usr/bin", ""], ":/bin:::/usr/bin:"));
    assert!(join_paths(["/te:st"].iter().cloned()).is_err());
}

#[test_case]
fn args_debug() {
    assert_eq!(
        format!("Args {{ inner: {:?} }}", args().collect::<Vec<_>>()),
        format!("{:?}", args())
    );
}

#[test_case]
fn args_os_debug() {
    assert_eq!(
        format!("ArgsOs {{ inner: {:?} }}", args_os().collect::<Vec<_>>()),
        format!("{:?}", args_os())
    );
}

#[test_case]
fn vars_debug() {
    assert_eq!(
        format!("Vars {{ inner: {:?} }}", vars().collect::<Vec<_>>()),
        format!("{:?}", vars())
    );
}

#[test_case]
fn vars_os_debug() {
    assert_eq!(
        format!("VarsOs {{ inner: {:?} }}", vars_os().collect::<Vec<_>>()),
        format!("{:?}", vars_os())
    );
}
