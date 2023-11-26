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

#![allow(dead_code)]

use crate::collections::BTreeMap;
use crate::env;
use crate::ffi::{OsStr, OsString};
use crate::fmt;
use crate::io;
use crate::sys::unsupported::pipe::read2;
use crate::sys::process::{EnvKey, ExitStatus, Process, StdioPipes};

// Stores a set of changes to an environment
#[derive(Clone)]
pub struct CommandEnv {
    clear: bool,
    saw_path: bool,
    vars: BTreeMap<EnvKey, Option<OsString>>,
}

#[allow(clippy::derivable_impls)]
impl Default for CommandEnv {
    fn default() -> Self {
        CommandEnv { clear: false, saw_path: false, vars: Default::default() }
    }
}

impl fmt::Debug for CommandEnv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_command_env = f.debug_struct("CommandEnv");
        debug_command_env.field("clear", &self.clear).field("vars", &self.vars);
        debug_command_env.finish()
    }
}

impl CommandEnv {
    // Capture the current environment with these changes applied
    #[allow(clippy::useless_conversion)]
    pub fn capture(&self) -> BTreeMap<EnvKey, OsString> {
        let mut result = BTreeMap::<EnvKey, OsString>::new();
        if !self.clear {
            for (k, v) in env::vars_os() {
                result.insert(k.into(), v);
            }
        }
        for (k, maybe_v) in &self.vars {
            if let Some(v) = maybe_v {
                result.insert(k.clone(), v.clone());
            } else {
                result.remove(k);
            }
        }
        result
    }

    pub fn is_unchanged(&self) -> bool {
        !self.clear && self.vars.is_empty()
    }

    pub fn capture_if_changed(&self) -> Option<BTreeMap<EnvKey, OsString>> {
        if self.is_unchanged() { None } else { Some(self.capture()) }
    }

    // The following functions build up changes
    pub fn set(&mut self, key: &OsStr, value: &OsStr) {
        let key = EnvKey::from(key);
        self.maybe_saw_path(&key);
        self.vars.insert(key, Some(value.to_owned()));
    }

    pub fn remove(&mut self, key: &OsStr) {
        let key = EnvKey::from(key);
        self.maybe_saw_path(&key);
        if self.clear {
            self.vars.remove(&key);
        } else {
            self.vars.insert(key, None);
        }
    }

    pub fn clear(&mut self) {
        self.clear = true;
        self.vars.clear();
    }

    pub fn does_clear(&self) -> bool {
        self.clear
    }

    pub fn have_changed_path(&self) -> bool {
        self.saw_path || self.clear
    }

    fn maybe_saw_path(&mut self, key: &EnvKey) {
        if !self.saw_path && key == "PATH" {
            self.saw_path = true;
        }
    }

    pub fn iter(&self) -> CommandEnvs<'_> {
        let iter = self.vars.iter();
        CommandEnvs { iter }
    }
}

/// An iterator over the command environment variables.
///
/// This struct is created by
/// [`Command::get_envs`][crate::process::Command::get_envs]. See its
/// documentation for more.
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Debug)]
pub struct CommandEnvs<'a> {
    iter: crate::collections::btree_map::Iter<'a, EnvKey, Option<OsString>>,
}

impl<'a> Iterator for CommandEnvs<'a> {
    type Item = (&'a OsStr, Option<&'a OsStr>);
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(key, value)| (key.as_ref(), value.as_deref()))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> ExactSizeIterator for CommandEnvs<'a> {
    fn len(&self) -> usize {
        self.iter.len()
    }
    fn is_empty(&self) -> bool {
        self.iter.is_empty()
    }
}

pub fn wait_with_output(
    mut process: Process,
    mut pipes: StdioPipes,
) -> io::Result<(ExitStatus, Vec<u8>, Vec<u8>)> {
    drop(pipes.stdin.take());

    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    match (pipes.stdout.take(), pipes.stderr.take()) {
        (None, None) => {}
        (Some(out), None) => {
            let res = out.read_to_end(&mut stdout);
            res.unwrap();
        }
        (None, Some(err)) => {
            let res = err.read_to_end(&mut stderr);
            res.unwrap();
        }
        (Some(out), Some(err)) => {
            let res = read2(out, &mut stdout, err, &mut stderr);
            res.unwrap();
        }
    }

    let status = process.wait()?;
    Ok((status, stdout, stderr))
}
