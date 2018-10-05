const ASN1_CONSTRUCTED_FLAG: u8 = 0x20;

#[allow(unused)]
#[repr(u8)]
pub enum Tag {
    Bool = 0x1,
    Integer = 0x2,
    BitString = 0x3,
    OctetString = 0x4,
    ObjectIdentifier = 0x6,
    PrintableString = 0x13,
    UTCTime = 0x17,
    Sequence = (0x10 | ASN1_CONSTRUCTED_FLAG),
}
