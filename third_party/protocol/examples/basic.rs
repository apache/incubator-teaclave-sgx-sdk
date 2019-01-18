#[macro_use] extern crate protocol_derive;
#[macro_use] extern crate protocol;

#[derive(Protocol, Clone, Debug, PartialEq)]
pub struct Handshake;

#[derive(Protocol, Clone, Debug, PartialEq)]
pub struct Hello {
    id: i64,
    data: Vec<u8>,
}

#[derive(Protocol, Clone, Debug, PartialEq)]
pub struct Goodbye {
    id: i64,
    reason: String,
}

#[derive(Protocol, Clone, Debug, PartialEq)]
pub struct Node {
    name: String,
    enabled: bool
}

// Defines a packet kind enum.
define_packet_kind!(Packet: u32 {
    0x00 => Handshake,
    0x01 => Hello,
    0x02 => Goodbye
});

fn main() {
    use std::net::TcpStream;

    let stream = TcpStream::connect("127.0.0.1:34254").unwrap();
    let mut connection = protocol::wire::stream::Connection::new(stream, protocol::wire::middleware::pipeline::default());

    connection.send_packet(&Packet::Handshake(Handshake)).unwrap();
    connection.send_packet(&Packet::Hello(Hello { id: 0, data: vec![ 55 ]})).unwrap();
    connection.send_packet(&Packet::Goodbye(Goodbye { id: 0, reason: "leaving".to_string() })).unwrap();

    loop {
        if let Some(response) = connection.receive_packet().unwrap() {
            println!("{:?}", response);
            break;
        }
    }
}

