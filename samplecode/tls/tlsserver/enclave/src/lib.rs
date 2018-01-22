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

#![crate_name = "tlsserver"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
extern crate sgx_trts;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

use sgx_types::*;

use std::fs;
use std::io::BufReader;

use std::ffi::CStr;
use std::os::raw::c_char;

use std::ptr;
use std::vec::Vec;
use std::boxed::Box;
use std::io::{Read, Write};
use std::slice;
use std::sync::Arc;
use std::net::TcpStream;

extern crate webpki;
extern crate rustls;
use rustls::{Session, NoClientAuth};

pub struct TlsServer {
    socket: TcpStream,
    tls_session: rustls::ServerSession,
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
    config.set_single_cert_with_ocsp_and_sct(certs, privkey, vec![], vec![]);

    Arc::new(config)
}

#[no_mangle]
pub extern "C" fn tls_server_new(fd: c_int, cert: * const c_char, key: * const c_char) ->  *const c_void {
    let certfile = unsafe { CStr::from_ptr(cert).to_str() };
    if certfile.is_err() {
        return ptr::null();
    }
    let keyfile = unsafe { CStr::from_ptr(key).to_str() };
    if keyfile.is_err() {
        return ptr::null();
    }
    let config = make_config(certfile.unwrap(), keyfile.unwrap());

    Box::into_raw(Box::new(TlsServer::new(fd, config))) as  *const c_void
}

#[no_mangle]
pub extern "C" fn tls_server_read(session: *const c_void, buf: * mut c_char, cnt: c_int) -> c_int {
    if session.is_null() {
        return -1;
    }

    let session = unsafe { &mut *(session as *mut TlsServer) };

    if buf.is_null() || cnt == 0 {
        // just read_tls
        session.do_read()
    } else {
        // read plain buffer
        let mut plaintext = Vec::new();
        let mut result = session.read(&mut plaintext);
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
}

#[no_mangle]
pub extern "C" fn tls_server_write(session: *const c_void, buf: * const c_char, cnt: c_int)  -> c_int {
    if session.is_null() {
        return -1;
    }

    let session = unsafe { &mut *(session as *mut TlsServer) };

    // no buffer, just write_tls.
    if buf.is_null() || cnt == 0 {
        session.do_write();
        return 0;
    }

    // cache buffer, waitting for next write_tls
    let cnt = cnt as usize;
    let plaintext = unsafe { slice::from_raw_parts(buf as * mut u8, cnt) };
    let result = session.write(plaintext);

    result
}

#[no_mangle]
pub extern "C" fn tls_server_wants_read(session: *const c_void)  -> c_int {
    if session.is_null() {
        return -1;
    }
    let session = unsafe { &mut *(session as *mut TlsServer) };
    let result = session.tls_session.wants_read() as c_int;
    result
}

#[no_mangle]
pub extern "C" fn tls_server_wants_write(session: *const c_void)  -> c_int {
    if session.is_null() {
        return -1;
    }
    let session = unsafe { &mut *(session as *mut TlsServer) };
    let result = session.tls_session.wants_write() as c_int;
    result
}

#[no_mangle]
pub extern "C" fn tls_server_close(session: * const c_void) {
    if !session.is_null() {
        let _ = unsafe { Box::<TlsServer>::from_raw(session as *mut _) };
    }
}

#[no_mangle]
pub extern "C" fn tls_server_send_close(session: * const c_void) {
    if !session.is_null() {
        let session = unsafe { &mut *(session as *mut TlsServer) };
        session.tls_session.send_close_notify();
    }
}