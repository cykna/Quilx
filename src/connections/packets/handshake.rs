use bytes::{Buf, BytesMut};

use crate::{
    connections::{LongHeader, LongHeaderParams},
    dencode::Dencode,
    frames::Frame,
};

#[derive(Debug)]
///Handshake packets are used by both client and servers to send/receive information about their handshake, thus, including cryptographic data. Check https://datatracker.ietf.org/doc/html/rfc9000#name-handshake-packet for more
pub struct HandshakePacket {
    long: LongHeader,
    packet_number: u32,
    ///Already serialized packets
    pub(crate) packets: Vec<u8>,
}

impl HandshakePacket {
    pub fn new() -> Self {
        Self {
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
        self.long.first_byte as usize & 0b11 + 1
    }

    #[inline]
    ///Appends the given `frame` into this packet
    pub fn push_frame(&mut self, frame: &Frame) {
        let mut buf = BytesMut::new();
        frame.encode(&mut buf);
        self.packets.extend_from_slice(&buf);
    }

    #[inline]
    ///Retrieves the length in bytes this header takes
    pub fn len(&self) -> usize {
        let u8_len = std::mem::size_of::<u8>();
        self.long.len() + (u8_len * self.packets.len()) + self.stream_num_len()
    }

    #[inline]
    ///Retrieves all the frames this Header contains
    pub fn frames(&self) -> Vec<Frame> {
        Vec::decode(&mut bytes::Bytes::copy_from_slice(&self.packets)).unwrap()
    }
}

impl Dencode for HandshakePacket {
    fn encode(&self, buf: &mut BytesMut) {
        self.long.encode(buf);

        ((self.packets.len() + self.stream_num_len()) as u64).encode(buf);
        self.packet_number.encode(buf);
        self.packets.encode(buf);
    }
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::dencode::DencodeError> {
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
        let packets = buf.copy_to_bytes(len).to_vec();
        Ok(Self {
            long,
            packet_number,
            packets,
        })
    }
}
