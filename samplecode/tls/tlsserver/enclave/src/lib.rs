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

#![crate_name = "tlsserver"]
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

use sgx_types::*;
use sgx_trts::trts::{rsgx_raw_is_outside_enclave, rsgx_lfence, rsgx_sfence};

use std::untrusted::fs;
use std::io::BufReader;

use std::ffi::CStr;
use std::os::raw::c_char;

use std::vec::Vec;
use std::boxed::Box;
use std::io::{Read, Write};
use std::slice;
use std::sync::{Arc, SgxRwLock};
use std::net::TcpStream;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, AtomicPtr, Ordering};

extern crate webpki;
extern crate rustls;
use rustls::{Session, NoClientAuth};

pub struct TlsServer {
    socket: TcpStream,
    tls_session: rustls::ServerSession,
}

static GLOBAL_CONTEXT_COUNT: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    static ref GLOBAL_CONTEXTS: SgxRwLock<HashMap<usize, AtomicPtr<TlsServer>>> = {
        SgxRwLock::new(HashMap::new())
    };
}

impl TlsServer {
    fn new(fd: c_int, cfg: Arc<rustls::ServerConfig>) -> TlsServer {
        TlsServer {
            socket: TcpStream::new(fd).unwrap(),
            tls_session: rustls::ServerSession::new(&cfg)
        }
    }

    fn do_read(&mut self) -> c_int {
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
        return 0;
    }

    fn read(&mut self, plaintext: &mut Vec<u8>) -> c_int {
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

fn load_certs(filename: &str) -> Vec<rustls::Certificate> {
    let certfile = fs::File::open(filename).expect("cannot open certificate file");
    let mut reader = BufReader::new(certfile);
    rustls::internal::pemfile::certs(&mut reader).unwrap()
}

fn load_private_key(filename: &str) -> rustls::PrivateKey {
    let rsa_keys = {
        let keyfile = fs::File::open(filename)
            .expect("cannot open private key file");
        let mut reader = BufReader::new(keyfile);
        rustls::internal::pemfile::rsa_private_keys(&mut reader)
            .expect("file contains invalid rsa private key")
    };

    let pkcs8_keys = {
        let keyfile = fs::File::open(filename)
            .expect("cannot open private key file");
        let mut reader = BufReader::new(keyfile);
        rustls::internal::pemfile::pkcs8_private_keys(&mut reader)
            .expect("file contains invalid pkcs8 private key (encrypted keys not supported)")
    };

    // prefer to load pkcs8 keys
    if !pkcs8_keys.is_empty() {
        pkcs8_keys[0].clone()
    } else {
        assert!(!rsa_keys.is_empty());
        rsa_keys[0].clone()
    }
}

fn make_config(cert: &str, key: &str) -> Arc<rustls::ServerConfig> {

    let mut config = rustls::ServerConfig::new(NoClientAuth::new());

    let certs = load_certs(cert);
    let privkey = load_private_key(key);
    config.set_single_cert_with_ocsp_and_sct(certs, privkey, vec![], vec![]).unwrap();

    Arc::new(config)
}

struct Sessions;

impl Sessions {
    fn new_session(svr_ptr : *mut TlsServer) -> Option<usize> {
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

    fn get_session(sess_id: size_t) -> Option<*mut TlsServer> {
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
                let _ = unsafe { Box::<TlsServer>::from_raw(session as *mut _) };
                let _ = gctxts.remove(&sess_id);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn tls_server_new(fd: c_int, cert: * const c_char, key: * const c_char) -> usize {
    if key.is_null() || cert.is_null() {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }

    let certfile = unsafe { CStr::from_ptr(cert).to_str() };
    if certfile.is_err() {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let keyfile = unsafe { CStr::from_ptr(key).to_str() };
    if keyfile.is_err() {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let config = make_config(certfile.unwrap(), keyfile.unwrap());

    let p: *mut TlsServer = Box::into_raw(Box::new(TlsServer::new(fd, config)));
    match Sessions::new_session(p) {
        Some(s) => s,
        None => 0xFFFF_FFFF_FFFF_FFFF,
    }
}

#[no_mangle]
pub extern "C" fn tls_server_read(session_id: size_t, buf: * mut c_char, cnt: c_int) -> c_int {
    if let Some(session_ptr) = Sessions::get_session(session_id) {
        let session = unsafe { &mut *(session_ptr) };
        if buf.is_null() || cnt == 0 {
            // just read_tls
            session.do_read()
        } else {
            if !rsgx_raw_is_outside_enclave(buf as * const u8, cnt as usize) {
                return -1;
            }
            // read plain buffer
            let mut plaintext = Vec::new();
            let mut result = session.read(&mut plaintext);
            if result == -1 {
                return result;
            }
            if cnt < result {
                result = cnt;
            }
            rsgx_sfence();
            let raw_buf = unsafe { slice::from_raw_parts_mut(buf as * mut u8, result as usize) };
            raw_buf.copy_from_slice(plaintext.as_slice());
            result
        }
    } else { -1 }
}

#[no_mangle]
pub extern "C" fn tls_server_write(session_id: usize, buf: * const c_char, cnt: c_int)  -> c_int {
    if let Some(session_ptr) = Sessions::get_session(session_id) {
        let session = unsafe { &mut *(session_ptr) };

        // no buffer, just write_tls.
        if buf.is_null() || cnt == 0 {
            session.do_write();
            return 0;
        }

        rsgx_lfence();
        // cache buffer, waitting for next write_tls
        let cnt = cnt as usize;
        let plaintext = unsafe { slice::from_raw_parts(buf as * mut u8, cnt) };
        let result = session.write(plaintext);

        result
    } else { -1 }
}

#[no_mangle]
pub extern "C" fn tls_server_wants_read(session_id: usize) -> c_int {
    if let Some(session_ptr) = Sessions::get_session(session_id) {
        let session = unsafe { &mut *(session_ptr) };
        let result = session.tls_session.wants_read() as c_int;
        result
    } else { -1 }
}

#[no_mangle]
pub extern "C" fn tls_server_wants_write(session_id: usize)  -> c_int {
    if let Some(session_ptr) = Sessions::get_session(session_id) {
        let session = unsafe { &mut *(session_ptr) };
        let result = session.tls_session.wants_write() as c_int;
        result
    } else { -1 }
}

#[no_mangle]
pub extern "C" fn tls_server_close(session_id: usize) {
    Sessions::remove_session(session_id)
}

#[no_mangle]
pub extern "C" fn tls_server_send_close(session_id: usize) {
    if let Some(session_ptr) = Sessions::get_session(session_id) {
        let session = unsafe { &mut *session_ptr };
        session.tls_session.send_close_notify();
    }
}
