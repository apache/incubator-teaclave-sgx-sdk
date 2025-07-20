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
// under the License.

use crate::io::ErrorKind;
use crate::net::test::{next_test_ip4, next_test_ip6};
use crate::net::*;
use crate::sync::mpsc::channel;
use crate::thread;
use crate::time::{Duration, Instant};

use sgx_test_utils::test_case;

fn each_ip(f: &mut dyn FnMut(SocketAddr, SocketAddr)) {
    f(next_test_ip4(), next_test_ip4());
    f(next_test_ip6(), next_test_ip6());
}

macro_rules! t {
    ($e:expr) => {
        match $e {
            Ok(t) => t,
            Err(e) => panic!("received error for `{}`: {}", stringify!($e), e),
        }
    };
}

#[test_case]
fn bind_error() {
    match UdpSocket::bind("1.1.1.1:9999") {
        Ok(..) => panic!(),
        Err(e) => assert_eq!(e.kind(), ErrorKind::AddrNotAvailable),
    }
}

#[test_case]
fn socket_smoke_test_ip4() {
    each_ip(&mut |server_ip, client_ip| {
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();

        let _t = thread::spawn(move || {
            let client = t!(UdpSocket::bind(client_ip));
            rx1.recv().unwrap();
            t!(client.send_to(&[99], server_ip));
            tx2.send(()).unwrap();
        });

        let server = t!(UdpSocket::bind(server_ip));
        tx1.send(()).unwrap();
        let mut buf = [0];
        let (nread, src) = t!(server.recv_from(&mut buf));
        assert_eq!(nread, 1);
        assert_eq!(buf[0], 99);
        assert_eq!(src, client_ip);
        rx2.recv().unwrap();
    })
}

#[test_case]
fn socket_name() {
    each_ip(&mut |addr, _| {
        let server = t!(UdpSocket::bind(addr));
        assert_eq!(addr, t!(server.local_addr()));
    })
}

#[test_case]
fn socket_peer() {
    each_ip(&mut |addr1, addr2| {
        let server = t!(UdpSocket::bind(addr1));
        assert_eq!(server.peer_addr().unwrap_err().kind(), ErrorKind::NotConnected);
        t!(server.connect(addr2));
        assert_eq!(addr2, t!(server.peer_addr()));
    })
}

#[test_case]
fn udp_clone_smoke() {
    each_ip(&mut |addr1, addr2| {
        let sock1 = t!(UdpSocket::bind(addr1));
        let sock2 = t!(UdpSocket::bind(addr2));

        let _t = thread::spawn(move || {
            let mut buf = [0, 0];
            assert_eq!(sock2.recv_from(&mut buf).unwrap(), (1, addr1));
            assert_eq!(buf[0], 1);
            t!(sock2.send_to(&[2], addr1));
        });

        let sock3 = t!(sock1.try_clone());

        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();
        let _t = thread::spawn(move || {
            rx1.recv().unwrap();
            t!(sock3.send_to(&[1], addr2));
            tx2.send(()).unwrap();
        });
        tx1.send(()).unwrap();
        let mut buf = [0, 0];
        assert_eq!(sock1.recv_from(&mut buf).unwrap(), (1, addr2));
        rx2.recv().unwrap();
    })
}

#[test_case]
fn udp_clone_two_read() {
    each_ip(&mut |addr1, addr2| {
        let sock1 = t!(UdpSocket::bind(addr1));
        let sock2 = t!(UdpSocket::bind(addr2));
        let (tx1, rx) = channel();
        let tx2 = tx1.clone();

        let _t = thread::spawn(move || {
            t!(sock2.send_to(&[1], addr1));
            rx.recv().unwrap();
            t!(sock2.send_to(&[2], addr1));
            rx.recv().unwrap();
        });

        let sock3 = t!(sock1.try_clone());

        let (done, rx) = channel();
        let _t = thread::spawn(move || {
            let mut buf = [0, 0];
            t!(sock3.recv_from(&mut buf));
            tx2.send(()).unwrap();
            done.send(()).unwrap();
        });
        let mut buf = [0, 0];
        t!(sock1.recv_from(&mut buf));
        tx1.send(()).unwrap();

        rx.recv().unwrap();
    })
}

#[test_case]
fn udp_clone_two_write() {
    each_ip(&mut |addr1, addr2| {
        let sock1 = t!(UdpSocket::bind(addr1));
        let sock2 = t!(UdpSocket::bind(addr2));

        let (tx, rx) = channel();
        let (serv_tx, serv_rx) = channel();

        let _t = thread::spawn(move || {
            let mut buf = [0, 1];
            rx.recv().unwrap();
            t!(sock2.recv_from(&mut buf));
            serv_tx.send(()).unwrap();
        });

        let sock3 = t!(sock1.try_clone());

        let (done, rx) = channel();
        let tx2 = tx.clone();
        let _t = thread::spawn(move || {
            if sock3.send_to(&[1], addr2).is_ok() {
                let _ = tx2.send(());
            }
            done.send(()).unwrap();
        });
        if sock1.send_to(&[2], addr2).is_ok() {
            let _ = tx.send(());
        }
        drop(tx);

        rx.recv().unwrap();
        serv_rx.recv().unwrap();
    })
}

