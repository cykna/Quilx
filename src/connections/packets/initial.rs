use crate::{connections::LongHeader, frames};

pub struct InitialPacket {
    long: LongHeader,
    ///The token received by NEW_TOKEN frame, or at a retry packet. Note that this is empty on trying to implement a handshake
    token: Vec<u8>,
    packets: Vec<frames::Frame>,
}
