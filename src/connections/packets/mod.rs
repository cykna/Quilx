mod handshake;
mod initial;
mod long;

pub use handshake::*;
pub use initial::*;
pub(crate) use long::*;

use crate::{dencode::Dencode, frames};

pub const PACKET_HANDSHAKE_SIZE: usize = 1200;

#[derive(Debug)]
///Enum representing the possible Packet types as defined on section 17
pub enum QuicPacket {
    Initial(InitialPacket),
    Handshake(HandshakePacket),
}

impl QuicPacket {
    ///Initializes a new Packet whose type is `Initial`. THis is used to stablish handshakes.
    ///By default the QUIC version is 1 and the stream number size is 4 bytes.
    pub fn initial() -> Self {
        Self::Initial(InitialPacket::new())
    }

    pub fn handshake() -> Self {
        Self::Handshake(HandshakePacket::new())
    }

    ///Retrieves the type of this Packet
    pub fn kind(&self) -> LongHeaderType {
        match self {
            Self::Initial(_) => LongHeaderType::Initial,
            Self::Handshake(_) => LongHeaderType::Handshake,
        }
    }

    pub fn frames(&self) -> Vec<frames::Frame> {
        match self {
            Self::Initial(ini) => ini.frames(),
            Self::Handshake(hand) => hand.frames(),
        }
    }

    pub fn push_frame(&mut self, frame: &frames::Frame) {
        match self {
            Self::Initial(ini) => ini.push_frame(frame),
            Self::Handshake(hand) => hand.push_frame(frame),
        }
    }

    ///Fills the Payload of this packet with the given `amount` of PADDING frames
    pub fn fill_padding(&mut self) {
        let len = PACKET_HANDSHAKE_SIZE - self.len();

        let packets = match self {
            Self::Initial(ini) => &mut ini.packets,
            Self::Handshake(hand) => &mut hand.packets,
        };
        packets.extend(std::iter::repeat(0).take(len));
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Initial(ini) => ini.len(),
            Self::Handshake(hand) => hand.len(),
        }
    }

    ///Retrieves the length of the buffer of this InitialPacket. Note that it's the size only of the payload of it
    pub fn buffer_len(&self) -> usize {
        match self {
            Self::Initial(ini) => &ini.packets,
            Self::Handshake(hand) => &hand.packets,
        }
        .len()
    }
}

impl Dencode for QuicPacket {
    fn encode(&self, buf: &mut bytes::BytesMut) {
        match self {
            Self::Initial(initial) => initial.encode(buf),
            Self::Handshake(hand) => hand.encode(buf),
        }
    }
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::dencode::DencodeError> {
        let first_byte = buf[0];

        let out = Ok(match (first_byte >> 4).into() {
            LongHeaderType::Initial => Self::Initial(InitialPacket::decode(buf)?),
            LongHeaderType::Handshake => Self::Handshake(HandshakePacket::decode(buf)?),
            ty => unimplemented!("Did not implement decoding for type {ty:?}"),
        });

        out
    }
}
