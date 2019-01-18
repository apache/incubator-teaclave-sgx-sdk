// Custom middleware example.
// All bytes that go through the protocol are rotated by an offset of 13.

#[macro_use] extern crate protocol;
#[macro_use] extern crate protocol_derive;

use std::num::Wrapping;

/// A rot-n middleware.
/// Rotates each byte by a specific offset.
#[derive(Clone, Debug)]
pub struct RotateMiddleware
{
    pub offset: u8,
}

impl RotateMiddleware
{
    pub fn rot13() -> Self {
        RotateMiddleware { offset: 13 }
    }
}

impl protocol::wire::Middleware for RotateMiddleware
{
    fn decode_data(&mut self, data: Vec<u8>) -> Result<Vec<u8>, protocol::Error> {
        Ok(data.into_iter().map(|byte| (Wrapping(byte) - Wrapping(self.offset)).0).collect())
    }

    fn encode_data(&mut self, data: Vec<u8>) -> Result<Vec<u8>, protocol::Error> {
        Ok(data.into_iter().map(|byte| (Wrapping(byte) + Wrapping(self.offset)).0).collect())
    }
}

define_middleware_pipeline!(Pipeline {
    rot: RotateMiddleware
});

impl Pipeline
{
    pub fn new() -> Self {
        Pipeline {
            rot: RotateMiddleware::rot13(),
        }
    }
}

#[derive(Protocol, Clone, Debug, PartialEq)]
pub struct Ping {
    id: i64,
    data: Vec<u8>
}

define_packet_kind!(Packet: u8 {
    0x00 => Ping
});

fn main() {
    use std::net::TcpStream;

    let stream = TcpStream::connect("127.0.0.1:34254").unwrap();
    let mut connection = protocol::wire::stream::Connection::new(stream, Pipeline::new());

    connection.send_packet(&Packet::Ping(Ping { id: 0, data: vec![ 55 ]})).unwrap();

    loop {
        if let Some(response) = connection.receive_packet().unwrap() {
            println!("{:?}", response);
            break;
        }
    }
}

