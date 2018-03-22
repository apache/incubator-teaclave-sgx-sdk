// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
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

#![crate_name = "tlsclient"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
extern crate sgx_trts;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

use sgx_trts::trts::{rsgx_raw_is_outside_enclave, rsgx_lfence};

use sgx_types::*;
use std::collections;
use std::mem;

use std::untrusted::fs;
use std::io::BufReader;

use std::ffi::CStr;
use std::os::raw::c_char;

use std::ptr;
use std::string::String;
use std::vec::Vec;
use std::boxed::Box;
use std::io::{Read, Write};
use std::slice;
use std::sync::{Arc, SgxMutex};
use std::net::TcpStream;

extern crate webpki;
extern crate rustls;
use rustls::Session;

pub struct TlsClient {
    socket: TcpStream,
    tls_session:  rustls::ClientSession,
}

impl TlsClient {
    fn new(fd: c_int, hostname: &str, cfg: Arc<rustls::ClientConfig>) -> TlsClient {
        TlsClient {
            socket: TcpStream::new(fd).unwrap(),
            tls_session: rustls::ClientSession::new(&cfg, webpki::DNSNameRef::try_from_ascii_str(hostname).unwrap())
        }
    }

    fn do_read(&mut self, plaintext: &mut Vec<u8>) -> c_int {
        // Read TLS data.  This fails if the underlying TCP connection
        // is broken.
        let rc = self.tls_session.read_tls(&mut self.socket);
        if rc.is_err() {
            println!("TLS read error: {:?}", rc);
            return -1;
        }

        // If we're ready but there's no data: EOF.
        if rc.unwrap() == 0 {
            println!("EOF");
            return -1;
        }

        // Reading some TLS data might have yielded new TLS
        // messages to process.  Errors from this indicate
        // TLS protocol problems and are fatal.
        let processed = self.tls_session.process_new_packets();
        if processed.is_err() {
            println!("TLS error: {:?}", processed.unwrap_err());
            return -1;
        }

        // Having read some TLS data, and processed any new messages,
        // we might have new plaintext as a result.
        //
        // Read it and then write it to stdout.
        let rc = self.tls_session.read_to_end(plaintext);

        // If that fails, the peer might have started a clean TLS-level
        // session closure.
        if rc.is_err() {
            let err = rc.unwrap_err();
            println!("Plaintext read error: {:?}", err);
            return -1;
        }
        plaintext.len() as c_int
    }

    // fn is_traffic(&self) -> bool {
    //     !self.tls_session.is_handshaking()
    // }

    fn write(&mut self, plaintext: &[u8]) -> c_int{
        self.tls_session.write(plaintext).unwrap() as c_int
    }

    fn do_write(&mut self) {
        self.tls_session.write_tls(&mut self.socket).unwrap();
    }
}

/// This is an example cache for client session data.
/// It optionally dumps cached data to a file, but otherwise
/// is just in-memory.
///
/// Note that the contents of such a file are extremely sensitive.
/// Don't write this stuff to disk in production code.
struct PersistCache {
    cache: SgxMutex<collections::HashMap<Vec<u8>, Vec<u8>>>,
    filename: Option<String>,
}

impl PersistCache {
    /// Make a new cache.  If filename is Some, load the cache
    /// from it and flush changes back to that file.
    fn new(filename: &Option<String>) -> PersistCache {
        let cache = PersistCache {
            cache: SgxMutex::new(collections::HashMap::new()),
            filename: filename.clone(),
        };
        if cache.filename.is_some() {
            cache.load();
        }
        cache
    }

    /// If we have a filename, save the cache contents to it.
    fn save(&self) {
        use rustls::internal::msgs::codec::Codec;
        use rustls::internal::msgs::base::PayloadU16;

        if self.filename.is_none() {
            return;
        }

        let mut file = fs::File::create(self.filename.as_ref().unwrap())
            .expect("cannot open cache file");

        for (key, val) in self.cache.lock().unwrap().iter() {
            let mut item = Vec::new();
            let key_pl = PayloadU16::new(key.clone());
            let val_pl = PayloadU16::new(val.clone());
            key_pl.encode(&mut item);
            val_pl.encode(&mut item);
            file.write_all(&item).unwrap();
        }
    }

    /// We have a filename, so replace the cache contents from it.
    fn load(&self) {
        use rustls::internal::msgs::codec::{Codec, Reader};
        use rustls::internal::msgs::base::PayloadU16;

        let mut file = match fs::File::open(self.filename.as_ref().unwrap()) {
            Ok(f) => f,
            Err(_) => return,
        };
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        let mut cache = self.cache.lock()
            .unwrap();
        cache.clear();
        let mut rd = Reader::init(&data);

        while rd.any_left() {
            let key_pl = PayloadU16::read(&mut rd).unwrap();
            let val_pl = PayloadU16::read(&mut rd).unwrap();
            cache.insert(key_pl.0, val_pl.0);
        }
    }
}

