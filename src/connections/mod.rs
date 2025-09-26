mod connection;
mod connection_writer;

mod packets;
use std::net::SocketAddr;

pub use connection::*;

pub use packets::*;

use crate::dencode::DencodeError;

#[derive(Debug)]
///The type of a content received by some connection
pub enum DataType {
    Packet,
}

#[derive(Debug)]
pub enum ConnectionError {
    ///Error on parsing a QuicPacket
    PacketError(DencodeError),
    UnexpectedContent,
    ///Weather an operation was attempted to be made on a closed connection
    ClosedConnection,
    ///Represents an error that supposed a connection was waiting a handshake from the address
    NoHandshakeInWait(SocketAddr),
    Io(std::io::Error),
}
