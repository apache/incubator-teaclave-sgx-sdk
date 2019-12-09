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

#![allow(dead_code)]
#![allow(unused_assignments)]

extern crate sgx_types;
extern crate sgx_urts;
use sgx_types::*;
use sgx_urts::SgxEnclave;

extern crate mio;
use mio::tcp::{TcpListener, TcpStream, Shutdown};

use std::os::unix::io::AsRawFd;
use std::ffi::CString;
use std::net;
use std::str;
use std::io::{self, Read, Write};
use std::collections::HashMap;

const BUFFER_SIZE: usize = 1024;

static ENCLAVE_FILE: &'static str = "enclave.signed.so";

extern {
    fn tls_server_new(eid: sgx_enclave_id_t, retval: *mut size_t,
                     fd: c_int, cert: *const c_char, key: *const c_char) -> sgx_status_t;
    fn tls_server_read(eid: sgx_enclave_id_t, retval: *mut c_int,
                     session_id: size_t, buf: *mut c_void, cnt: c_int) -> sgx_status_t;
    fn tls_server_write(eid: sgx_enclave_id_t, retval: *mut c_int,
                     session_id: size_t, buf: *const c_void, cnt: c_int) -> sgx_status_t;
    fn tls_server_wants_read(eid: sgx_enclave_id_t, retval: *mut c_int,
                     session_id: size_t) -> sgx_status_t;
    fn tls_server_wants_write(eid: sgx_enclave_id_t, retval: *mut c_int,
                     session_id: size_t) -> sgx_status_t;
    fn tls_server_close(eid: sgx_enclave_id_t,
                     session_id: size_t) -> sgx_status_t;
    fn tls_server_send_close(edi: sgx_enclave_id_t,
                     session_id: size_t) -> sgx_status_t;
}

fn init_enclave() -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {secs_attr: sgx_attributes_t { flags:0, xfrm:0}, misc_select:0};
    SgxEnclave::create(ENCLAVE_FILE,
                       debug,
                       &mut launch_token,
                       &mut launch_token_updated,
                       &mut misc_attr)
}

// Token for our listening socket.
const LISTENER: mio::Token = mio::Token(0);

// Which mode the server operates in.
#[derive(Clone)]
enum ServerMode {
    /// Write back received bytes
    Echo,

    /// Do one read, then write a bodged HTTP response and
    /// cleanly close the connection.
    Http,

    /// Forward traffic to/from given port on localhost.
    Forward(u16),
}

/// This binds together a TCP listening socket, some outstanding
/// connections, and a TLS server configuration.
struct TlsServer {
    enclave_id: sgx_enclave_id_t,
    server: TcpListener,
    cert: CString,
    key: CString,
    mode: ServerMode,
    connections: HashMap<mio::Token, Connection>,
    next_id: usize,
}

impl TlsServer {
    fn new(enclave_id: sgx_enclave_id_t, server: TcpListener, mode: ServerMode, cert: CString, key: CString) -> TlsServer {

        println!("[+] TlsServer new {:?} {:?}", cert, key);

        TlsServer {
            enclave_id: enclave_id,
            server: server,
            cert: cert,
            key: key,
            mode: mode,
            connections: HashMap::new(),
            next_id: 2
        }
    }

    fn accept(&mut self, poll: &mut mio::Poll) -> bool {
        match self.server.accept() {
            Ok((socket, addr)) => {

                println!("Accepting new connection from {:?}", addr);

                let mut tlsserver_id: usize = 0xFFFF_FFFF_FFFF_FFFF;
                let retval = unsafe {
                    tls_server_new(self.enclave_id,
                                   &mut tlsserver_id,
                                   socket.as_raw_fd(),
                                   self.cert.as_bytes_with_nul().as_ptr() as * const c_char,
                                   self.key.as_bytes_with_nul().as_ptr() as * const c_char)
                };

                if retval != sgx_status_t::SGX_SUCCESS {
                    println!("[-] ECALL Enclave [tls_server_new] Failed {}!", retval);
                    return false;
                }

                if tlsserver_id == 0xFFFF_FFFF_FFFF_FFFF {
                    println!("[-] New enclave tlsserver error");
                    return false;
                }

                let mode = self.mode.clone();
                let token = mio::Token(self.next_id);
                self.next_id += 1;
                self.connections.insert(token, Connection::new(self.enclave_id,
                                                               socket,
                                                               token,
                                                               mode,
                                                               tlsserver_id));
                self.connections[&token].register(poll);
                true
            }
            Err(e) => {
                println!("encountered error while accepting connection; err={:?}", e);
                false
            }
        }
    }

    fn conn_event(&mut self, poll: &mut mio::Poll, event: &mio::Event) {
        let token = event.token();

        if self.connections.contains_key(&token) {
            self.connections
                .get_mut(&token)
                .unwrap()
                .ready(poll, event);

            if self.connections[&token].is_closed() {
                self.connections.remove(&token);
            }
        }
    }
}