impl rustls::StoresClientSessions for PersistCache {
    /// put: insert into in-memory cache, and perhaps persist to disk.
    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> bool {
        self.cache.lock()
            .unwrap()
            .insert(key, value);
        self.save();
        true
    }

    /// get: from in-memory cache
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.cache.lock()
            .unwrap()
            .get(key).cloned()
    }
}

/// Build a `ClientConfig` from our arguments
fn make_config(cert: &str) -> Arc<rustls::ClientConfig> {
    let mut config = rustls::ClientConfig::new();

    let certfile = fs::File::open(cert).expect("Cannot open CA file");
    let mut reader = BufReader::new(certfile);
    config.root_store
        .add_pem_file(&mut reader)
        .unwrap();

    let cache = Option::None;
    let persist = Arc::new(PersistCache::new(&cache));
    config.set_persistence(persist);

    Arc::new(config)
}


#[no_mangle]
pub extern "C" fn tls_client_new(fd: c_int, hostname: * const c_char, cert: * const c_char) ->  *const c_void {
    let certfile = unsafe { CStr::from_ptr(cert).to_str() };
    if certfile.is_err() {
        return ptr::null();
    }
    let config = make_config(certfile.unwrap());
    let name = unsafe { CStr::from_ptr(hostname).to_str() };
    let name = match name {
        Ok(n) => n,
        Err(_) => {
            return ptr::null();
        }
    };
    Box::into_raw(Box::new(TlsClient::new(fd, name, config))) as *const c_void
}

#[no_mangle]
pub extern "C" fn tls_client_read(session: *const c_void, buf: * mut c_char, cnt: c_int) -> c_int {
    if session.is_null() {
        return -1;
    }

    if rsgx_raw_is_outside_enclave(session as * const u8, mem::size_of::<TlsClient>()) {
        return -1;
    }
    rsgx_lfence();

    if buf.is_null() {
        return -1;
    }

    let session= unsafe { &mut *(session as *mut TlsClient) };

    let mut plaintext = Vec::new();
    let mut result = session.do_read(&mut plaintext);

    if result == -1 {
        return result;
    }
    if cnt < result {
        result = cnt;
    }

    let raw_buf = unsafe { slice::from_raw_parts_mut(buf as * mut u8, result as usize) };
    raw_buf.copy_from_slice(plaintext.as_slice());
    result
}

#[no_mangle]
pub extern "C" fn tls_client_write(session: *const c_void, buf: * const c_char, cnt: c_int)  -> c_int {
    if session.is_null() {
        return -1;
    }

    if rsgx_raw_is_outside_enclave(session as * const u8, mem::size_of::<TlsClient>()) {
        return -1;
    }
    rsgx_lfence();

    let session= unsafe { &mut *(session as *mut TlsClient) };

    // no buffer, just write_tls.
    if buf.is_null() || cnt == 0 {
        session.do_write();
        0
    } else {
        let cnt = cnt as usize;
        let plaintext = unsafe { slice::from_raw_parts(buf as * mut u8, cnt) };
        let result = session.write(plaintext);

        result
    }    
}

#[no_mangle]
pub extern "C" fn tls_client_wants_read(session: *const c_void)  -> c_int {
    if session.is_null() {
        return -1;
    }

    if rsgx_raw_is_outside_enclave(session as * const u8, mem::size_of::<TlsClient>()) {
        return -1;
    }
    rsgx_lfence();

    let session= unsafe { &mut *(session as *mut TlsClient) };
    let result = session.tls_session.wants_read() as c_int;
    result
}

#[no_mangle]
pub extern "C" fn tls_client_wants_write(session: *const c_void)  -> c_int {
    if session.is_null() {
        return -1;
    }

    if rsgx_raw_is_outside_enclave(session as * const u8, mem::size_of::<TlsClient>()) {
        return -1;
    }
    rsgx_lfence();

    let session= unsafe { &mut *(session as *mut TlsClient) };
    let result = session.tls_session.wants_write() as c_int;
    result
}

#[no_mangle]
pub extern "C" fn tls_client_close(session: * const c_void) {
    if !session.is_null() {

        if rsgx_raw_is_outside_enclave(session as * const u8, mem::size_of::<TlsClient>()) {
            return;
        }
        rsgx_lfence();

        let _ = unsafe { Box::<TlsClient>::from_raw(session as *mut _) };
    }
}
