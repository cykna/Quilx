use bytes::{Buf, BytesMut};

use crate::{
    connections::{LongHeader, LongHeaderParams},
    dencode::Dencode,
    frames,
};

pub const INITIAL_PACKET_MINIMUM_SIZE: usize = 1200;

pub struct InitialPacket {
    long: LongHeader,
    ///The token received by NEW_TOKEN frame, or at a retry packet. Note that this is empty on trying to implement a handshake
    token: Vec<u8>,
    //already serialized frames
    packets: Vec<u8>,
}

impl InitialPacket {
    pub fn new(header_config: LongHeaderParams) -> Self {
        Self {
            long: LongHeader::new(header_config),
            token: Vec::new(),
            packets: Vec::new(),
        }
    }
    ///Appends the given `frame` on this packet
    pub fn push_frame(&mut self, frame: frames::Frame) {
        let mut buf = BytesMut::new();
        frame.encode(&mut buf);
        self.packets.extend_from_slice(&buf[..]);
    }

    ///Fills the `packets` of this header with `PADDING` until the length in bytes is the minimum required by QUIC as defined on Section 14.1
    pub fn fill_padding(&mut self) {
        let remaining = self.len() - self.packets.len();
        if remaining > 0 {
            self.packets
                .resize(INITIAL_PACKET_MINIMUM_SIZE - remaining, 0); //0 == Padding, as defined at 19.1
        }
    }

    ///Retrieves the length in bytes this header takes
    pub fn len(&self) -> usize {
        let u8_len = std::mem::size_of::<u8>();
        self.long.len() + (u8_len * self.token.len()) + (u8_len * self.packets.len())
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
        (self.packets.len() as u64).encode(buf);
        self.packets.encode(buf);
    }
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::dencode::DencodeError> {
        let long = LongHeader::decode(buf)?;
        let token = {
            let size = buf.get_u64() as usize;
            let bytes = buf.copy_to_bytes(size);
            bytes.to_vec()
        };
        let packets = {
            let size = buf.get_u64() as usize;
            let bytes = buf.copy_to_bytes(size);
            bytes.to_vec()
        };
        Ok(Self {
            long,
            token,
            packets,
        })
    }
}
