//dencode stands for Decode/Encode(I know, terrible name)

use bytes::Bytes;

pub enum DencodeError {}

///Trait used for encoding/decoding data on Quilx
pub trait Dencode: Sized {
    ///Encodes this type into `buf`, assuming `buf` is sliced, so it begins at 0 and returns the amount of bytes written
    fn encode(&self, buf: &mut [u8]) -> usize;
    ///Tries to encode the given `buf` into Self. Returns the value decoded and how many bytes were read
    fn decode(buf: &[u8]) -> Result<(Self, usize), DencodeError>;
}

macro_rules! dencode_numeric {
    ($($kind:ty),*) => {
        $(
            impl Dencode for $kind {
                fn encode(&self, buf: &mut [u8]) -> usize {
                    let bytes = self.to_be_bytes();
                    buf[0..8].copy_from_slice(&bytes);
                    bytes.len()
                }
                fn decode(buf: &[u8]) -> Result<(Self, usize), DencodeError> {
                    let bytes = &buf[0..std::mem::size_of::<$kind>()];
                    Ok((<$kind>::from_be_bytes(bytes.try_into().unwrap()), bytes.len()))
                }
            }
        )*
    };
}

dencode_numeric!(u8, u16, u32, u64, i8, i16, i32, i64);

impl Dencode for bytes::Bytes {
    fn encode(&self, buf: &mut [u8]) -> usize {
        buf[..self.len()].copy_from_slice(&self);
        self.len()
    }
    fn decode(buf: &[u8]) -> Result<(Self, usize), DencodeError> {
        Ok((Bytes::copy_from_slice(buf), buf.len()))
    }
}
