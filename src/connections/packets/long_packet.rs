use bytes::{Buf, BufMut, BytesMut};

use crate::{
    connections::{LongHeader, LongHeaderParams, LongHeaderType},
    dencode::Dencode,
    frames,
};

pub const PACKET_HANDSHAKE_SIZE: usize = 1200;

#[derive(Debug)]
///Enum representing the possible Packet types as defined on section 17
pub enum QuicLongPacket {
    Initial {
        long: LongHeader,

        ///The token received by NEW_TOKEN frame, or at a retry packet. Note that this is empty on trying to stablish a handshake
        token: Vec<u8>,
        ///Numeric value of the packet
        packet_number: u32,
        ///already serialized frames
        packets: Vec<frames::Frame>,
    },
    Handshake {
        long: LongHeader,
        packet_number: u32,
        ///Already serialized packets
        packets: Vec<frames::Frame>,
    },
}

impl QuicLongPacket {
    ///Initializes a new Packet whose type is `Initial`. THis is used to stablish handshakes.
    ///By default the QUIC version is 1 and the stream number size is 4 bytes.
    pub fn initial() -> Self {
        let long = LongHeader::new(LongHeaderParams {
            version: 1,
            ty: super::LongHeaderType::Initial,
            reserved: 0b11,
        });
        Self::Initial {
            packet_number: 0,
            token: Vec::new(),
            packets: Vec::new(),
            long,
        }
    }

    pub fn handshake() -> Self {
        Self::Handshake {
            long: LongHeader::new(LongHeaderParams {
                version: 1,
                ty: super::LongHeaderType::Handshake,
                reserved: 0b11,
            }),
            packet_number: 0,
            packets: Vec::new(),
        }
    }

    #[inline]
    ///Retrieves the numeric length of the stream number of this packet
    pub fn stream_num_len(&self) -> usize {
        match self {
            Self::Initial { long, .. } | Self::Handshake { long, .. } => {
                long.first_byte as usize & 0b11 + 1
            }
        }
    }

    ///Retrieves the type of this Packet
    pub fn kind(&self) -> LongHeaderType {
        match self {
            Self::Initial { .. } => LongHeaderType::Initial,
            Self::Handshake { .. } => LongHeaderType::Handshake,
        }
    }

    pub fn frames(&self) -> &Vec<frames::Frame> {
        match self {
            Self::Initial { packets, .. } | Self::Handshake { packets, .. } => packets,
        }
    }

    pub fn frames_mut(&mut self) -> &mut Vec<frames::Frame> {
        match self {
            Self::Initial { packets, .. } | Self::Handshake { packets, .. } => packets,
        }
    }

    pub fn push_frame(&mut self, frame: frames::Frame) {
        self.frames_mut().push(frame);
    }

    ///Retrieves the size in bytes of this header without the payload
    pub fn header_size(&self) -> usize {
        match self {
            Self::Initial { long, token, .. } => {
                long.len() + std::mem::size_of::<u64>() * 2 + token.len()
            }
            Self::Handshake { long, .. } => long.len() + std::mem::size_of::<u64>() * 2,
        }
    }

    ///Encodes only the header of this packet assuming the payload contain `len`
    pub fn encode_header_with_len(&self, len: usize, buf: &mut bytes::BytesMut) {
        match self {
            Self::Initial {
                long,
                token,
                packet_number,
                packets,
            } => {
                long.encode(buf);
                (token.len() as u64).encode(buf);
                if !token.is_empty() {
                    token.encode(buf);
                }
                assert!(!packets.is_empty(), "Cannot send packet whose size is 0");

                ((len + self.stream_num_len()) as u64).encode(buf);
            }
            Self::Handshake {
                long,
                packet_number,
                packets,
            } => {
                long.encode(buf);

                ((len + self.stream_num_len()) as u64).encode(buf);
                packet_number.encode(buf);
            }
        }
    }
}

impl Dencode for QuicLongPacket {
    fn encode(&self, buf: &mut bytes::BytesMut) {
        match self {
            Self::Initial {
                long,
                token,
                packet_number,
                packets,
            } => {
                long.encode(buf);
                (token.len() as u64).encode(buf);
                if !token.is_empty() {
                    token.encode(buf);
                }
                assert!(!packets.is_empty(), "Cannot send packet whose size is 0");
                let packets = {
                    let mut buf = BytesMut::with_capacity(packets.len());
                    packets.encode(&mut buf);
                    buf.split().freeze()
                };

                let len = 1200 - packets.len();
                ((len + self.stream_num_len()) as u64).encode(buf);
                packet_number.encode(buf);

                buf.put_slice(&packets[..]);
                buf.put_bytes(0, len);
            }
            Self::Handshake {
                long,
                packet_number,
                packets,
            } => {
                long.encode(buf);
                let other = {
                    let mut other = BytesMut::new();
                    packets.encode(&mut other);
                    other
                };
                ((other.len() + self.stream_num_len()) as u64).encode(buf);
                packet_number.encode(buf);
                buf.copy_from_slice(&other);
            }
        }
    }
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::dencode::DencodeError> {
        let first_byte = buf[0];

        match (first_byte >> 4).into() {
            LongHeaderType::Initial => {
                let long = LongHeader::decode(buf)?;

                let token = {
                    let size = buf.get_u64() as usize;
                    buf.copy_to_bytes(size).to_vec()
                };

                let mut len = buf.get_u64() as usize;

                let packet_number = {
                    let size = long.first_byte & 0b11; //length of stream number
                    len -= size as usize + 1;
                    match size {
                        0 => buf.get_u8() as u32,
                        1 => buf.get_u16() as u32,
                        2 => (buf.get_u16() << 8 | buf.get_u8() as u16) as u32,
                        3 => buf.get_u32(),
                        _ => unreachable!(),
                    }
                };

                let mut buf = buf.split_to(len);

                let packets = Vec::<frames::Frame>::decode(&mut buf)?;
                Ok(Self::Initial {
                    long,
                    token,
                    packet_number,
                    packets,
                })
            }
            LongHeaderType::Handshake => {
                let long = LongHeader::decode(buf)?;

                let len = buf.get_u64() as usize;
                let packet_number = {
                    let size = long.first_byte & 0b11; //length of stream number

                    match size {
                        0 => buf.get_u8() as u32,
                        1 => buf.get_u16() as u32,
                        2 => (buf.get_u16() << 8 | buf.get_u8() as u16) as u32,
                        3 => buf.get_u32(),
                        _ => unreachable!(),
                    }
                };
                let mut buf = buf.split_to(len);
                let packets = Vec::decode(&mut buf)?;
                Ok(Self::Handshake {
                    long,
                    packet_number,
                    packets,
                })
            }
            ty => unimplemented!("Did not implement decoding for type {ty:?}"),
        }
    }
}
