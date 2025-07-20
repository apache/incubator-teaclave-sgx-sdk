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

pub const DEFAULT_BUF_SIZE: usize = 8 * 1024;

#[cfg(feature = "unit_test")]
#[allow(dead_code)] // not used on emscripten
pub mod test {
    use crate::env;
    use crate::fs;
    use crate::path::{Path, PathBuf};
    use crate::thread;
    use sgx_trts::rand::Rng;

    pub struct TempDir(PathBuf);

    impl TempDir {
        pub fn join(&self, path: &str) -> PathBuf {
            let TempDir(ref p) = *self;
            p.join(path)
        }

        pub fn path(&self) -> &Path {
            let TempDir(ref p) = *self;
            p
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            // Gee, seeing how we're testing the fs module I sure hope that we
            // at least implement this correctly!
            let TempDir(ref p) = *self;
            let result = fs::remove_dir_all(p);
            // Avoid panicking while panicking as this causes the process to
            // immediately abort, without displaying test results.
            if !thread::panicking() {
                result.unwrap();
            }
        }
    }

    pub fn tmpdir() -> TempDir {
        let p = env::temp_dir();
        let mut rng = Rng::new();
        let ret = p.join(format!("rust-{}", rng.next_u32()));
        fs::create_dir(&ret).unwrap();
        TempDir(ret)
    }
}
