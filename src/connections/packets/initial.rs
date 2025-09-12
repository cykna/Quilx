use bytes::{Buf, Bytes, BytesMut};

use crate::{
    connections::{LongHeader, LongHeaderParams},
    dencode::Dencode,
    frames,
};

pub const INITIAL_PACKET_MINIMUM_SIZE: usize = 1200;

#[derive(Debug)]
///Initial Packet is a Header type used primary to send data to stablish a handshake, it is defined at section 17.2.2
pub struct InitialPacket {
    long: LongHeader,

    ///The token received by NEW_TOKEN frame, or at a retry packet. Note that this is empty on trying to implement a handshake
    token: Vec<u8>,
    ///Numeric value of the stream
    stream_number: u32,
    ///already serialized frames
    packets: Vec<u8>,
}

impl InitialPacket {
    ///Initializes a new InitialPacket with QUIC version of 1 and stream number size of 4bytes
    pub fn new() -> Self {
        Self {
            long: LongHeader::new(LongHeaderParams {
                version: 1,
                ty: super::LongHeaderType::Initial,
                reserved: 0b11,
            }),
            stream_number: 0,
            token: Vec::new(),
            packets: Vec::new(),
        }
    }

    #[inline]
    ///Retrieves the numeric length of the packet number
    pub fn stream_num_len(&self) -> usize {
        self.long.first_byte as usize & 0b11
    }

    ///Appends the given `frame` on this packet
    pub fn push_frame(&mut self, frame: frames::Frame) {
        let mut buf = BytesMut::new();
        frame.encode(&mut buf);
        self.packets.extend_from_slice(&buf[..]);
    }

    ///Fills the `packets` of this header with `PADDING` until the length in bytes is the minimum required by QUIC as defined on Section 14.1
    pub fn fill_padding(&mut self) {
        if self.len() < INITIAL_PACKET_MINIMUM_SIZE {
            self.packets
                .resize(INITIAL_PACKET_MINIMUM_SIZE - self.len(), 0); //0 == Padding, as defined at 19.1
        }
    }

    #[inline]
    ///Retrieves the length in bytes this header takes
    pub fn len(&self) -> usize {
        let u8_len = std::mem::size_of::<u8>();
        self.long.len() + (u8_len * self.token.len()) + (u8_len * self.packets.len())
    }

    #[inline]
    ///Retrieves all the frames this Header contains
    pub fn frames(&self) -> Vec<frames::Frame> {
        Vec::decode(&mut Bytes::copy_from_slice(&self.packets)).unwrap()
    }
}

impl Dencode for InitialPacket {
    fn encode(&self, buf: &mut BytesMut) {
        self.long.encode(buf);
        (self.token.len() as u64).encode(buf);
        if !self.token.is_empty() {
            self.token.encode(buf);
        }
        assert!(
            !self.packets.is_empty(),
            "Cannot send packet whose size is 0"
        );

        ((self.packets.len() + self.stream_num_len()) as u64).encode(buf);
        self.stream_number.encode(buf);
        self.packets.encode(buf);
    }
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::dencode::DencodeError> {
        let long = LongHeader::decode(buf)?;
        let token = {
            let size = buf.get_u64() as usize;
            buf.copy_to_bytes(size).to_vec()
        };

        let mut len = buf.get_u64() as usize;
        let stream_number = {
            let size = long.first_byte & 0b11; //length of stream number
            len -= size as usize + 1; //len = packets_size + stream_number_size
            match size {
                0 => buf.get_u8() as u32,
                1 => buf.get_u16() as u32,
                2 => (buf.get_u16() << 8 | buf.get_u8() as u16) as u32,
                3 => buf.get_u32(),
                _ => unreachable!(),
            }
        };
        let packets = buf.copy_to_bytes(len.min(buf.remaining())).to_vec();
        Ok(Self {
            long,
            stream_number,
            token,
            packets,
        })
    }
}
