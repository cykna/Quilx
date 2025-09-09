mod stream;
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
    Stream = 0x8, //as it goes from 0x8 to 0xf, the next values must be >0xf
}

///An Abstraction over the possible frames QUIC can have. When using this to send data, first it will be encoded on a way that is equivalent to
///what the specifications say.
pub enum Frame {
    Padding,
    Ping,
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
    ///Creates a new Stream Frame
    pub fn new_stream(stream: Stream) -> Self {
        Self::Stream(stream)
    }

    ///Retrieves the FrameType of this Frame. Used primarly to be sent on a datagram to determine how to read the following bytes
    pub fn frame_type(&self) -> FrameType {
        match self {
            Self::Padding => FrameType::Padding,
            Self::Ping => FrameType::Ping,
            Self::Stream(_) => FrameType::Stream,
        }
    }
}

impl Dencode for Frame {
    fn encode(&self, buf: &mut [u8]) -> usize {
        match self {
            Self::Padding | Self::Ping => {
                buf[0] = self.frame_type() as u8;
                1
            }
            Self::Stream(s) => {
                buf[0] = self.frame_type() as u8;
                let out = s.encode(&mut buf[1..]);
                out + 1
            }
        }
    }
    fn decode(buf: &[u8]) -> Result<(Self, usize), crate::dencode::DencodeError> {
        match buf[0] {
            n if n == FrameType::Padding as u8 => Ok((Self::Padding, 1)),
            n if n == FrameType::Ping as u8 => Ok((Self::Ping, 1)),
            n if n == FrameType::Stream as u8 => {
                let (stream, amount) = Stream::decode(&buf[1..])?;
                Ok((Self::new_stream(stream), amount + 1))
            }
            n => panic!("Invalid or not implemented value {n}"),
        }
    }
}