#[test_case]
fn debug() {
    let name = if cfg!(windows) { "socket" } else { "fd" };
    let socket_addr = next_test_ip4();

    let udpsock = t!(UdpSocket::bind(socket_addr));
    let udpsock_inner = udpsock.0.socket().as_raw();
    let compare = format!("UdpSocket {{ addr: {socket_addr:?}, {name}: {udpsock_inner:?} }}");
    assert_eq!(format!("{udpsock:?}"), compare);
}

#[test_case]
fn timeouts() {
    let addr = next_test_ip4();

    let stream = t!(UdpSocket::bind(addr));
    let dur = Duration::new(15410, 0);

    assert_eq!(None, t!(stream.read_timeout()));

    t!(stream.set_read_timeout(Some(dur)));
    assert_eq!(Some(dur), t!(stream.read_timeout()));

    assert_eq!(None, t!(stream.write_timeout()));

    t!(stream.set_write_timeout(Some(dur)));
    assert_eq!(Some(dur), t!(stream.write_timeout()));

    t!(stream.set_read_timeout(None));
    assert_eq!(None, t!(stream.read_timeout()));

    t!(stream.set_write_timeout(None));
    assert_eq!(None, t!(stream.write_timeout()));
}

#[test_case]
fn test_read_timeout() {
    let addr = next_test_ip4();

    let stream = t!(UdpSocket::bind(addr));
    t!(stream.set_read_timeout(Some(Duration::from_millis(1000))));

    let mut buf = [0; 10];

    let start = Instant::now();
    loop {
        let kind = stream.recv_from(&mut buf).err().expect("expected error").kind();
        if kind != ErrorKind::Interrupted {
            assert!(
                kind == ErrorKind::WouldBlock || kind == ErrorKind::TimedOut,
                "unexpected_error: {:?}",
                kind
            );
            break;
        }
    }
    assert!(start.elapsed() > Duration::from_millis(400));
}

#[test_case]
fn test_read_with_timeout() {
    let addr = next_test_ip4();

    let stream = t!(UdpSocket::bind(addr));
    t!(stream.set_read_timeout(Some(Duration::from_millis(1000))));

    t!(stream.send_to(b"hello world", addr));

    let mut buf = [0; 11];
    t!(stream.recv_from(&mut buf));
    assert_eq!(b"hello world", &buf[..]);

    let start = Instant::now();
    loop {
        let kind = stream.recv_from(&mut buf).err().expect("expected error").kind();
        if kind != ErrorKind::Interrupted {
            assert!(
                kind == ErrorKind::WouldBlock || kind == ErrorKind::TimedOut,
                "unexpected_error: {:?}",
                kind
            );
            break;
        }
    }
    assert!(start.elapsed() > Duration::from_millis(400));
}

// Ensure the `set_read_timeout` and `set_write_timeout` calls return errors
// when passed zero Durations
#[test_case]
fn test_timeout_zero_duration() {
    let addr = next_test_ip4();

    let socket = t!(UdpSocket::bind(addr));

    let result = socket.set_write_timeout(Some(Duration::new(0, 0)));
    let err = result.unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidInput);

    let result = socket.set_read_timeout(Some(Duration::new(0, 0)));
    let err = result.unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidInput);
}

#[test_case]
fn connect_send_recv() {
    let addr = next_test_ip4();

    let socket = t!(UdpSocket::bind(addr));
    t!(socket.connect(addr));

    t!(socket.send(b"hello world"));

    let mut buf = [0; 11];
    t!(socket.recv(&mut buf));
    assert_eq!(b"hello world", &buf[..]);
}

#[test_case]
fn connect_send_peek_recv() {
    each_ip(&mut |addr, _| {
        let socket = t!(UdpSocket::bind(addr));
        t!(socket.connect(addr));

        t!(socket.send(b"hello world"));

        for _ in 1..3 {
            let mut buf = [0; 11];
            let size = t!(socket.peek(&mut buf));
            assert_eq!(b"hello world", &buf[..]);
            assert_eq!(size, 11);
        }

        let mut buf = [0; 11];
        let size = t!(socket.recv(&mut buf));
        assert_eq!(b"hello world", &buf[..]);
        assert_eq!(size, 11);
    })
}

#[test_case]
fn peek_from() {
    each_ip(&mut |addr, _| {
        let socket = t!(UdpSocket::bind(addr));
        t!(socket.send_to(b"hello world", addr));

        for _ in 1..3 {
            let mut buf = [0; 11];
            let (size, _) = t!(socket.peek_from(&mut buf));
            assert_eq!(b"hello world", &buf[..]);
            assert_eq!(size, 11);
        }

        let mut buf = [0; 11];
        let (size, _) = t!(socket.recv_from(&mut buf));
        assert_eq!(b"hello world", &buf[..]);
        assert_eq!(size, 11);
    })
}

#[test_case]
fn ttl() {
    let ttl = 100;

    let addr = next_test_ip4();

    let stream = t!(UdpSocket::bind(addr));

    t!(stream.set_ttl(ttl));
    assert_eq!(ttl, t!(stream.ttl()));
}

#[test_case]
fn set_nonblocking() {
    each_ip(&mut |addr, _| {
        let socket = t!(UdpSocket::bind(addr));

        t!(socket.set_nonblocking(true));
        t!(socket.set_nonblocking(false));

        t!(socket.connect(addr));

        t!(socket.set_nonblocking(false));
        t!(socket.set_nonblocking(true));

        let mut buf = [0];
        match socket.recv(&mut buf) {
            Ok(_) => panic!("expected error"),
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(e) => panic!("unexpected error {e}"),
        }
    })
}
