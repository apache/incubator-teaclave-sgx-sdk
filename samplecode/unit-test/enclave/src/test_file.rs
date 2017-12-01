// Copyright (c) 2017 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use sgx_rand::{Rng, StdRng};
use std::fs::{self, SgxFile};
use std::io::{Read, Write};

pub fn test_file() {

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

    let result = fs::remove("sgx_file");
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
    {
        let opt = SgxFile::open("/dev/isgx");
        assert_eq!(opt.is_ok(), true);
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