/// This is a connection which has been accepted by the server,
/// and is currently being served.
///
/// It has a TCP-level stream, a TLS-level session, and some
/// other state/metadata.
struct Connection {
    enclave_id: sgx_enclave_id_t,
    socket: TcpStream,
    token: mio::Token,
    closing: bool,
    mode: ServerMode,
    tlsserver_id: usize,
    back: Option<TcpStream>,
    sent_http_response: bool,
}

/// Open a plaintext TCP-level connection for forwarded connections.
fn open_back(mode: &ServerMode) -> Option<TcpStream> {
    match *mode {
        ServerMode::Forward(ref port) => {
            let addr = net::SocketAddrV4::new(net::Ipv4Addr::new(127, 0, 0, 1), *port);
            let conn = TcpStream::connect(&net::SocketAddr::V4(addr)).unwrap();
            Some(conn)
        }
        _ => None,
    }
}

/// This used to be conveniently exposed by mio: map EWOULDBLOCK
/// errors to something less-errory.
fn try_read(r: io::Result<usize>) -> io::Result<Option<usize>> {
    match r {
        Ok(len) => Ok(Some(len)),
        Err(e) => {
            if e.kind() == io::ErrorKind::WouldBlock {
                Ok(None)
            } else {
                Err(e)
            }
        }
    }
}

impl Connection {
    fn new(enclave_id: sgx_enclave_id_t,
           socket: TcpStream,
           token: mio::Token,
           mode: ServerMode,
           tlsserver_id: usize)
           -> Connection {
        let back = open_back(&mode);
        Connection {
            enclave_id: enclave_id,
            socket: socket,
            token: token,
            closing: false,
            mode: mode,
            tlsserver_id: tlsserver_id,
            back: back,
            sent_http_response: false,
        }
    }

    fn read_tls(&self, buf: &mut [u8]) -> isize {
        let mut retval = -1;
        let result = unsafe {
            tls_server_read(self.enclave_id,
                            &mut retval,
                            self.tlsserver_id,
                            buf.as_ptr() as * mut c_void,
                            buf.len() as c_int)
        };
        match result {
            sgx_status_t::SGX_SUCCESS => { retval as isize },
            _ => {
                println!("[-] ECALL Enclave [tls_server_wants_read] Failed {}!", result);
                return -1;
            },
        }
    }

    fn write_tls(&self, buf: &[u8]) -> isize {
        let mut retval = -1;
        let result = unsafe {
            tls_server_write(self.enclave_id,
                             &mut retval,
                             self.tlsserver_id,
                             buf.as_ptr() as * const c_void,
                             buf.len() as c_int)
        };
        match result {
            sgx_status_t::SGX_SUCCESS => { retval as isize },
            _ => {
                println!("[-] ECALL Enclave [tls_server_wants_read] Failed {}!", result);
                return -1;
            },
        }
    }

    fn wants_read(&self) -> bool {
        let mut retval = -1;
        let result = unsafe {
            tls_server_wants_read(self.enclave_id,
                                  &mut retval,
                                  self.tlsserver_id)
        };
        match result {
            sgx_status_t::SGX_SUCCESS => {},
            _ => {
                println!("[-] ECALL Enclave [tls_server_wants_read] Failed {}!", result);
                return false;
            },
        }

        match retval {
            0 => false,
            _ => true
        }
    }

    fn wants_write(&self) -> bool {
        let mut retval = -1 ;
        let result = unsafe {
            tls_server_wants_write(self.enclave_id,
                                   &mut retval,
                                   self.tlsserver_id)
        };

        match result {
            sgx_status_t::SGX_SUCCESS => {},
            _ => {
                println!("[-] ECALL Enclave [tls_server_wants_write] Failed {}!", result);
                return false;
            },
        }

        match retval {
            0 => false,
            _ => true
        }
    }

    fn tls_close(&self) {
        unsafe {
            tls_server_close(self.enclave_id, self.tlsserver_id)
        };
    }

    fn send_close_notify(&self) {
        unsafe {
            tls_server_send_close(self.enclave_id, self.tlsserver_id);
        }
    }

    /// We're a connection, and we have something to do.
    fn ready(&mut self, poll: &mut mio::Poll, ev: &mio::Event) {
        // If we're readable: read some TLS.  Then
        // see if that yielded new plaintext.  Then
        // see if the backend is readable too.
        if ev.readiness().is_readable() {
            self.do_tls_read();
            self.try_plain_read();
            self.try_back_read();
        }

        if ev.readiness().is_writable() {
            self.do_tls_write();
        }

        if self.closing {
            self.tls_close();
            let _ = self.socket.shutdown(Shutdown::Both);
            self.close_back();
        } else {
            self.reregister(poll);
        }
    }

    /// Close the backend connection for forwarded sessions.
    fn close_back(&mut self) {
        if self.back.is_some() {
            let back = self.back.as_mut().unwrap();
            back.shutdown(Shutdown::Both).unwrap();
        }
        self.back = None;
    }

