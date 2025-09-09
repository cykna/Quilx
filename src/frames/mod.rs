mod crypto;
mod stream;
use bytes::{Buf, BufMut, Bytes, BytesMut};
pub use crypto::*;
pub use stream::*;

use crate::dencode::Dencode;

///This enum covers all the frame types implemented by quilx, not meanning it will have 100% of them as defined on the specification(No.19), even though it might be
///Each variant is just the frame type used to determine the frame when sending/receiving content on a datagram.
///Look up to https://datatracker.ietf.org/doc/html/rfc9000#name-frame-types-and-formats for more information about
#[derive(Debug, Clone)]
#[repr(u8)]
pub enum FrameType {
    Padding = 0x00,
    Ping = 0x01,
    Crypto = 0x06,
    Stream = 0x7, //as it goes from 0x8 to 0xf, the next values must be >0xf
}

impl Dencode for FrameType {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u8(self.clone() as u8);
    }
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::dencode::DencodeError> {
        Ok(match buf.get_u8() {
            0x0 => Self::Padding,
            0x1 => Self::Ping,
            0x6 => Self::Crypto,
            0x7..0xf => Self::Stream,
            _ => panic!("Unrecognized frame type"),
        })
    }
}

#[derive(Debug)]
///An Abstraction over the possible frames QUIC can have. When using this to send data, first it will be encoded on a way that is equivalent to
///what the specifications say.
pub enum Frame {
    Padding,
    Ping,
    Crypto(Crypto),
    Stream(Stream),
}

impl Frame {
    ///Creates a new Padding Frame
    pub fn new_padding() -> Self {
        Self::Padding
    }
    ///Creates a new Ping Frame
    pub fn new_ping() -> Self {
        Self::Ping
    }

    pub fn new_crypto(crypto: Crypto) -> Self {
        Self::Crypto(crypto)
    }
    ///Creates a new Stream Frame
    pub fn new_stream(stream: Stream) -> Self {
        Self::Stream(stream)
    }

    ///Retrieves the FrameType of this Frame. Used primarly to be sent on a datagram to determine how to read the following bytes
    pub fn frame_type(&self) -> FrameType {
        match self {
            Self::Padding => FrameType::Padding,
            Self::Ping => FrameType::Ping,
            Self::Crypto(_) => FrameType::Crypto,
            Self::Stream(_) => FrameType::Stream,
        }
    }
}

impl Dencode for Frame {
    fn encode(&self, buf: &mut BytesMut) {
        self.frame_type().encode(buf);
        match self {
            Self::Padding | Self::Ping => {}
            Self::Stream(s) => s.encode(buf),
            Self::Crypto(s) => s.encode(buf),
        }
    }
    fn decode(buf: &mut Bytes) -> Result<Self, crate::dencode::DencodeError> {
        let ty = buf.get_u8();
        match ty {
            n if n == FrameType::Padding as u8 => Ok(Self::Padding),
            n if n == FrameType::Ping as u8 => Ok(Self::Ping),
            n if n == FrameType::Stream as u8 => {
                let stream = Stream::decode(buf)?;
                Ok(Self::new_stream(stream))
            }
            n if n == FrameType::Crypto as u8 => {
                let crypto = Crypto::decode(buf)?;
                Ok(Self::Crypto(crypto))
            }

            n => panic!("Invalid or not implemented value {n}"),
        }
    }
}
