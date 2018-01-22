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

#![allow(deprecated)]

extern crate sgx_types;
extern crate sgx_urts;
use sgx_types::*;
use sgx_urts::SgxEnclave;

extern crate mio;
use mio::tcp::TcpStream;

use std::os::unix::io::AsRawFd;
use std::ffi::CString;
use std::fs;
use std::path;
use std::env;
use std::net::SocketAddr;
use std::str;
use std::ptr;
use std::io::{self, Read, Write};

const BUFFER_SIZE: usize = 1024;

static ENCLAVE_FILE: &'static str = "enclave.signed.so";
static ENCLAVE_TOKEN: &'static str = "enclave.token";

extern {
    fn tls_client_new(eid: sgx_enclave_id_t, retval: *mut *const c_void,
                     fd: c_int, hostname: *const c_char, cert: *const c_char) -> sgx_status_t;
    fn tls_client_read(eid: sgx_enclave_id_t, retval: *mut c_int,
                     session: *const c_void, buf: *mut c_void, cnt: c_int) -> sgx_status_t;
    fn tls_client_write(eid: sgx_enclave_id_t, retval: *mut c_int,
                     session: *const c_void, buf: *const c_void, cnt: c_int) -> sgx_status_t;
    fn tls_client_wants_read(eid: sgx_enclave_id_t, retval: *mut c_int,
                     session: *const c_void) -> sgx_status_t;
    fn tls_client_wants_write(eid: sgx_enclave_id_t, retval: *mut c_int,
                     session: *const c_void) -> sgx_status_t;
    fn tls_client_close(eid: sgx_enclave_id_t,
                     session: *const c_void) -> sgx_status_t;
}

fn init_enclave() -> SgxResult<SgxEnclave> {

    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // Step 1: try to retrieve the launch token saved by last transaction
    //         if there is no token, then create a new one.
    //
    // try to get the token saved in $HOME */
    let mut home_dir = path::PathBuf::new();
    let use_token = match env::home_dir() {
        Some(path) => {
            println!("[+] Home dir is {}", path.display());
            home_dir = path;
            true
        },
        None => {
            println!("[-] Cannot get home dir");
            false
        }
    };

    let token_file: path::PathBuf = home_dir.join(ENCLAVE_TOKEN);;
    if use_token == true {
        match fs::File::open(&token_file) {
            Err(_) => {
                println!("[-] Open token file {} error! Will create one.", token_file.as_path().to_str().unwrap());
            },
            Ok(mut f) => {
                println!("[+] Open token file success! ");
                match f.read(&mut launch_token) {
                    Ok(1024) => {
                        println!("[+] Token file valid!");
                    },
                    _ => println!("[+] Token file invalid, will create new token file"),
                }
            }
        }
    }

    // Step 2: call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {secs_attr: sgx_attributes_t { flags:0, xfrm:0}, misc_select:0};
    let enclave = try!(SgxEnclave::create(ENCLAVE_FILE,
                                          debug,
                                          &mut launch_token,
                                          &mut launch_token_updated,
                                          &mut misc_attr));

    // Step 3: save the launch token if it is updated
    if use_token == true && launch_token_updated != 0 {
        // reopen the file with write capablity
        match fs::File::create(&token_file) {
            Ok(mut f) => {
                match f.write_all(&launch_token) {
                    Ok(()) => println!("[+] Saved updated launch token!"),
                    Err(_) => println!("[-] Failed to save updated launch token!"),
                }
            },
            Err(_) => {
                println!("[-] Failed to save updated enclave token, but doesn't matter");
            },
        }
    }

    Ok(enclave)
}

const CLIENT: mio::Token = mio::Token(0);

/// This encapsulates the TCP-level connection, some connection
/// state, and the underlying TLS-level session.
struct TlsClient {
    enclave_id: sgx_enclave_id_t,
    socket: TcpStream,
    closing: bool,
    tlsclient: *const c_void,
}

