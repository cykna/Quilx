mod handshake;
mod initial;
mod long;
mod long_packet;

pub use handshake::*;
pub use initial::*;
pub use long::*;
pub use long_packet::*;

use crate::dencode::Dencode;

#[derive(Debug)]
///A Quic Packet is either a 1-Rtt packet used to transfer data, or a Long Packet, used to stablish handshakes and
pub enum QuicPacket {
    Long(QuicLongPacket),
}

impl Dencode for QuicPacket {
    fn encode(&self, buf: &mut bytes::BytesMut) {
        match self {
            Self::Long(long) => long.encode(buf),
        }
    }
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::dencode::DencodeError> {
        Ok(Self::Long(QuicLongPacket::decode(buf)?))
    }
}
