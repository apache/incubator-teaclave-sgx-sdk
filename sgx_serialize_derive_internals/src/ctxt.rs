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

use std::fmt::Display;
use std::cell::RefCell;

#[derive(Default)]
pub struct Ctxt {
    errors: RefCell<Option<Vec<String>>>,
}

impl Ctxt {
    pub fn new() -> Self {
        Ctxt { errors: RefCell::new(Some(Vec::new())) }
    }

    pub fn error<T: Display>(&self, msg: T) {
        self.errors
            .borrow_mut()
            .as_mut()
            .unwrap()
            .push(msg.to_string());
    }

    pub fn check(self) -> Result<(), String> {
        let mut errors = self.errors.borrow_mut().take().unwrap();
        match errors.len() {
            0 => Ok(()),
            1 => Err(errors.pop().unwrap()),
            n => {
                let mut msg = format!("{} errors:", n);
                for err in errors {
                    msg.push_str("\n\t# ");
                    msg.push_str(&err);
                }
                Err(msg)
            }
        }
    }
}

impl Drop for Ctxt {
    fn drop(&mut self) {
        if self.errors.borrow().is_some() {
            panic!("forgot to check for errors");
        }
    }
}
