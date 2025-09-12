mod initial;
mod long;

pub use initial::*;
pub(crate) use long::*;

use crate::{dencode::Dencode, frames};
#[derive(Debug)]
///Enum representing the possible Packet types as defined on section 17
pub enum QuicPacket {
    Initial(InitialPacket),
}

impl QuicPacket {
    ///Initializes a new Packet whose type is `Initial`. THis is used to stablish handshakes.
    ///By default the QUIC version is 1 and the stream number size is 4 bytes.
    pub fn initial() -> Self {
        Self::Initial(InitialPacket::new())
    }

    pub fn frames(&self) -> Vec<frames::Frame> {
        match self {
            Self::Initial(ini) => ini.frames(),
        }
    }
}

impl Dencode for QuicPacket {
    fn encode(&self, buf: &mut bytes::BytesMut) {
        match self {
            Self::Initial(initial) => initial.encode(buf),
        }
    }
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::dencode::DencodeError> {
        let first_byte = buf[0];
        //Retrieves the type of the header
        Ok(match (first_byte >> 4).into() {
            LongHeaderType::Initial => Self::Initial(InitialPacket::decode(buf)?),
            ty => unimplemented!("Did not implement decoding for type {ty:?}"),
        })
    }
}
