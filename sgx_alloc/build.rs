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

use std::env;
use std::process::Command;

fn main() {
    // nightly-2020-12-05 (rustc 2020-12-04)
    // Rename `AllocRef` to `Allocator` and `(de)alloc` to `(de)allocate`
    // https://github.com/rust-lang/rust/commit/9274b37d99f608e5fde569788ee79bd72fc3cf13
    if let Some(true) = is_min_date("2020-12-04") {
        println!("cargo:rustc-cfg=enable_allocator_traits");
    }
}

// code below copied from crate version_check
// we want to remove the build dependencies to make the dependency tree
// as clean as possible. the following codes credit to SergioBenitez
#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
struct Date(u32);

impl Date {
    fn read() -> Option<Date> {
        get_version_and_date()
            .and_then(|(_, date)| date)
            .and_then(|date| Date::parse(&date))
    }

    fn parse(date: &str) -> Option<Date> {
        let ymd: Vec<u32> = date.split("-")
            .filter_map(|s| s.parse::<u32>().ok())
            .collect();
    
        if ymd.len() != 3 {
            return None
        }
    
        let (y, m, d) = (ymd[0], ymd[1], ymd[2]);
        Some(Date((y << 9) | ((m & 0xF) << 5) | (d & 0x1F)))
    }
}

fn get_version_and_date() -> Option<(Option<String>, Option<String>)> {
    env::var("RUSTC").ok()
        .and_then(|rustc| Command::new(rustc).arg("--version").output().ok())
        .or_else(|| Command::new("rustc").arg("--version").output().ok())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| version_and_date_from_rustc_version(&s))
}

fn version_and_date_from_rustc_version(s: &str) -> (Option<String>, Option<String>) {
    let last_line = s.lines().last().unwrap_or(s);
    let mut components = last_line.trim().split(" ");
    let version = components.nth(1);
    let date = components.filter(|c| c.ends_with(')')).next()
        .map(|s| s.trim_end().trim_end_matches(")").trim_start().trim_start_matches('('));
    (version.map(|s| s.to_string()), date.map(|s| s.to_string()))
}

fn is_min_date(min_date: &str) -> Option<bool> {
    match (Date::read(), Date::parse(min_date)) {
        (Some(rustc_date), Some(min_date)) => Some(rustc_date >= min_date),
        _ => None
    }
}
