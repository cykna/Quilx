use std::ops::{Deref, Range};

use bytes::{Buf, Bytes, BytesMut};

use crate::dencode::Dencode;
#[derive(Debug)]
///A Crypto frame as defined on 19.6 of the specification of QUIC.
///This can be understood as the same of a STREAM frame, but the `data` of this is supposed to be encrypted bytes. This is used on Handshakes for example as defined on the section 17.2.2
pub struct Crypto {
    offset: u64,
    data: Bytes,
}

impl Crypto {
    ///Creates a new Crypto frame with the given `offset` and `data`
    pub fn new(offset: u64, data: Bytes) -> Self {
        Self { offset, data }
    }

    ///Moves the offset by `amount` bytes and returns it's new value
    pub fn move_offset(&mut self, amount: u64) -> u64 {
        self.offset += amount;
        self.offset
    }
    ///Returns a new cursor frame that is equivalent to the slicing of this one by the given `range`. Note that the new one will have it's cursor moved by the range to point to the first element on the sliced data
    pub fn slice(&self, range: Range<usize>) -> Crypto {
        let offset = self.offset as usize;
        Self {
            offset: self.offset + range.start as u64,
            data: self.data.slice(offset + range.start..offset + range.end),
        }
    }

    #[inline]
    ///Creates a new Crypto frame containing the underlying data but going from offset..offset+`amount`
    pub fn slice_until(&self, amount: usize) -> Crypto {
        Self {
            data: self
                .data
                .slice(self.offset as usize..self.offset as usize + amount),
            offset: self.offset,
        }
    }

    #[inline]
    ///Retrieves the length of this stream but without counting the payload
    pub fn size_without_payload(&self) -> usize {
        std::mem::size_of::<u64>() //offset + id, by now both are u64
    }
}

impl Deref for Crypto {
    type Target = Bytes;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl Dencode for Crypto {
    fn encode(&self, buf: &mut BytesMut) {
        self.offset.encode(buf);
        (self.data.len() as u64).encode(buf);
        self.data.encode(buf)
    }
    fn decode(buf: &mut Bytes) -> Result<Self, crate::dencode::DencodeError> {
        let offset = u64::decode(buf)?;
        let len = u64::decode(buf)? as usize;
        let data = buf.copy_to_bytes(len);
        Ok(Self::new(offset, data))
    }
}
