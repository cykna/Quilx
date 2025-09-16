use bytes::Buf;

use crate::dencode::Dencode;

///Enum representation of the types a `LongHeader` can have.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum LongHeaderType {
    Initial,
    ZeroRTT,
    Handshake,
    Retry,
}

impl From<u8> for LongHeaderType {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0 => Self::Initial,
            1 => Self::ZeroRTT,
            2 => Self::Handshake,
            3 => Self::Retry,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
///Long Header defined in Section 17.2 of QUIC specification, which is used to transfer data between based on it's type.
///Note that all the fields are in order of appearance on the QUIC representation of it
pub struct LongHeader {
    pub(crate) first_byte: u8, //header, fixed bit, long packet type, type specific, are included here in this order
    pub(crate) version: u32,
    ///Length of both dest_id and src_id is ommited here, since we can calculate them via `.len()` method, but they are added on the raw bytes.
    ///'conn_dest_id' determines the connection destination id
    pub(crate) conn_dest_id: Vec<u8>, //0..160
    ///Length of both dest_id and src_id is ommited here, since we can calculate them via `.len()` method, but they are added on the raw bytes
    ///'conn_src_id' determines the connection source id
    pub(crate) conn_src_id: Vec<u8>, //0..160
}

#[derive(Debug, Clone, Copy)]
pub struct LongHeaderParams {
    ///The version of QUIC
    pub version: u32,
    ///The type of the LongHeader
    pub ty: LongHeaderType,
    ///The 4 lowest bits of the first byte of the header
    pub reserved: u8,
}

impl LongHeader {
    ///Creates a new LongHeader. By the specification, `ty` determines the type of this header, and `reserved` are the 4 lowest bits ysed by type specifics. To understand each, look up to section 17.2 of the specification
    pub fn new(params: LongHeaderParams) -> Self {
        assert!(params.reserved <= 0xf);
        Self {
            first_byte: 0x80 | 0x40 | ((params.ty as u8) << 4) | params.reserved,
            version: params.version,
            conn_src_id: Vec::new(),
            conn_dest_id: Vec::new(),
        }
    }

    ///Takes the length in bytes this header takes
    pub fn len(&self) -> usize {
        let u8_size = std::mem::size_of::<u8>();
        //first byte, version, DCID len, DCID, SCID len, SCID
        u8_size
            + std::mem::size_of::<u32>()
            + u8_size
            + (u8_size * self.conn_dest_id.len())
            + u8_size
            + (u8_size * self.conn_src_id.len())
    }
}

impl Dencode for LongHeader {
    fn encode(&self, buf: &mut bytes::BytesMut) {
        self.first_byte.encode(buf);
        self.version.encode(buf);
        (self.conn_dest_id.len() as u8).encode(buf);
        self.conn_dest_id.encode(buf);
        (self.conn_src_id.len() as u8).encode(buf);
        self.conn_src_id.encode(buf);
    }
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::dencode::DencodeError> {
        let first = buf.get_u8();
        let version = buf.get_u32();
        let cdid = {
            let cdid_len = buf.get_u8();
            let cdid = buf.copy_to_bytes(cdid_len as usize);
            cdid.to_vec()
        };
        //literally the same shit
        let scid = {
            let scid_len = buf.get_u8();

            let cdid = buf.copy_to_bytes(scid_len as usize);
            cdid.to_vec()
        };
        Ok(Self {
            first_byte: first,
            version,
            conn_dest_id: cdid,
            conn_src_id: scid,
        })
    }
}
