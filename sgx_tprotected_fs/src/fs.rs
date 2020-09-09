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

//! # Intel Protected File System API
use core::cmp;
use sgx_trts::c_str::CStr;
use sgx_trts::error::errno;
use sgx_trts::libc::{self, c_void};
use sgx_types::*;

fn max_len() -> usize {
    u32::MAX as usize
}

unsafe fn rsgx_fopen(filename: &CStr, mode: &CStr, key: &sgx_key_128bit_t) -> SysResult<SGX_FILE> {
    let file = sgx_fopen(
        filename.as_ptr(),
        mode.as_ptr(),
        key as *const sgx_key_128bit_t,
    );
    if file.is_null() {
        Err(errno())
    } else {
        Ok(file)
    }
}

unsafe fn rsgx_fopen_auto_key(filename: &CStr, mode: &CStr) -> SysResult<SGX_FILE> {
    let file = sgx_fopen_auto_key(filename.as_ptr(), mode.as_ptr());
    if file.is_null() {
        Err(errno())
    } else {
        Ok(file)
    }
}

unsafe fn rsgx_fwrite(stream: SGX_FILE, buf: &[u8]) -> SysResult<usize> {
    if stream.is_null() {
        return Err(libc::EINVAL);
    }

    let write_size = cmp::min(buf.len(), max_len());
    let ret_size = sgx_fwrite(buf.as_ptr() as *const c_void, 1, write_size, stream);
    if ret_size != write_size {
        Err(rsgx_ferror(stream))
    } else {
        Ok(ret_size)
    }
}

unsafe fn rsgx_fread(stream: SGX_FILE, buf: &mut [u8]) -> SysResult<usize> {
    if stream.is_null() {
        return Err(libc::EINVAL);
    }

    let read_size = cmp::min(buf.len(), max_len());
    let ret_size = sgx_fread(buf.as_mut_ptr() as *mut c_void, 1, read_size, stream);

    if ret_size != read_size {
        let is_eof = rsgx_feof(stream)?;
        if is_eof {
            Ok(ret_size)
        } else {
            Err(rsgx_ferror(stream))
        }
    } else {
        Ok(ret_size)
    }
}

unsafe fn rsgx_ftell(stream: SGX_FILE) -> SysResult<i64> {
    if stream.is_null() {
        return Err(libc::EINVAL);
    }

    let pos = sgx_ftell(stream);
    if pos == -1 {
        Err(errno())
    } else {
        Ok(pos)
    }
}

unsafe fn rsgx_fseek(stream: SGX_FILE, offset: i64, origin: i32) -> SysError {
    if stream.is_null() {
        return Err(libc::EINVAL);
    }

    let ret = sgx_fseek(stream, offset, origin);
    if ret == 0 {
        Ok(())
    } else {
        Err(rsgx_ferror(stream))
    }
}

unsafe fn rsgx_fflush(stream: SGX_FILE) -> SysError {
    let ret = sgx_fflush(stream);
    if ret == 0 {
        Ok(())
    } else if ret == libc::EOF {
        Err(rsgx_ferror(stream))
    } else {
        Err(ret)
    }
}

unsafe fn rsgx_ferror(stream: SGX_FILE) -> i32 {
    let mut err = sgx_ferror(stream);
    if err == -1 {
        err = libc::EINVAL;
    }
    err
}

