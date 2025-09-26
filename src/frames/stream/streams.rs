use std::{
    ops::{Deref, Range},
    sync::atomic::AtomicU64,
};

use bytes::{Buf, Bytes, BytesMut};

use crate::dencode::Dencode;

use super::stream_id::StreamId;

static STREAM_INDEX: AtomicU64 = AtomicU64::new(0);

fn next_index() -> u64 {
    STREAM_INDEX.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[derive(Debug)]
///A contiguous byte memory which contains an ID and can be sent out of order. It represents the STREAM frame on QUIC specification
pub struct Stream {
    ///The actual contents to be sent
    pub(crate) content: Bytes,
    ///The ID of this stream. Read https://datatracker.ietf.org/doc/html/rfc9000 at 2.0 to 2.1 to understand better about this field
    pub(crate) id: StreamId,
    ///The offset the contents are being at. When something is sent, as it might be out of order, we send the offset to know where to start reading from
    pub(crate) offset: u64,
}

impl Stream {
    pub fn new(id: StreamId, offset: u64, content: Bytes) -> Self {
        Self {
            content,
            id,
            offset,
        }
    }

    #[inline]
    pub fn from_raw(id: u64, offset: u64, bytes: Bytes) -> Self {
        Self {
            content: bytes,
            id: unsafe { StreamId::from_raw(id) },
            offset,
        }
    }
    #[inline]
    ///Creates a new UniDirectional Stream with the given `bytes`
    pub fn new_unidirectional(initiator: super::stream_id::InitiatorType, bytes: Bytes) -> Self {
        Self {
            content: bytes,
            id: StreamId::new(
                initiator,
                super::stream_id::StreamType::UniDirectional,
                next_index(),
            ),
            offset: 0,
        }
    }

    #[inline]
    ///Creates a new UniDirectional Stream with the given `bytes`
    pub fn new_bidirectional(initiator: super::stream_id::InitiatorType, bytes: Bytes) -> Self {
        Self {
            content: bytes,
            id: StreamId::new(
                initiator,
                super::stream_id::StreamType::BiDirectional,
                next_index(),
            ),
            offset: 0,
        }
    }

    #[inline]
    ///Creates a new Stream frame containing the underlying data but sliced based on the given `range` starting from the current offset
    pub fn slice(&self, range: Range<usize>) -> Stream {
        let offset = self.offset as usize;
        Self {
            content: self.content.slice(offset + range.start..range.end + offset),
            id: self.id,
            offset: self.offset + range.start as u64,
        }
    }

    #[inline]
    ///Creates a new stream containing the underlying data but going from offset..offset+`amount`
    pub fn slice_until(&self, amount: usize) -> Stream {
        let offset = self.offset as usize;
        Self {
            content: self.content.slice(offset..amount + offset),
            id: self.id,
            offset: self.offset,
        }
    }
    #[inline]
    ///Moves the offset by `amount` bytes and returns it's new position
    pub fn move_offset(&mut self, amount: u64) -> u64 {
        self.offset += amount;
        self.offset
    }

    #[inline]
    ///Retrieves the length of this stream but without counting the payload
    pub fn size_without_payload(&self) -> usize {
        std::mem::size_of::<u64>() + std::mem::size_of::<u64>() //offset + id, by now both are u64
    }
}

impl Deref for Stream {
    type Target = Bytes;
    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl Dencode for Stream {
    fn encode(&self, buf: &mut BytesMut) {
        self.id.encode(buf);
        self.offset.encode(buf);
        (self.content.len() as u64).encode(buf);
        self.content.encode(buf);
    }
    fn decode(buf: &mut Bytes) -> Result<Self, crate::dencode::DencodeError> {
        let id = StreamId::decode(buf)?;
        let stream_offset = u64::decode(buf)?;

        let len = u64::decode(buf)? as usize;
        let bytes = buf.copy_to_bytes(len);

        Ok(Self::new(id, stream_offset, bytes))
    }
}
