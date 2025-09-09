//dencode stands for Decode/Encode(I know, terrible name)

use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Debug)]
pub enum DencodeError {}

///Trait used for encoding/decoding data on Quilx
pub trait Dencode: Sized {
    ///Encodes this type into `buf`, assuming `buf` is sliced, so it begins at 0 and returns the amount of bytes written
    fn encode(&self, buf: &mut BytesMut);
    ///Tries to encode the given `buf` into Self. Returns the value decoded and how many bytes were read
    fn decode(buf: &mut Bytes) -> Result<Self, DencodeError>;
}

impl Dencode for bytes::Bytes {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_slice(&self[..]);
    }
    fn decode(buf: &mut Bytes) -> Result<Self, DencodeError> {
        let len = buf.len();
        Ok(buf.clone())
    }
}

impl Dencode for u8 {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u8(*self);
    }
    fn decode(buf: &mut Bytes) -> Result<Self, DencodeError> {
        Ok(buf.get_u8())
    }
}
impl Dencode for u16 {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u16(*self);
    }
    fn decode(buf: &mut Bytes) -> Result<Self, DencodeError> {
        Ok(buf.get_u16())
    }
}
impl Dencode for u32 {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u32(*self);
    }
    fn decode(buf: &mut Bytes) -> Result<Self, DencodeError> {
        Ok(buf.get_u32())
    }
}
impl Dencode for u64 {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u64(*self);
    }
    fn decode(buf: &mut Bytes) -> Result<Self, DencodeError> {
        Ok(buf.get_u64())
    }
}
