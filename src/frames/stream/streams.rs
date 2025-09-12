use std::{io::Cursor, ops::Deref, sync::atomic::AtomicU64};

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
