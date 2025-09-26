use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::dencode::Dencode;

///Enum determining who defined initialized a specific STREAM frame
#[derive(Debug)]
#[repr(u8)]
pub enum InitiatorType {
    Client,
    Server,
}
///Enum determining if a specific STREAM frame is Bi or Uni directional
#[derive(Debug)]
#[repr(u8)]
pub enum StreamType {
    UniDirectional,
    BiDirectional,
}

#[derive(Clone, Copy)]
///Struct used to determine the ID of a STREAM frame, based on QUIC specification
pub struct StreamId(u64);
impl StreamId {
    #[inline]

    ///Creates a new StreamId with the provided `index`. Note it will suppose `index` is a raw value which is correct on following what is said
    ///on the specification
    pub unsafe fn from_raw(index: u64) -> StreamId {
        Self(index)
    }

    ///Creates a new Stream Id with the provided `initiator` `direction` and `index`. The `index` is actually a value from 0..2^62 used to track
    ///the actual index of some STREAM frame
    pub fn new(initiator: InitiatorType, direction: StreamType, index: u64) -> Self {
        Self((index << 2) | ((initiator as u64) << 1) | direction as u64)
    }

    ///Retrieves the actual index of this ID
    pub fn index(&self) -> u64 {
        self.0 >> 2
    }

    ///Retrieves weather this stream id is uni or bidirectional
    pub fn stream_type(&self) -> StreamType {
        match (self.0 >> 1) & 0b1 {
            0 => StreamType::UniDirectional,
            1 => StreamType::BiDirectional,
            _ => unreachable!(),
        }
    }

    ///Retrieves who initializes the stream with this stream id
    pub fn initiator_type(&self) -> InitiatorType {
        match self.0 & 0b1 {
            0 => InitiatorType::Client,
            1 => InitiatorType::Server,
            _ => unreachable!(),
        }
    }

    ///Retrieves the raw value of this StreamID
    pub fn raw(&self) -> u64 {
        self.0
    }
}

impl Dencode for StreamId {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u64(self.0);
    }
    fn decode(buf: &mut Bytes) -> Result<Self, crate::dencode::DencodeError> {
        let raw = buf.get_u64();
        let raw_value = unsafe { Self::from_raw(raw) };
        Ok(raw_value)
    }
}

impl std::fmt::Debug for StreamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamId")
            .field("index", &self.index())
            .field("initiator", &self.initiator_type())
            .field("direction", &self.stream_type())
            .finish()
    }
}
