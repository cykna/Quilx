use std::net::SocketAddr;

use bytes::Bytes;

use crate::{
    connections::{ConnectionError, QuicPacket},
    dencode::Dencode,
    frames::{self, Crypto},
};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq)]
///Represents the State of a Connection
pub enum ConnectionState {
    ///Weather the connection requires handshake
    Handshake,
    ///If the connection is Established and can send data safely
    Established,
}

#[derive(Debug)]
///A connection, at least in this implementation, is an End, which is used only to parse and execute the content that is being received and to send something else.
///It doesn't send data and receive data on it's own, but instead receives from the EndPoint it's within
pub struct QuicConnection {
    target: SocketAddr,
    state: ConnectionState,
}

impl QuicConnection {
    ///Creates a new Connection with the given `target` and awaiting for a Handshake
    pub fn new(target: SocketAddr) -> Self {
        Self {
            target,
            state: ConnectionState::Handshake,
        }
    }

    pub fn state(&self) -> ConnectionState {
        self.state
    }

    ///Retrieves the initial packet used to start the establishment of a handshake
    pub fn retrieve_initial(&self) -> QuicPacket {
        let mut initial = QuicPacket::initial();
        initial.push_frame(&frames::Frame::new_crypto(Crypto::new(
            0,
            Bytes::from("Hello World"),
        )));
        initial.fill_padding();
        initial
    }

    pub fn retrieve_handshake(&self) -> QuicPacket {
        let mut handshake = QuicPacket::handshake();
        handshake.push_frame(&frames::Frame::new_ping());
        handshake
    }

    ///Handles the given packet assuming this connection is at HandshakeState and returns the
    ///packets to be sent back
    pub fn handle_initial(
        &mut self,
        packet: QuicPacket,
    ) -> Result<Vec<QuicPacket>, ConnectionError> {
        assert!(self.state == ConnectionState::Handshake);
        let QuicPacket::Initial(initial) = packet else {
            return Err(ConnectionError::UnexpectedContent);
        };

        let mut packet = QuicPacket::handshake();
        let mut initial = QuicPacket::initial();
        initial.fill_padding();
        packet.push_frame(&frames::Frame::Ping);
        let out = vec![initial, packet];
        Ok(out)
    }

    pub fn receive(&mut self, bytes: &mut Bytes) {
        let packet = QuicPacket::decode(bytes);
        println!("{packet:?}");
    }
}