unsafe fn rsgx_feof(stream: SGX_FILE) -> SysResult<bool> {
    if stream.is_null() {
        return Err(libc::EINVAL);
    }

    if sgx_feof(stream) == 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

unsafe fn rsgx_clearerr(stream: SGX_FILE) {
    sgx_clearerr(stream)
}

unsafe fn rsgx_fclose(stream: SGX_FILE) -> SysError {
    if stream.is_null() {
        return Err(libc::EINVAL);
    }

    let ret = sgx_fclose(stream);
    if ret == 0 {
        Ok(())
    } else {
        Err(libc::EIO)
    }
}

unsafe fn rsgx_fclear_cache(stream: SGX_FILE) -> SysError {
    if stream.is_null() {
        return Err(libc::EINVAL);
    }

    let ret = sgx_fclear_cache(stream);
    if ret == 0 {
        Ok(())
    } else {
        Err(rsgx_ferror(stream))
    }
}

unsafe fn rsgx_remove(filename: &CStr) -> SysError {
    let ret = sgx_remove(filename.as_ptr());
    if ret == 0 {
        Ok(())
    } else {
        Err(errno())
    }
}

unsafe fn rsgx_fexport_auto_key(filename: &CStr, key: &mut sgx_key_128bit_t) -> SysError {
    let ret = sgx_fexport_auto_key(filename.as_ptr(), key as *mut sgx_key_128bit_t);
    if ret == 0 {
        Ok(())
    } else {
        Err(errno())
    }
}

unsafe fn rsgx_fimport_auto_key(filename: &CStr, key: &sgx_key_128bit_t) -> SysError {
    let ret = sgx_fimport_auto_key(filename.as_ptr(), key as *const sgx_key_128bit_t);
    if ret == 0 {
        Ok(())
    } else {
        Err(errno())
    }
}

pub struct SgxFileStream {
    stream: SGX_FILE,
}

impl SgxFileStream {
    ///
    /// The open function creates or opens a protected file.
    ///
    /// # Description
    ///
    /// open is similar to the C file API fopen. It creates a new Protected File or opens an existing
    /// Protected File created with a previous call to open. Regular files cannot be opened with this API.
    ///
    /// # Parameters
    ///
    /// **filename**
    ///
    /// The name of the file to be created or opened.
    ///
    /// **mode**
    ///
    /// The file open mode string. Allowed values are any combination of ‘r’, ‘w’ or ‘a’, with possible ‘+’
    /// and possible ‘b’ (since string functions are currently not sup- ported, ‘b’ is meaningless).
    ///
    /// **key**
    ///
    /// The encryption key of the file. This key is used as a key derivation key, used for deriving encryption
    /// keys for the file. If the file is created with open, you should protect this key and provide it as
    /// input every time the file is opened.
    ///
    /// # Requirements
    ///
    /// Header: sgx_tprotected_fs.edl
    ///
    /// Library: libsgx_tprotected_fs.a
    ///
    /// # Return value
    ///
    /// If the function succeeds, it returns a valid file pointer, which can be used by all the other functions
    /// in the Protected FS API, otherwise, error code is returned.
    ///
    pub fn open(filename: &CStr, mode: &CStr, key: &sgx_key_128bit_t) -> SysResult<SgxFileStream> {
        unsafe { rsgx_fopen(filename, mode, key).map(|f| SgxFileStream { stream: f }) }
    }

    ///
    /// The open_auto_key function creates or opens a protected file.
    ///
    /// # Description
    ///
    /// open_auto_key is similar to the C file API fopen. It creates a new Protected File or opens an existing
    /// Protected File created with a previous call to open_auto_key. Regular files cannot be opened with this API.
    ///
    /// # Parameters
    ///
    /// **filename**
    ///
    /// The name of the file to be created or opened.
    ///
    /// **mode**
    ///
    /// The file open mode string. Allowed values are any combination of ‘r’, ‘w’ or ‘a’, with possible ‘+’
    /// and possible ‘b’ (since string functions are currently not sup- ported, ‘b’ is meaningless).
    ///
    /// # Requirements
    ///
    /// Header: sgx_tprotected_fs.edl
    ///
    /// Library: libsgx_tprotected_fs.a
    ///
    /// # Return value
    ///
    /// If the function succeeds, it returns a valid file pointer, which can be used by all the other functions
    /// in the Protected FS API, otherwise, error code is returned.
    ///
    pub fn open_auto_key(filename: &CStr, mode: &CStr) -> SysResult<SgxFileStream> {
        unsafe { rsgx_fopen_auto_key(filename, mode).map(|f| SgxFileStream { stream: f }) }
    }

    ///
    /// The read function reads the requested amount of data from the file, and extends the file pointer by that amount.
    ///
    /// # Description
    ///
    /// read is similar to the file API fread. In case of an error, error can be called to get the error code.
    ///
    /// # Parameters
    ///
    /// **buf**
    ///
    /// A pointer to a buffer to receive the data read from the file.
    ///
    /// # Requirements
    ///
    /// Header: sgx_tprotected_fs.edl
    ///
    /// Library: libsgx_tprotected_fs.a
    ///
    /// # Return value
    ///
    /// If the function succeeds, the number of bytes read is returned (zero indicates end of file).
    /// otherwise, error code is returned.
    ///
    pub fn read(&self, buf: &mut [u8]) -> SysResult<usize> {
        unsafe { rsgx_fread(self.stream, buf) }
    }

    ///
    /// The write function writes the given amount of data to the file, and extends the file pointer by that amount.
    ///
    /// # Description
    ///
    /// write is similar to the file API fwrite. In case of an error, error can be called to get the error code.
    ///
    /// # Parameters
    ///
    /// **buf**
    ///
    /// A pointer to a buffer, that contains the data to write to the file.
    ///
    /// # Requirements
    ///
    /// Header: sgx_tprotected_fs.edl
    ///
    /// Library: libsgx_tprotected_fs.a
    ///
    /// # Return value
    ///
    /// If the function succeeds, the number of bytes written is returned (zero indicates nothing was written).
    /// otherwise, error code is returned.
    ///
    pub fn write(&self, buf: &[u8]) -> SysResult<usize> {
        unsafe { rsgx_fwrite(self.stream, buf) }
    }

    ///
    /// The tell function obtains the current value of the file position indicator for the stream pointed to by stream.
    ///
    /// # Description
    ///
    /// tell is similar to the C file API ftell.
    ///
    /// # Requirements
    ///
    /// Header: sgx_tprotected_fs.edl
    ///
    /// Library: libsgx_tprotected_fs.a
    ///
    /// # Return value
    ///
    /// If the function succeeds, it returns the current value of the position indicator of the file.
    /// otherwise, error code is returned.
    ///
    pub fn tell(&self) -> SysResult<i64> {
        unsafe { rsgx_ftell(self.stream) }
    }

    ///
    /// The seek function sets the current value of the position indicator of the file.
    ///
    /// # Description
    ///
    /// seek is similar to the C file API fseek.
    ///
    /// # Parameters
    ///
    /// **offset**
    ///
    /// The new required value, relative to the origin parameter.
    ///
    /// **origin**
    ///
    /// The origin from which to calculate the offset (Start, ENd or Current).
    ///
    /// # Requirements
    ///
    /// Header: sgx_tprotected_fs.edl
    ///
    /// Library: libsgx_tprotected_fs.a
    ///
    /// # Return value
    ///
    /// If the function failed, error code is returned.
    ///
    pub fn seek(&self, offset: i64, origin: SeekFrom) -> SysError {
        let whence = match origin {
            SeekFrom::Start => libc::SEEK_SET,
            SeekFrom::End => libc::SEEK_END,
            SeekFrom::Current => libc::SEEK_CUR,
        };
        unsafe { rsgx_fseek(self.stream, offset, whence) }
    }

    ///
    /// The flush function forces a cache flush, and if it returns successfully, it is guaranteed
    /// that your changes are committed to a file on the disk.
    ///
    /// # Description
    ///
    /// flush is similar to the C file API fflush. This function flushes all the modified data from
    /// the cache and writes it to a file on the disk. In case of an error, error can be called to
    /// get the error code. Note that this function does not clear the cache, but only flushes the
    /// changes to the actual file on the disk. Flushing also happens automatically when the cache
    /// is full and page eviction is required.
    ///
    /// # Requirements
    ///
    /// Header: sgx_tprotected_fs.edl
    ///
    /// Library: libsgx_tprotected_fs.a
    ///
    /// # Return value
    ///
    /// If the function failed, error code is returned.
    ///
    pub fn flush(&self) -> SysError {
        unsafe { rsgx_fflush(self.stream) }
    }

    ///
    /// The error function returns the latest operation error code.
    ///
    /// # Description
    ///
    /// error is similar to the C file API ferror. In case the latest operation failed because
    /// the file is in a bad state, SGX_ERROR_FILE_BAD_STATUS will be returned.
    ///
    /// # Requirements
    ///
    /// Header: sgx_tprotected_fs.edl
    ///
    /// Library: libsgx_tprotected_fs.a
    ///
    /// # Return value
    ///
    /// The latest operation error code is returned. 0 indicates that no errors occurred.
    ///
    pub fn error(&self) -> i32 {
        unsafe { rsgx_ferror(self.stream) }
    }

    ///
    /// The is_eof function tells the caller if the file's position indicator hit the end of the
    /// file in a previous read operation.
    ///
    /// # Description
    ///
    /// is_eof is similar to the C file API feof.
    ///
    /// # Requirements
    ///
    /// Header: sgx_tprotected_fs.edl
    ///
    /// Library: libsgx_tprotected_fs.a
    ///
    /// # Return value
    ///
    /// true - End of file was not reached. false - End of file was reached.
    ///
    pub fn is_eof(&self) -> bool {
        unsafe { rsgx_feof(self.stream).unwrap() }
    }

    ///
    /// The clearerr function attempts to repair a bad file status, and also clears the end-of-file flag.
    ///
    /// # Description
    ///
    /// clearerr is similar to the C file API clearerr. This function attempts to repair errors resulted
    /// from the underlying file system, like write errors to the disk (resulting in a full cache thats
    /// cannot be emptied). Call error or is_eof after a call to this function to learn if it was successful
    /// or not.
    ///
    /// clearerr does not repair errors resulting from a corrupted file, like decryption errors, or from
    /// memory corruption, etc.
    ///
    /// # Requirements
    ///
    /// Header: sgx_tprotected_fs.edl
    ///
    /// Library: libsgx_tprotected_fs.a
    ///
    /// # Return value
    ///
    /// None
    ///
    pub fn clearerr(&self) {
        unsafe { rsgx_clearerr(self.stream) }
    }

    ///
    /// The clear_cache function is used for clearing the internal file cache. The function scrubs all
    /// the data from the cache, and releases all the allocated cache memory.
    ///
    /// # Description
    ///
    /// clear_cache is used to scrub all the data from the cache and release all the allocated cache
    /// memory. If modified data is found in the cache, it will be written to the file on disk before
    /// being scrubbed.
    ///
    /// This function is especially useful if you do not trust parts of your own enclave (for example,
    /// external libraries you linked against, etc.) and want to make sure there is as little sensitive
    /// data in the memory as possible before transferring control to the code they do not trust.
    /// Note, however, that the SGX_FILE structure itself still holds sensitive data. To remove all
    /// such data related to the file from memory completely, you should close the file handle.
    ///
    /// # Requirements
    ///
    /// Header: sgx_tprotected_fs.edl
    ///
    /// Library: libsgx_tprotected_fs.a
    ///
    /// # Return value
    ///
    /// If the function failed, error code is returned.
    ///
    pub fn clear_cache(&self) -> SysError {
        unsafe { rsgx_fclear_cache(self.stream) }
    }
}

///
/// The remove function deletes a file from the file system.
///
/// # Description
///
/// remove is similar to the C file API remove.
///
/// # Parameters
///
/// **filename**
///
/// The name of the file to delete.
///
/// # Requirements
///
/// Header: sgx_tprotected_fs.edl
///
/// Library: libsgx_tprotected_fs.a
///
/// # Return value
///
/// If the function failed, error code is returned.
///
pub fn remove(filename: &CStr) -> SysError {
    unsafe { rsgx_remove(filename) }
}

///
/// The export_auto_key function is used for exporting the latest key used for the file encryption.
///
/// # Description
///
/// export_auto_key is used to export the last key that was used in the encryption of the file.
/// With this key you can import the file in a different enclave or system.
///
/// # Parameters
///
/// **filename**
///
/// The name of the file to be exported. This should be the name of a file created with the open_auto_key API.
///
/// # Requirements
///
/// Header: sgx_tprotected_fs.edl
///
/// Library: libsgx_tprotected_fs.a
///
/// # Return value
///
/// If the function succeeds, it returns the latest encryption key.
/// otherwise, error code is returned.
///
pub fn export_auto_key(filename: &CStr) -> SysResult<sgx_key_128bit_t> {
    let mut key: sgx_key_128bit_t = Default::default();
    unsafe { rsgx_fexport_auto_key(filename, &mut key).map(|_| key) }
}

pub fn export_align_auto_key(filename: &CStr) -> SysResult<sgx_align_key_128bit_t> {
    let mut align_key: sgx_align_key_128bit_t = Default::default();
    unsafe { rsgx_fexport_auto_key(filename, &mut align_key.key).map(|_| align_key) }
}

///
/// The import_auto_key function is used for importing a Protected FS auto key file created on
/// a different enclave or platform.
///
/// # Description
///
/// import_auto_key is used for importing a Protected FS file. After this call returns successfully,
/// the file can be opened normally with export_auto_key.
///
/// # Parameters
///
/// **filename**
///
/// The name of the file to be imported. This should be the name of a file created with the
/// open_auto_key API, on a different enclave or system.
///
/// **key**
///
/// he encryption key, exported with a call to export_auto_key in the source enclave or system.
///
/// # Requirements
///
/// Header: sgx_tprotected_fs.edl
///
/// Library: libsgx_tprotected_fs.a
///
/// # Return value
///
/// otherwise, error code is returned.
///
pub fn import_auto_key(filename: &CStr, key: &sgx_key_128bit_t) -> SysError {
    unsafe { rsgx_fimport_auto_key(filename, key) }
}

impl Drop for SgxFileStream {
    fn drop(&mut self) {
        // Note that errors are ignored when closing a file descriptor. The
        // reason for this is that if an error occurs we don't actually know if
        // the file descriptor was closed or not, and if we retried (for
        // something like EINTR), we might close another valid file descriptor
        // (opened after we closed ours.
        let _ = unsafe { rsgx_fclose(self.stream) };
    }
}

/// Enumeration of possible methods to seek within an I/O object.
#[derive(Copy, PartialEq, Eq, Clone, Debug)]
pub enum SeekFrom {
    /// Set the offset to the provided number of bytes.
    Start,

    /// Set the offset to the size of this object plus the specified number of
    /// bytes.
    ///
    /// It is possible to seek beyond the end of an object, but it's an error to
    /// seek before byte 0.
    End,

    /// Set the offset to the current position plus the specified number of
    /// bytes.
    ///
    /// It is possible to seek beyond the end of an object, but it's an error to
    /// seek before byte 0.
    Current,
}
