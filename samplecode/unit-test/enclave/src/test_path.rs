use core::str;
use sgx_rand::*;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

macro_rules! check {
    ($e:expr) => {
        match $e {
            Ok(t) => t,
            Err(e) => panic!("{} failed with: {}", stringify!($e), e),
        }
    };
}

pub struct TempDir(PathBuf);

impl TempDir {
    pub fn join(&self, path: &str) -> PathBuf {
        let TempDir(ref p) = *self;
        p.join(path)
    }

    pub fn path<'a>(&'a self) -> &'a Path {
        let TempDir(ref p) = *self;
        p
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // Gee, seeing how we're testing the fs module I sure hope that we
        // at least implement this correctly!
        let TempDir(ref p) = *self;
        fs::remove_dir_all(p).unwrap();
    }
}

pub fn tmpdir() -> TempDir {
    let p = env::temp_dir();
    let mut r = os::SgxRng::new().unwrap();
    let ret = p.join(&format!("sgx_rust-{}", r.next_u32()));
    fs::create_dir(&ret).unwrap();
    TempDir(ret)
}

pub fn test_path_stat_is_correct_on_is_dir() {
    let tmpdir = tmpdir();
    let filename = &tmpdir.join("file_stat_correct_on_is_dir");

    check!(fs::create_dir(filename));
    let stat_res_fn = check!(fs::metadata(filename));
    assert!(stat_res_fn.is_dir());
    let stat_res_meth = check!(filename.metadata());
    assert!(stat_res_meth.is_dir());
    check!(fs::remove_dir(filename));
}

pub fn test_path_fileinfo_false_when_checking_is_file_on_a_directory() {
    let tmpdir = tmpdir();
    let dir = &tmpdir.join("fileinfo_false_on_dir");
    check!(fs::create_dir(dir));
    assert!(!dir.is_file());
    check!(fs::remove_dir(dir));
}

pub fn test_path_directoryinfo_check_exists_before_and_after_mkdir() {
    let tmpdir = tmpdir();
    let dir = &tmpdir.join("before_and_after_dir");
    assert!(!dir.exists());
    check!(fs::create_dir(dir));
    assert!(dir.exists());
    assert!(dir.is_dir());
    check!(fs::remove_dir(dir));
    assert!(!dir.exists());
}

pub fn test_path_directoryinfo_readdir() {
    let tmpdir = tmpdir();
    let dir = &tmpdir.join("di_readdir");
    check!(fs::create_dir(dir));
    let prefix = "foo";
    for n in 0..3 {
        let f = dir.join(&format!("{}.txt", n));
        let mut w = check!(File::create(&f));
        let msg_str = format!("{}{}", prefix, n);
        let msg = msg_str.as_bytes();
        check!(w.write(msg));
    }

    let files = check!(fs::read_dir(dir));
    let mut mem = [0; 4];
    for f in files {
        let f = f.unwrap().path();

        {
            let n = f.file_stem().unwrap();
            check!(check!(File::open(&f)).read(&mut mem));
            let read_str = str::from_utf8(&mem).unwrap();
            let expected = format!("{}{}", prefix, n.to_str().unwrap());
            assert_eq!(expected, read_str);
        }
        check!(fs::remove_file(&f));
    }
    check!(fs::remove_dir(dir));
}

pub fn test_path_mkdir_path_already_exists_error() {
    let tmpdir = tmpdir();
    let dir = &tmpdir.join("mkdir_error_twice");
    check!(fs::create_dir(dir));
    let e = fs::create_dir(dir).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::AlreadyExists);
}

pub fn test_path_recursive_mkdir() {
    let tmpdir = tmpdir();
    let dir = tmpdir.join("d1/d2");
    check!(fs::create_dir_all(&dir));
    assert!(dir.is_dir())
}

pub fn test_path_recursive_mkdir_failure() {
    let tmpdir = tmpdir();
    let dir = tmpdir.join("d1");
    let file = dir.join("f1");

    check!(fs::create_dir_all(&dir));
    check!(File::create(&file));

    let result = fs::create_dir_all(&file);

    assert!(result.is_err());
}

pub fn test_path_recursive_mkdir_slash() {
    check!(fs::create_dir_all(Path::new("/")));
}

pub fn test_path_recursive_mkdir_dot() {
    check!(fs::create_dir_all(Path::new(".")));
}

pub fn test_path_recursive_mkdir_empty() {
    check!(fs::create_dir_all(Path::new("")));
}

pub fn test_path_recursive_rmdir() {
    let tmpdir = tmpdir();
    let d1 = tmpdir.join("d1");
    let dt = d1.join("t");
    let dtt = dt.join("t");
    let d2 = tmpdir.join("d2");
    let canary = d2.join("do_not_delete");
    check!(fs::create_dir_all(&dtt));
    check!(fs::create_dir_all(&d2));
    check!(check!(File::create(&canary)).write(b"foo"));
    check!(fs::soft_link(&d2, &dt.join("d2")));
    let _ = fs::soft_link(&canary, &d1.join("canary"));
    check!(fs::remove_dir_all(&d1));

    assert!(!d1.is_dir());
    assert!(canary.exists());
}

