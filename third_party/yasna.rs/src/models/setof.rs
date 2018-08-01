// Copyright 2016 Masaki Hara
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::prelude::v1::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SetOf<T> {
    pub vec: Vec<T>,
}

impl<T> SetOf<T> {
    pub fn new() -> Self {
        SetOf {
            vec: Vec::new(),
        }
    }
}
