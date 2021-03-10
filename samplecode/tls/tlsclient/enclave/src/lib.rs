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

#![crate_name = "tlsclient"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
extern crate sgx_trts;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

#[macro_use]
extern crate lazy_static;

use sgx_trts::trts::{rsgx_lfence, rsgx_sfence};

use sgx_types::*;
use std::collections;

use std::untrusted::fs;
use std::io::BufReader;

use std::ffi::CStr;
use std::os::raw::c_char;

use std::string::String;
use std::vec::Vec;
use std::boxed::Box;
use std::io::{Read, Write};
use std::slice;
use std::sync::{Arc, SgxMutex, SgxRwLock};
use std::net::TcpStream;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, AtomicPtr, Ordering};

extern crate webpki;
extern crate rustls;
use rustls::Session;

pub struct TlsClient {
    socket: TcpStream,
    tls_session:  rustls::ClientSession,
}

static GLOBAL_CONTEXT_COUNT: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    static ref GLOBAL_CONTEXTS: SgxRwLock<HashMap<usize, AtomicPtr<TlsClient>>> = {
        SgxRwLock::new(HashMap::new())
    };
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

struct Sessions;

impl Sessions {
    fn new_session(svr_ptr : *mut TlsClient) -> Option<usize> {
        match GLOBAL_CONTEXTS.write() {
            Ok(mut gctxts) => {
                let curr_id = GLOBAL_CONTEXT_COUNT.fetch_add(1, Ordering::SeqCst);
                gctxts.insert(curr_id, AtomicPtr::new(svr_ptr));
                Some(curr_id)
            },
            Err(x) => {
                println!("Locking global context SgxRwLock failed! {:?}", x);
                None
            },
        }
    }

    fn get_session(sess_id: size_t) -> Option<*mut TlsClient> {
        match GLOBAL_CONTEXTS.read() {
            Ok(gctxts) => {
                match gctxts.get(&sess_id) {
                    Some(s) => {
                        Some(s.load(Ordering::SeqCst))
                    },
                    None => {
                        println!("Global contexts cannot find session id = {}", sess_id);
                        None
                    }
                }
            },
            Err(x) => {
                println!("Locking global context SgxRwLock failed on get_session! {:?}", x);
                None
            },
        }
    }

    fn remove_session(sess_id: size_t) {
        if let Ok(mut gctxts) = GLOBAL_CONTEXTS.write() {
            if let Some(session_ptr) = gctxts.get(&sess_id) {
                let session_ptr = session_ptr.load(Ordering::SeqCst);
                let session = unsafe { &mut *session_ptr };
                let _ = unsafe { Box::<TlsClient>::from_raw(session as *mut _) };
                let _ = gctxts.remove(&sess_id);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn tls_client_new(fd: c_int, hostname: * const c_char, cert: * const c_char) -> usize {
    if hostname.is_null() || cert.is_null() {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }

    let certfile = unsafe { CStr::from_ptr(cert).to_str() };
    if certfile.is_err() {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let config = make_config(certfile.unwrap());
    let name = unsafe { CStr::from_ptr(hostname).to_str() };
    let name = match name {
        Ok(n) => n,
        Err(_) => {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
    };
    let p: *mut TlsClient = Box::into_raw(Box::new(TlsClient::new(fd, name, config)));
    match Sessions::new_session(p) {
        Some(s) => s,
        None => 0xFFFF_FFFF_FFFF_FFFF,
    }
}

#[no_mangle]
pub extern "C" fn tls_client_read(session_id: usize, buf: * mut c_char, cnt: c_int) -> c_int {
    if buf.is_null() {
        return -1;
    }

    rsgx_sfence();

    if let Some(session_ptr) = Sessions::get_session(session_id) {
        let session= unsafe { &mut *session_ptr };

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
    } else { -1 }
}

#[no_mangle]
pub extern "C" fn tls_client_write(session_id: usize, buf: * const c_char, cnt: c_int)  -> c_int {
    if let Some(session_ptr) = Sessions::get_session(session_id) {
        let session = unsafe { &mut *session_ptr };

        // no buffer, just write_tls.
        if buf.is_null() || cnt == 0 {
            session.do_write();
            0
        } else {
            rsgx_lfence();
            let cnt = cnt as usize;
            let plaintext = unsafe { slice::from_raw_parts(buf as * mut u8, cnt) };
            let result = session.write(plaintext);

            result
        }
    } else { -1 }
}

#[no_mangle]
pub extern "C" fn tls_client_wants_read(session_id: usize)  -> c_int {
    if let Some(session_ptr) = Sessions::get_session(session_id) {
        let session= unsafe { &mut *session_ptr };
        let result = session.tls_session.wants_read() as c_int;
        result
    } else { -1 }
}

#[no_mangle]
pub extern "C" fn tls_client_wants_write(session_id: usize)  -> c_int {
    if let Some(session_ptr) = Sessions::get_session(session_id) {
        let session= unsafe { &mut *session_ptr };
        let result = session.tls_session.wants_write() as c_int;
        result
    } else { -1 }
}

#[no_mangle]
pub extern "C" fn tls_client_close(session_id: usize) {
    Sessions::remove_session(session_id)
}