pub fn test_path_recursive_rmdir_of_symlink() {
    // test we do not recursively delete a symlink but only dirs.
    let tmpdir = tmpdir();

    let link = tmpdir.join("d1");
    let dir = tmpdir.join("d2");
    let canary = dir.join("do_not_delete");
    check!(fs::create_dir_all(&dir));

    check!(check!(File::create(&canary)).write(b"foo"));
    check!(fs::soft_link(&dir, &link));

    check!(fs::remove_dir_all(&link));

    assert!(!link.is_dir());
    assert!(canary.exists());
}

pub fn test_path_unicode_path_is_dir() {
    assert!(Path::new(".").is_dir());
    assert!(!Path::new("test/stdtest/fs.rs").is_dir());

    let tmpdir = tmpdir();

    let mut dirpath = tmpdir.path().to_path_buf();
    dirpath.push("test-가一ー你好");
    check!(fs::create_dir(&dirpath));
    assert!(dirpath.is_dir());

    let mut filepath = dirpath;
    filepath.push("unicode-file-\u{ac00}\u{4e00}\u{30fc}\u{4f60}\u{597d}.rs");
    check!(File::create(&filepath)); // ignore return; touch only
    assert!(!filepath.is_dir());
    assert!(filepath.exists());
}

pub fn test_path_unicode_path_exists() {
    assert!(Path::new(".").exists());
    assert!(!Path::new("test/nonexistent-bogus-path").exists());

    let tmpdir = tmpdir();
    let unicode = tmpdir.path();
    let unicode = unicode.join("test-각丁ー再见");
    check!(fs::create_dir(&unicode));
    assert!(unicode.exists());
    assert!(!Path::new("test/unicode-bogus-path-각丁ー再见").exists());
}

pub fn test_path_copy_file_dst_dir() {
    let tmpdir = tmpdir();
    let out = tmpdir.join("out");

    check!(File::create(&out));
    match fs::copy(&*out, tmpdir.path()) {
        Ok(..) => panic!(),
        Err(..) => {}
    }
}

pub fn test_path_copy_file_src_dir() {
    let tmpdir = tmpdir();
    let out = tmpdir.join("out");

    match fs::copy(tmpdir.path(), &out) {
        Ok(..) => panic!(),
        Err(..) => {}
    }
    assert!(!out.exists());
}

pub fn test_path_mkdir_trailing_slash() {
    let tmpdir = tmpdir();
    let path = tmpdir.join("file");
    check!(fs::create_dir_all(&path.join("a/")));
}

pub fn test_path_canonicalize_works_simple() {
    let tmpdir = tmpdir();
    let tmpdir = fs::canonicalize(tmpdir.path()).unwrap();
    let file = tmpdir.join("test");
    File::create(&file).unwrap();
    assert_eq!(fs::canonicalize(&file).unwrap(), file);
}

pub fn test_path_dir_entry_methods() {
    let tmpdir = tmpdir();

    fs::create_dir_all(&tmpdir.join("a")).unwrap();
    File::create(&tmpdir.join("b")).unwrap();

    for file in tmpdir.path().read_dir().unwrap().map(|f| f.unwrap()) {
        let fname = file.file_name();
        match fname.to_str() {
            Some("a") => {
                assert!(file.file_type().unwrap().is_dir());
                assert!(file.metadata().unwrap().is_dir());
            }
            Some("b") => {
                assert!(file.file_type().unwrap().is_file());
                assert!(file.metadata().unwrap().is_file());
            }
            f => panic!("unknown file name: {:?}", f),
        }
    }
}

pub fn test_path_read_dir_not_found() {
    let res = fs::read_dir("/path/that/does/not/exist");
    assert_eq!(res.err().unwrap().kind(), ErrorKind::NotFound);
}

pub fn test_path_create_dir_all_with_junctions() {
    let tmpdir = tmpdir();
    let target = tmpdir.join("target");

    let junction = tmpdir.join("junction");
    let b = junction.join("a/b");

    let link = tmpdir.join("link");
    let d = link.join("c/d");

    fs::create_dir(&target).unwrap();

    check!(fs::soft_link(&target, &junction));
    check!(fs::create_dir_all(&b));
    // the junction itself is not a directory, but `is_dir()` on a Path
    // follows links
    assert!(junction.is_dir());
    assert!(b.exists());

    check!(fs::create_dir_all(&d));
    assert!(link.is_dir());
    assert!(d.exists());
}

pub fn test_path_copy_file_follows_dst_symlink() {
    let tmp = tmpdir();

    let in_path = tmp.join("in.txt");
    let out_path = tmp.join("out.txt");
    let out_path_symlink = tmp.join("out_symlink.txt");

    check!(fs::write(&in_path, "foo"));
    check!(fs::write(&out_path, "bar"));
    check!(fs::soft_link(&out_path, &out_path_symlink));

    check!(fs::copy(&in_path, &out_path_symlink));

    assert!(check!(out_path_symlink.symlink_metadata())
        .file_type()
        .is_symlink());
    assert_eq!(check!(fs::read(&out_path_symlink)), b"foo".to_vec());
    assert_eq!(check!(fs::read(&out_path)), b"foo".to_vec());
}
