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

use crate::fs::OpenOptions;
use crate::io;
use crate::io::Result;
use crate::io::SeekFrom;
use crate::io::{BufRead, Read, Seek, Write};
use crate::sys_common::io::test::tmpdir;

use sgx_test_utils::{bench_case, test_case};
use sgx_test_utils::Bencher;

#[test_case]
fn copy_specialization() {
    use crate::io::{BufReader, BufWriter};

    let tmp_path = tmpdir();
    let source_path = tmp_path.join("copy-spec.source");
    let sink_path = tmp_path.join("copy-spec.sink");

    let result: Result<()> = try {
        let mut source = crate::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&source_path)?;
        source.write_all(b"abcdefghiklmnopqr")?;
        source.seek(SeekFrom::Start(8))?;
        let mut source = BufReader::with_capacity(8, source.take(5));
        source.fill_buf()?;
        assert_eq!(source.buffer(), b"iklmn");
        source.get_mut().set_limit(6);
        source.get_mut().get_mut().seek(SeekFrom::Start(1))?; // "bcdefg"
        let mut source = source.take(10); // "iklmnbcdef"

        let mut sink = crate::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&sink_path)?;
        sink.write_all(b"000000")?;
        let mut sink = BufWriter::with_capacity(5, sink);
        sink.write_all(b"wxyz")?;
        assert_eq!(sink.buffer(), b"wxyz");

        let copied = crate::io::copy(&mut source, &mut sink)?;
        assert_eq!(copied, 10, "copy obeyed limit imposed by Take");
        assert_eq!(sink.buffer().len(), 0, "sink buffer was flushed");
        assert_eq!(source.limit(), 0, "outer Take was exhausted");
        assert_eq!(source.get_ref().buffer().len(), 0, "source buffer should be drained");
        assert_eq!(
            source.get_ref().get_ref().limit(),
            1,
            "inner Take allowed reading beyond end of file, some bytes should be left"
        );

        let mut sink = sink.into_inner()?;
        sink.seek(SeekFrom::Start(0))?;
        let mut copied = Vec::new();
        sink.read_to_end(&mut copied)?;
        assert_eq!(&copied, b"000000wxyziklmnbcdef");
    };

    let rm1 = crate::fs::remove_file(source_path);
    let rm2 = crate::fs::remove_file(sink_path);

    assert!(result.and(rm1).and(rm2).is_ok());
}

#[test_case]
fn copies_append_mode_sink() {
    let tmp_path = tmpdir();
    let source_path = tmp_path.join("copies_append_mode.source");
    let sink_path = tmp_path.join("copies_append_mode.sink");

    let result: Result<()> = try {
        let mut source =
            OpenOptions::new().create(true).truncate(true).write(true).read(true).open(&source_path)?;
        write!(source, "not empty")?;
        source.seek(SeekFrom::Start(0))?;
        let mut sink = OpenOptions::new().create(true).append(true).open(&sink_path)?;

        let copied = crate::io::copy(&mut source, &mut sink)?;

        assert_eq!(copied, 9);
    };

    assert!(result.is_ok());
}

#[bench_case]
fn bench_file_to_file_copy(b: &mut Bencher) {
    const BYTES: usize = 128 * 1024;
    let temp_path = tmpdir();
    let src_path = temp_path.join("file-copy-bench-src");
    let mut src = crate::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(src_path)
        .unwrap();
    src.write(&vec![0u8; BYTES]).unwrap();

    let sink_path = temp_path.join("file-copy-bench-sink");
    let mut sink = crate::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(sink_path)
        .unwrap();

    b.bytes = BYTES as u64;
    b.iter(|| {
        src.seek(SeekFrom::Start(0)).unwrap();
        sink.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(BYTES as u64, io::copy(&mut src, &mut sink).unwrap());
    });
}

#[bench_case]
fn bench_file_to_socket_copy(b: &mut Bencher) {
    const BYTES: usize = 128 * 1024;
    let temp_path = tmpdir();
    let src_path = temp_path.join("pipe-copy-bench-src");
    let mut src = OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(src_path)
        .unwrap();
    src.write(&vec![0u8; BYTES]).unwrap();

    let sink_drainer = crate::net::TcpListener::bind("localhost:0").unwrap();
    let mut sink = crate::net::TcpStream::connect(sink_drainer.local_addr().unwrap()).unwrap();
    let mut sink_drainer = sink_drainer.accept().unwrap().0;

    crate::thread::spawn(move || {
        let mut sink_buf = vec![0u8; 1024 * 1024];
        loop {
            sink_drainer.read(&mut sink_buf[..]).unwrap();
        }
    });

    b.bytes = BYTES as u64;
    b.iter(|| {
        src.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(BYTES as u64, io::copy(&mut src, &mut sink).unwrap());
    });
}

#[bench_case]
fn bench_file_to_uds_copy(b: &mut Bencher) {
    const BYTES: usize = 128 * 1024;
    let temp_path = tmpdir();
    let src_path = temp_path.join("uds-copy-bench-src");
    let mut src = OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(src_path)
        .unwrap();
    src.write(&vec![0u8; BYTES]).unwrap();

    let (mut sink, mut sink_drainer) = crate::os::unix::net::UnixStream::pair().unwrap();

    crate::thread::spawn(move || {
        let mut sink_buf = vec![0u8; 1024 * 1024];
        loop {
            sink_drainer.read(&mut sink_buf[..]).unwrap();
        }
    });

    b.bytes = BYTES as u64;
    b.iter(|| {
        src.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(BYTES as u64, io::copy(&mut src, &mut sink).unwrap());
    });
}