impl TlsClient {
    fn ready(&mut self,
             poll: &mut mio::Poll,
             ev: &mio::Event) -> bool {

        assert_eq!(ev.token(), CLIENT);

        if ev.readiness().is_error() {
            println!("Error");
            return false;
        }

        if ev.readiness().is_readable() {
            self.do_read();
        }

        if ev.readiness().is_writable() {
            self.do_write();
        }

        if self.is_closed() {
            println!("Connection closed");
            return false;
        }

        self.reregister(poll);

        true
    }
}

impl TlsClient {
    fn new(enclave_id: sgx_enclave_id_t, sock: TcpStream, hostname: &str, cert: &str) -> Option<TlsClient> {

        println!("[+] TlsClient new {} {}", hostname, cert);

        let mut tlsclient: *const c_void = ptr::null();
        let c_host = CString::new(hostname.to_string()).unwrap();
        let c_cert = CString::new(cert.to_string()).unwrap();

        let retval = unsafe {
            tls_client_new(enclave_id,
                           &mut tlsclient as *mut *const c_void,
                           sock.as_raw_fd(),
                           c_host.as_ptr() as *const c_char,
                           c_cert.as_ptr() as *const c_char)
        };

        if retval != sgx_status_t::SGX_SUCCESS {
            println!("[-] ECALL Enclave [tls_client_new] Failed {}!", retval);
            return Option::None;
        }

        if tlsclient.is_null() {
            println!("[-] New enclave tlsclient error");
            return Option::None;
        }

        Option::Some(
            TlsClient {
            enclave_id: enclave_id,
            socket: sock,
            closing: false,
            tlsclient: tlsclient as *const c_void,
        })
    }

    fn close(&self) {

        let retval = unsafe {
            tls_client_close(self.enclave_id, self.tlsclient)
        };

        if retval != sgx_status_t::SGX_SUCCESS {
            println!("[-] ECALL Enclave [tls_client_close] Failed {}!", retval);
        }
    }

    fn read_tls(&self, buf: &mut [u8]) -> isize {
        let mut retval = -1;
        let result = unsafe {
            tls_client_read(self.enclave_id,
                            &mut retval,
                            self.tlsclient,
                            buf.as_mut_ptr() as * mut c_void,
                            buf.len() as c_int)
        };

        match result {
            sgx_status_t::SGX_SUCCESS => { retval as isize }
            _ => {
                println!("[-] ECALL Enclave [tls_client_read] Failed {}!", result);
                -1
            }
        }
    }

    fn write_tls(&self, buf: &[u8]) -> isize {
        let mut retval = -1;
        let result = unsafe {
            tls_client_write(self.enclave_id,
                             &mut retval,
                             self.tlsclient,
                             buf.as_ptr() as * const c_void,
                             buf.len() as c_int)
        };

        match result {
            sgx_status_t::SGX_SUCCESS => { retval as isize }
            _ => {
                println!("[-] ECALL Enclave [tls_client_write] Failed {}!", result);
                -1
            }
        }
    }

    /// We're ready to do a read.
    fn do_read(&mut self) {
        // BUFFER_SIZE = 1024, just for test.
        // Do read all plaintext, you need to do more ecalls to get buffer size and buffer.
        let mut plaintext = vec![0; BUFFER_SIZE];
        let rc = self.read_tls(plaintext.as_mut_slice());
        if rc == -1 {
            println!("TLS read error: {:?}", rc);
            self.closing = true;
            return;
        }
        plaintext.resize(rc as usize, 0);
        io::stdout().write_all(&plaintext).unwrap();
    }

    fn do_write(&mut self) {
        let buf = Vec::new();
        self.write_tls(buf.as_slice());
    }

    fn register(&self, poll: &mut mio::Poll) {
        poll.register(&self.socket,
                      CLIENT,
                      self.ready_interest(),
                      mio::PollOpt::level() | mio::PollOpt::oneshot())
            .unwrap();
    }

