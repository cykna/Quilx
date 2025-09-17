mod connection;
mod packets;
pub use connection::*;
pub use packets::*;

#[derive(Debug)]
///The type of a content received by some connection
pub enum DataType {
    Packet,
}

#[derive(Debug)]
pub enum ConnectionError {
    UnexpectedContent,
}
