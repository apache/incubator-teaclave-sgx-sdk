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

use sgx_test_utils::test_case;

#[test_case]
fn quickack() {
    use crate::{
        net::{test::next_test_ip4, TcpListener, TcpStream},
        os::net::linux_ext::tcp::TcpStreamExt,
    };

    macro_rules! t {
        ($e:expr) => {
            match $e {
                Ok(t) => t,
                Err(e) => panic!("received error for `{}`: {}", stringify!($e), e),
            }
        };
    }

    let addr = next_test_ip4();
    let _listener = t!(TcpListener::bind(addr));

    let stream = t!(TcpStream::connect(("localhost", addr.port())));

    t!(stream.set_quickack(false));
    assert!(!t!(stream.quickack()));
    t!(stream.set_quickack(true));
    assert!(t!(stream.quickack()));
    t!(stream.set_quickack(false));
    assert!(!t!(stream.quickack()));
}
