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

use sgx_rand::{Rng, StdRng};
use std::io::{Read, Write};
use std::sgxfs::{self, SgxFile};
use std::string::*;
use std::untrusted::fs::remove_file;
use std::untrusted::fs::File;

pub fn test_sgxfs() {
    let mut write_data: [u8; 16] = [0; 16];
    let mut read_data: [u8; 16] = [0; 16];
    let write_size;
    let read_size;
    {
        let mut rand = StdRng::new().unwrap();
        rand.fill_bytes(&mut write_data);

        let opt = SgxFile::create("sgx_file");
        assert_eq!(opt.is_ok(), true);
        let mut file = opt.unwrap();

        let result = file.write(&write_data);
        assert_eq!(result.is_ok(), true);
        write_size = result.unwrap();
    }

    {
        let opt = SgxFile::open("sgx_file");
        assert_eq!(opt.is_ok(), true);
        let mut file = opt.unwrap();

        let result = file.read(&mut read_data);
        assert_eq!(result.is_ok(), true);
        read_size = result.unwrap();
    }

    let result = sgxfs::remove("sgx_file");
    assert_eq!(result.is_ok(), true);

    assert_eq!(write_data, read_data);
    assert_eq!(write_size, read_size);

    {
        let opt = SgxFile::open("/");
        assert_eq!(opt.is_err(), true);
        let opt = SgxFile::open(".");
        assert_eq!(opt.is_err(), true);
        let opt = SgxFile::open("..");
        assert_eq!(opt.is_err(), true);
        let opt = SgxFile::open("?");
        assert_eq!(opt.is_err(), true);
    }
    #[cfg(feature = "hw_test")]
    {
        let opt1 = SgxFile::open("/dev/isgx");
        let opt2 = SgxFile::open("/dev/sgx/enclave");
        assert_eq!(opt1.is_ok() || opt2.is_ok(), true);
    }
    {
        let opt = SgxFile::create("/");
        assert_eq!(opt.is_err(), true);
    }
    {
        let opt = SgxFile::create("/proc/100");
        assert_eq!(opt.is_err(), true);
        let opt = SgxFile::create(".");
        assert_eq!(opt.is_err(), true);
        let opt = SgxFile::create("..");
        assert_eq!(opt.is_err(), true);
    }
}

pub fn test_fs() {
    {
        let f = File::create("foo.txt");
        assert!(f.is_ok());

        let result = f.unwrap().write_all(b"Hello, world!");
        assert!(result.is_ok());

        let f = File::open("foo.txt");
        assert!(f.is_ok());

        let mut s = String::new();
        let result = f.unwrap().read_to_string(&mut s);
        assert!(result.is_ok());
        assert_eq!(s, "Hello, world!");

        let f = remove_file("foo.txt");
        assert!(f.is_ok());
    }
}

pub fn test_fs_untrusted_fs_feature_enabled() {
    {
        use std::fs;
        let f = fs::File::create("foo.txt");
        assert!(f.is_ok());

        let result = f.unwrap().write_all(b"Hello, world!");
        assert!(result.is_ok());

        let f = fs::File::open("foo.txt");
        assert!(f.is_ok());

        let mut s = String::new();
        let result = f.unwrap().read_to_string(&mut s);
        assert!(result.is_ok());
        assert_eq!(s, "Hello, world!");

        let f = remove_file("foo.txt");
        assert!(f.is_ok());
    }
}
