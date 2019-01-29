pub use Parcel;

define_packet!(Handshake);
define_packet!(Kick);

define_packet!(Hello {
    id: i64,
    data: Vec<u8>
});

define_packet!(Goodbye {
    id: i64,
    reason: String
});

define_packet!(Properties {
    properties: ::std::collections::HashMap<String, bool>
});

define_packet_kind!(PacketKind: u32 {
    0x00 => Handshake,
    0x01 => Kick,
    0x02 => Hello,
    0x03 => Goodbye,
    0x04 => Properties
});

const HELLO_EXPECTED_BYTES: &'static [u8] = &[
    0x00, 0x00, 0x00, 0x02, // Packet ID
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 55, // 'id' field
    0x00, 0x00, 0x00, 0x03, // 'data' length
    0x01, 0x02, 0x03, // 'data' array
];

#[test]
fn sets_packet_ids_correctly() {
    let hello = Hello { id: 55, data: vec![1, 2, 3] };
    let goodbye = Goodbye { id: 8765, reason: "um".to_string() };

    assert_eq!(PacketKind::Handshake(Handshake).packet_id(), 0x00);
    assert_eq!(PacketKind::Kick(Kick).packet_id(), 0x01);

    assert_eq!(PacketKind::Hello(hello).packet_id(), 0x02);
    assert_eq!(PacketKind::Goodbye(goodbye).packet_id(), 0x03);
}

#[test]
fn reads_packet_ids_correctly() {
    let packet = PacketKind::from_raw_bytes(HELLO_EXPECTED_BYTES).unwrap();

    assert_eq!(packet.raw_bytes().unwrap(), HELLO_EXPECTED_BYTES);
}

#[test]
fn writes_packet_ids_correctly() {
    let hello = Hello { id: 55, data: vec![1, 2, 3] };
    let packet = PacketKind::Hello(hello);

    assert_eq!(packet.raw_bytes().unwrap(), HELLO_EXPECTED_BYTES);
}

