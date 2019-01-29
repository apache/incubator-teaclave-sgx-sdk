use super::Transport;

use {Error, Parcel};

use std::prelude::v1::*;
use std::collections::VecDeque;
use std::io::prelude::*;
use std::io::Cursor;
use std::mem;

/// The type that we use to describe packet sizes.
pub type PacketSize = u32;

/// The current state.
#[derive(Clone, Debug)]
enum State
{
    /// We are awaiting packet size bytes.
    AwaitingSize(Vec<u8>),
    AwaitingPacket {
        size: PacketSize,
        received_data: Vec<u8>,
    },
}

/// A simple transport.
#[derive(Clone, Debug)]
pub struct Simple
{
    state: State,
    packets: VecDeque<Vec<u8>>,
}

impl Simple
{
    pub fn new() -> Self {
        Simple {
            state: State::AwaitingSize(Vec::new()),
            packets: VecDeque::new(),
        }
    }

    fn process_bytes(&mut self, bytes: &[u8]) -> Result<(), Error> {
        let mut read = Cursor::new(bytes);

        loop {
            match self.state.clone() {
                State::AwaitingSize(mut size_bytes) => {
                    let remaining_bytes = mem::size_of::<PacketSize>() - size_bytes.len();

                    let mut received_bytes = vec![0; remaining_bytes];
                    let bytes_read = read.read(&mut received_bytes)?;
                    received_bytes.drain(bytes_read..);

                    assert_eq!(received_bytes.len(), bytes_read);

                    size_bytes.extend(received_bytes.into_iter());

                    if size_bytes.len() == mem::size_of::<PacketSize>() {
                        let mut size_buffer = Cursor::new(size_bytes);

                        let size = PacketSize::read(&mut size_buffer).unwrap();

                        // We are now ready to receive packet data.
                        self.state = State::AwaitingPacket { size: size, received_data: Vec::new() }
                    } else {
                        // Still waiting to receive the whole packet.
                        self.state = State::AwaitingSize(size_bytes);
                        break;
                    }
                },
                State::AwaitingPacket { size, mut received_data } => {
                    let remaining_bytes = (size as usize) - received_data.len();
                    assert!(remaining_bytes > 0);

                    let mut received_bytes = vec![0; remaining_bytes];
                    let bytes_read = read.read(&mut received_bytes)?;
                    received_bytes.drain(bytes_read..);

                    assert_eq!(received_bytes.len(), bytes_read);

                    received_data.extend(received_bytes.into_iter());

                    assert!(received_data.len() <= (size as usize));

                    if (size as usize) == received_data.len() {
                        self.packets.push_back(received_data);

                        // Start reading the next packet.
                        self.state = State::AwaitingSize(Vec::new());
                    } else {
                        // Keep reading the current packet.
                        self.state = State::AwaitingPacket { size: size, received_data: received_data };
                        break;
                    }
                },
            }
        }

        Ok(())
    }
}

const BUFFER_SIZE: usize = 10000;

impl Transport for Simple
{
    fn process_data(&mut self,
                    read: &mut Read) -> Result<(), Error> {
        // Load the data into a temporary buffer before we process it.
        loop {
            let mut buffer = [0u8; BUFFER_SIZE];
            let bytes_read = read.read(&mut buffer).unwrap();
            let buffer = &buffer[0..bytes_read];

            if bytes_read == 0 {
                break;
            } else {
                self.process_bytes(buffer)?;

                // We didn't fill the whole buffer so stop now.
                if bytes_read != BUFFER_SIZE { break; }
            }
        }

        Ok(())
    }

    fn send_raw_packet(&mut self,
                       write: &mut Write,
                       packet: &[u8]) -> Result<(), Error> {
        // Prefix the packet size.
        (packet.len() as PacketSize).write(write)?;
        // Write the packet data.
        write.write(&packet)?;

        Ok(())
    }

    fn receive_raw_packet(&mut self) -> Result<Option<Vec<u8>>, Error> {
        Ok(self.packets.pop_front())
    }
}

#[cfg(test)]
mod test
{
    pub use super::Simple;
    pub use std::io::Cursor;
    pub use wire::stream::Transport;

    #[test]
    fn serialises_the_data_with_32bit_length_prefix() {
        let data: Vec<u8> = vec![5, 4, 3, 2, 1];
        let expected_data = &[
                0x00, 0x00, 0x00, 0x05, // 32-bit size prefix
                0x05, 0x04, 0x03, 0x02, 0x01, // The data
        ];

        let mut transport = Simple::new();

        let mut buffer = Cursor::new(Vec::new());

        transport.send_raw_packet(&mut buffer, &data).unwrap();
        let written_data = buffer.into_inner();

        assert_eq!(&written_data, &expected_data);
    }

    #[test]
    fn successfully_deserializes_data_with_32bit_length_prefix() {
        let data: Vec<u8> = vec![5, 4, 3, 2, 1];
        let expected_data = &[
                0x00, 0x00, 0x00, 0x05, // 32-bit size prefix
                0x05, 0x04, 0x03, 0x02, 0x01, // The data
        ];

        let mut transport = Simple::new();

        let mut buffer = Cursor::new(&expected_data);

        transport.process_data(&mut buffer).unwrap();
        let read_data = transport.receive_raw_packet().ok().unwrap().unwrap();
        assert_eq!(&read_data, &data);
    }
}