    fn do_tls_read(&mut self) {
        // Read some TLS data.
        println!("Read some TLS data.");
        let mut buf = Vec::new();
        let rc = self.read_tls(buf.as_mut_slice());
        if rc == -1 {
            println!("read error {:?}", rc);
            self.closing = true;
            return;
        }
    }

    fn try_plain_read(&mut self) {
        // Read and process all available plaintext.
        let mut buf = vec![0; BUFFER_SIZE];
        let rc = self.read_tls(buf.as_mut_slice());
        if rc == -1 {
            println!("plaintext read failed: {:?}", rc);
            self.closing = true;
            return;
        }
        buf.resize(rc as usize, 0);
        if !buf.is_empty() {
            println!("plaintext read {:?} {:? }", buf.len(), buf);
            self.incoming_plaintext(&buf);
        }
    }

    fn try_back_read(&mut self) {
        if self.back.is_none() {
            return;
        }

        println!("try back read");

        // Try a non-blocking read.
        let mut buf = [0u8; BUFFER_SIZE];
        let maybe_len = {
            let back = self.back.as_mut().unwrap();
            let rc = try_read(back.read(&mut buf));
            if rc.is_err() {
                println!("backend read failed: {:?}", rc);
                self.closing = true;
                return;
            }
            rc.unwrap()
        };

        // If we have a successful but empty read, that's an EOF.
        // Otherwise, we shove the data into the TLS session.
        match maybe_len {
            Some(len) if len == 0 => {
                println!("back eof");
                self.closing = true;
            }
            Some(len) => {
                self.write_all(&buf[..len]).unwrap();
            }
            None => {}
        };
    }

    /// Process some amount of received plaintext.
    fn incoming_plaintext(&mut self, buf: &[u8]) {
        match self.mode {
            ServerMode::Echo => {
                self.write_all(buf).unwrap();
            }
            ServerMode::Http => {
                self.send_http_response_once();
            }
            ServerMode::Forward(_) => {
                self.back.as_mut().unwrap().write_all(buf).unwrap();
            }
        }
    }

    fn send_http_response_once(&mut self) {

        println!("send_http_response_once");

        let response = b"HTTP/1.0 200 OK\r\nConnection: close\r\n\r\nHello world from rustls tlsserver\r\n";
        if !self.sent_http_response {
            self.write_all(response)
                .unwrap();
            self.sent_http_response = true;
            self.send_close_notify();
        }
    }

    fn do_tls_write(&mut self) {
        let buf = Vec::new();
        let rc = self.write_tls(buf.as_slice());
        if rc == -1 {
            println!("write failed {:?}", rc);
            self.closing = true;
            return;
        }
    }

    fn register(&self, poll: &mut mio::Poll) {
        poll.register(&self.socket,
                      self.token,
                      self.event_set(),
                      mio::PollOpt::level() | mio::PollOpt::oneshot())
            .unwrap();

        if self.back.is_some() {
            poll.register(self.back.as_ref().unwrap(),
                          self.token,
                          mio::Ready::readable(),
                          mio::PollOpt::level() | mio::PollOpt::oneshot())
                .unwrap();
        }
    }

    fn reregister(&self, poll: &mut mio::Poll) {
        poll.reregister(&self.socket,
                        self.token,
                        self.event_set(),
                        mio::PollOpt::level() | mio::PollOpt::oneshot())
            .unwrap();

        if self.back.is_some() {
            poll.reregister(self.back.as_ref().unwrap(),
                            self.token,
                            mio::Ready::readable(),
                            mio::PollOpt::level() | mio::PollOpt::oneshot())
                .unwrap();
        }
    }

    /// What IO events we're currently waiting for,
    /// based on wants_read/wants_write.
    fn event_set(&self) -> mio::Ready {
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
impl io::Write for Connection {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        Ok(self.write_tls(bytes) as usize)
    }
    // unused
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Read for Connection {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        Ok(self.read_tls(bytes) as usize)
    }
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

    let addr: net::SocketAddr = "0.0.0.0:8443".parse().unwrap();
    let listener = TcpListener::bind(&addr).expect("cannot listen on port");

    let cert = CString::new("end.fullchain").unwrap();
    let key = CString::new("end.rsa").unwrap();

    let mut poll = mio::Poll::new().unwrap();
    poll.register(&listener,
                  LISTENER,
                  mio::Ready::readable(),
                  mio::PollOpt::level()).unwrap();

    let mut tlsserv = TlsServer::new(enclave.geteid(), listener, ServerMode::Echo, cert, key);

    println!("[+] TlsServer new success!");

    let mut events = mio::Events::with_capacity(256);

    'outer: loop {
        poll.poll(&mut events, None)
            .unwrap();

        for event in events.iter() {
            match event.token() {
                LISTENER => {
                    if !tlsserv.accept(&mut poll) {
                        break 'outer;
                    }
                }
                _ => tlsserv.conn_event(&mut poll, &event)
            }
        }
    }

    println!("[+] Test tlsServer in enclave, done!");

    enclave.destroy();
}
