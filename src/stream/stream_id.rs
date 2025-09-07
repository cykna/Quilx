#[derive(Debug)]
#[repr(u8)]
pub enum InitiatorType {
    Client,
    Server,
}
#[derive(Debug)]
#[repr(u8)]
pub enum StreamType {
    UniDirectional,
    BiDirectional,
}

pub struct StreamId(u64);
impl StreamId {
    #[inline]

    ///Creates a new StreamId with the provided `index`
    pub fn from_raw(index: u64) -> StreamId {
        Self(index)
    }

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

    pub fn raw(&self) -> u64 {
        self.0
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
