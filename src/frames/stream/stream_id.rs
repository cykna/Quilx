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
    fn encode(&self, buf: &mut [u8]) -> usize {
        let bytes = self.0.to_be_bytes();
        buf[0..bytes.len()].copy_from_slice(&bytes);
        bytes.len()
    }
    fn decode(buf: &[u8]) -> Result<(Self, usize), crate::dencode::DencodeError> {
        let raw_value =
            unsafe { Self::from_raw(u64::from_be_bytes(buf[0..8].try_into().unwrap())) };
        Ok((raw_value, 8))
    }
}

impl std::fmt::Debug for StreamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamId")
            .field("index", &self.index())
            .field("initiator", &self.stream_type())
            .field("direction", &self.stream_type())
            .finish()
    }
}
