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

use super::*;
use crate::collections::HashMap;

use sgx_test_utils::test_case;

#[test_case]
fn no_lookup_host_duplicates() {
    let mut addrs = HashMap::new();
    let lh = match LookupHost::try_from(("localhost", 0)) {
        Ok(lh) => lh,
        Err(e) => panic!("couldn't resolve `localhost`: {e}"),
    };
    for sa in lh {
        *addrs.entry(sa).or_insert(0) += 1;
    }
    assert_eq!(
        addrs.iter().filter(|&(_, &v)| v > 1).collect::<Vec<_>>(),
        vec![],
        "There should be no duplicate localhost entries"
    );
}
