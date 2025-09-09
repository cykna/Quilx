use bytes::{Bytes, BytesMut};

use crate::dencode::Dencode;
#[derive(Debug)]
pub struct Crypto {
    offset: u64,
    data: Bytes,
}

impl Crypto {
    pub fn new(offset: u64, data: Bytes) -> Self {
        Self { offset, data }
    }
}

impl Dencode for Crypto {
    fn encode(&self, buf: &mut BytesMut) {
        self.offset.encode(buf);
        self.data.encode(buf)
    }
    fn decode(buf: &mut Bytes) -> Result<Self, crate::dencode::DencodeError> {
        let offset = u64::decode(buf)?;
        let data = Bytes::decode(buf)?;
        Ok(Self::new(offset, data))
    }
}
