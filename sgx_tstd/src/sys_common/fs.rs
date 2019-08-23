// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
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
#![allow(dead_code)]
#[cfg(feature = "untrusted_fs")]
use crate::fs;
#[cfg(not(feature = "untrusted_fs"))]
use crate::untrusted::fs;
use crate::io::{self, Error, ErrorKind};
use crate::path::Path;

pub fn remove_dir_all(path: &Path) -> io::Result<()> {
    let filetype = fs::symlink_metadata(path)?.file_type();
    if filetype.is_symlink() {
        fs::remove_file(path)
    } else {
        remove_dir_all_recursive(path)
    }
}

fn remove_dir_all_recursive(path: &Path) -> io::Result<()> {
    for child in fs::read_dir(path)? {
        let child = child?;
        if child.file_type()?.is_dir() {
            remove_dir_all_recursive(&child.path())?;
        } else {
            fs::remove_file(&child.path())?;
        }
    }
    fs::remove_dir(path)
}