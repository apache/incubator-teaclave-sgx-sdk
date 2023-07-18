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

use spin::Once;

#[derive(Clone, Copy)]
pub struct UserRange {
    pub start: usize,
    pub end: usize,
}

impl UserRange {
    fn start(&self) -> usize {
        self.start
    }
    fn end(&self) -> usize {
        self.end
    }
}

pub static USER_RANGE: Once<UserRange> = Once::new();

pub fn init_range(start: usize, end: usize) {
    // init
    let _ = *USER_RANGE.call_once(|| UserRange { start, end });
}

pub fn is_within_rts_range(start: usize, len: usize) -> bool {
    let end = if len > 0 {
        if let Some(end) = start.checked_add(len - 1) {
            end
        } else {
            return false;
        }
    } else {
        start
    };
    let user_range = USER_RANGE.get().unwrap();
    let user_start = user_range.start();
    let user_end = user_range.end();

    (start >= user_end) || (end < user_start)
}

pub fn is_within_user_range(start: usize, len: usize) -> bool {
    let end = if len > 0 {
        if let Some(end) = start.checked_add(len - 1) {
            end
        } else {
            return false;
        }
    } else {
        start
    };
    let user_range = USER_RANGE.get().unwrap();
    let user_start = user_range.start();
    let user_end = user_range.end();

    (start <= end) && (start >= user_start) && (end < user_end)
}
