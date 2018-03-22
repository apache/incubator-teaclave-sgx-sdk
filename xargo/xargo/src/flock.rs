//! Copy paste of Cargo's src/util/flock.rs with modifications to not depend on
//! other Cargo stuff

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Display, Path, PathBuf};
use std::{fs, io};

use fs2::FileExt;
use fs2;

#[derive(PartialEq)]
enum State {
    Exclusive,
    Shared,
}

pub struct FileLock {
    file: File,
    path: PathBuf,
}

impl FileLock {
    pub fn parent(&self) -> &Path {
        self.path.parent().unwrap()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn remove_siblings(&self) -> io::Result<()> {
        let path = self.path();
        for entry in path.parent().unwrap().read_dir()? {
            let entry = entry?;
            if Some(&entry.file_name()[..]) == path.file_name() {
                continue;
            }
            let kind = entry.file_type()?;
            if kind.is_dir() {
                fs::remove_dir_all(entry.path())?;
            } else {
                fs::remove_file(entry.path())?;
            }
        }
        Ok(())
    }
}

pub struct Filesystem {
    path: PathBuf,
}

impl Filesystem {
    pub fn new(path: PathBuf) -> Filesystem {
        Filesystem { path: path }
    }

    pub fn join<T>(&self, other: T) -> Filesystem
    where
        T: AsRef<Path>,
    {
        Filesystem::new(self.path.join(other))
    }

    pub fn open_ro<P>(&self, path: P, msg: &str) -> io::Result<FileLock>
    where
        P: AsRef<Path>,
    {
        self.open(
            path.as_ref(),
            OpenOptions::new().read(true),
            State::Shared,
            msg,
        )
    }

    pub fn open_rw<P>(&self, path: P, msg: &str) -> io::Result<FileLock>
    where
        P: AsRef<Path>,
    {
        self.open(
            path.as_ref(),
            OpenOptions::new().read(true).write(true).create(true),
            State::Exclusive,
            msg,
        )
    }

    fn open(
        &self,
        path: &Path,
        opts: &OpenOptions,
        state: State,
        msg: &str,
    ) -> io::Result<FileLock> {
        let path = self.path.join(path);

        let f = opts.open(&path).or_else(|e| {
            if e.kind() == io::ErrorKind::NotFound && state == State::Exclusive {
                create_dir_all(path.parent().unwrap())?;
                opts.open(&path)
            } else {
                Err(e)
            }
        })?;

        match state {
            State::Exclusive => {
                acquire(
                    msg,
                    &path,
                    &|| f.try_lock_exclusive(),
                    &|| f.lock_exclusive(),
                )?;
            }
            State::Shared => {
                acquire(msg, &path, &|| f.try_lock_shared(), &|| f.lock_shared())?;
            }
        }

        Ok(FileLock {
            file: f,
            path: path,
        })
    }

    pub fn display(&self) -> Display {
        self.path.display()
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        self.file.unlock().ok();
    }
}

fn acquire(
    msg: &str,
    path: &Path,
    try: &Fn() -> io::Result<()>,
    block: &Fn() -> io::Result<()>,
) -> io::Result<()> {
    #[cfg(all(target_os = "linux", not(target_env = "musl")))]
    fn is_on_nfs_mount(path: &Path) -> bool {
        use std::ffi::CString;
        use std::mem;
        use std::os::unix::prelude::*;

        let path = match CString::new(path.as_os_str().as_bytes()) {
            Ok(path) => path,
            Err(_) => return false,
        };

        unsafe {
            let mut buf: ::libc::statfs = mem::zeroed();
            let r = ::libc::statfs(path.as_ptr(), &mut buf);

            r == 0 && buf.f_type as u32 == ::libc::NFS_SUPER_MAGIC as u32
        }
    }

    #[cfg(any(not(target_os = "linux"), target_env = "musl"))]
    fn is_on_nfs_mount(_path: &Path) -> bool {
        false
    }

    if is_on_nfs_mount(path) {
        return Ok(());
    }

    match try() {
        Ok(_) => return Ok(()),
        #[cfg(target_os = "macos")]
        Err(ref e) if e.raw_os_error() == Some(::libc::ENOTSUP) =>
        {
            return Ok(())
        }
        Err(e) => if e.raw_os_error() != fs2::lock_contended_error().raw_os_error() {
            return Err(e);
        },
    }

    writeln!(
        io::stderr(),
        "{:>12} waiting for file lock on {}",
        "Blocking",
        msg
    ).ok();

    block()
}

fn create_dir_all(path: &Path) -> io::Result<()> {
    match create_dir(path) {
        Ok(()) => Ok(()),
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                if let Some(p) = path.parent() {
                    return create_dir_all(p).and_then(|()| create_dir(path));
                }
            }
            Err(e)
        }
    }
}

fn create_dir(path: &Path) -> io::Result<()> {
    match fs::create_dir(path) {
        Ok(()) => Ok(()),
        Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(e),
    }
}