    fn reregister(&self, poll: &mut mio::Poll) {
        poll.reregister(&self.socket,
                        CLIENT,
                        self.ready_interest(),
                        mio::PollOpt::level() | mio::PollOpt::oneshot())
            .unwrap();
    }

    fn wants_read(&self) -> bool {
        let mut retval = -1;
        let result = unsafe {
            tls_client_wants_read(self.enclave_id,
                                  &mut retval,
                                  self.tlsclient)
        };

        match result {
            sgx_status_t::SGX_SUCCESS => { },
            _ => {
                println!("[-] ECALL Enclave [tls_client_wants_read] Failed {}!", result);
                return false;
            }
        }
        match retval {
            0 => false,
            _ => true
        }
    }

    fn wants_write(&self) -> bool {
        let mut retval = -1;
        let result = unsafe {
            tls_client_wants_write(self.enclave_id,
                                   &mut retval,
                                   self.tlsclient)
        };

        match result {
            sgx_status_t::SGX_SUCCESS => { },
            _ => {
                println!("[-] ECALL Enclave [tls_client_wants_write] Failed {}!", result);
                return false;
            }
        }
        match retval {
            0 => false,
            _ => true
        }
    }

    // Use wants_read/wants_write to register for different mio-level
    // IO readiness events.
    fn ready_interest(&self) -> mio::Ready {
        let rd = self.wants_read();
        let wr = self.wants_write();

        if rd && wr {
            mio::Ready::readable() | mio::Ready::writable()
        } else if wr {
            mio::Ready::writable()
        } else {
            mio::Ready::readable()
        }
    }

    fn is_closed(&self) -> bool {
        self.closing
    }
}

/// We implement `io::Write` and pass through to the TLS session
impl io::Write for TlsClient {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        Ok(self.write_tls(bytes) as usize)
    }
    // unused
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Read for TlsClient {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        Ok(self.read_tls(bytes) as usize)
    }
}

fn lookup_ipv4(host: &str, port: u16) -> SocketAddr {
    use std::net::ToSocketAddrs;

    let addrs = (host, port).to_socket_addrs().unwrap();
    for addr in addrs {
        if let SocketAddr::V4(_) = addr {
            return addr;
        }
    }

    unreachable!("Cannot lookup address");
}

fn main() {

    let enclave = match init_enclave() {
        Ok(r) => {
            println!("[+] Init Enclave Successful {}!", r.geteid());
            r
        },
        Err(x) => {
            println!("[-] Init Enclave Failed {}!", x.as_str());
            return;
        },
    };

    println!("[+] Test tlsclient in enclave, start!");

    let port = 8443;
    let hostname = "localhost";
    let cert = "./ca.cert";
    let addr = lookup_ipv4(hostname, port);
    let sock = TcpStream::connect(&addr).expect("[-] Connect tls server failed!");

    let tlsclient = TlsClient::new(enclave.geteid(),
                                   sock,
                                   hostname,
                                   cert);

    if tlsclient.is_some() {
        println!("[+] Tlsclient new success!");

        let mut tlsclient = tlsclient.unwrap();

        let httpreq = format!("GET / HTTP/1.1\r\nHost: {}\r\nConnection: \
                               close\r\nAccept-Encoding: identity\r\n\r\n",
                              hostname);

        tlsclient.write_all(httpreq.as_bytes()).unwrap();

        let mut poll = mio::Poll::new()
            .unwrap();
        let mut events = mio::Events::with_capacity(32);
        tlsclient.register(&mut poll);

        'outer: loop {
            poll.poll(&mut events, None).unwrap();
            for ev in events.iter() {
                if !tlsclient.ready(&mut poll, &ev) {
                    tlsclient.close();
                    break 'outer ;
                }
            }
        }
    } else {
        println!("[-] Tlsclient new failed!");
    }

    println!("[+] Test tlsclient in enclave, done!");

    enclave.destroy();
}
