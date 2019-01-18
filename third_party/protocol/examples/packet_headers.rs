//! An example of packets with common headers.
//! This works because types and packets are the same thing.
//! This means that we can simply have a packet with another packet field.

extern crate protocol;
#[macro_use] extern crate protocol_derive;

#[derive(Protocol, Clone, Debug, PartialEq)]
pub struct Handshake;

#[derive(Protocol, Clone, Debug, PartialEq)]
pub struct Packet {
    headers: std::collections::HashMap<String, String>,
    kind: PacketKind
}

#[derive(Protocol, Clone, Debug, PartialEq)]
pub enum PacketKind {
    Handshake(Handshake),
}

fn main() {
    use std::net::TcpStream;

    let stream = TcpStream::connect("127.0.0.1:34254").unwrap();
    let mut connection = protocol::wire::stream::Connection::new(stream, protocol::wire::middleware::pipeline::default());

    connection.send_packet(&Packet {
        headers: std::collections::HashMap::new(),
        kind: PacketKind::Handshake(Handshake),
    }).unwrap();

    loop {
        if let Some(response) = connection.receive_packet().unwrap() {
            println!("{:?}", response);
            break;
        }
    }
}

