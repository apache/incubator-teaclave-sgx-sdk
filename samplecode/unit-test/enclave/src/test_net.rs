use std::net::ToSocketAddrs;

pub fn test_gai_error() {
    let result = ("rust-lang.org", 80).to_socket_addrs();
    assert!(result.is_ok());
    let result = ("rust-lang.or", 80).to_socket_addrs(); 
    assert!(result.is_err());
}


use std::thread;
use std::net;
use std::vec::Vec;

fn socket(listen_on: net::SocketAddr) -> net::UdpSocket {
  let attempt = net::UdpSocket::bind(listen_on);
  let socket;
  match attempt {
    Ok(sock) => {
      println!("Bound socket to {}", listen_on);
      socket = sock;
    },
    Err(err) => panic!("Could not bind: {}", err)
  }
  socket
}

fn read_message(socket: net::UdpSocket) -> Vec<u8> {
  let mut buf: [u8; 1] = [0; 1];
  println!("Reading data");
  let result = socket.recv_from(&mut buf);
  drop(socket);
  let data;
  match result {
    Ok((amt, src)) => {
      println!("Received data from {}", src);
      data = Vec::from(&buf[0..amt]);
    },
    Err(err) => panic!("Read error: {}", err)
  }
  data
}

pub fn send_message(send_addr: net::SocketAddr, target: net::SocketAddr, data: Vec<u8>) {
  let socket = socket(send_addr);
  println!("Sending data");
  let result = socket.send_to(&data, target);
  drop(socket);
  match result {
    Ok(amt) => println!("Sent {} bytes", amt),
    Err(err) => panic!("Write error: {}", err)
  }
}

fn listen(listen_on: net::SocketAddr) -> thread::JoinHandle<Vec<u8>> {
  let socket = socket(listen_on);
  let handle = thread::spawn(move || {
    read_message(socket)
  });
  handle
}

pub fn test_udp() {
    println!("UDP");
    let ip = net::Ipv4Addr::new(127, 0, 0, 1);
    let listen_addr = net::SocketAddrV4::new(ip, 8888);
    let send_addr = net::SocketAddrV4::new(ip, 8889);
    let future = listen(net::SocketAddr::V4(listen_addr));
    let message: Vec<u8> = vec![10];
    thread::sleep_ms(3000);
    send_message(net::SocketAddr::V4(send_addr), net::SocketAddr::V4(listen_addr), message);
    println!("Waiting");
    let received = future.join().unwrap();
    println!("Got {} bytes", received.len());
    assert_eq!(1, received.len());
    assert_eq!(10, received[0]);
  }