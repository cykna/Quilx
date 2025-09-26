use std::{
    collections::{HashMap, VecDeque},
    net::SocketAddr,
    sync::Arc,
};

use bytes::Bytes;
use tokio::sync::Notify;

use crate::{
    connections::{
        ConnectionError, LongHeader, QuicLongPacket, QuicPacket,
        connection_writer::ConnectionWriter,
    },
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
    ///Weather the connection was closed and won't be able to respond anymore
    Closed,
}

#[derive(Debug)]
///A connection, at least in this implementation, is an End, which is used only to parse and execute the content that is being received and to send something else.
///It doesn't send data and receive data on it's own, but instead receives from the EndPoint it's within
pub struct QuicConnection {
    target: SocketAddr,
    state: ConnectionState,
    notifications: HashMap<SocketAddr, Arc<Notify>>,
    writer: ConnectionWriter,
    frames: VecDeque<QuicPacket>,
}

impl QuicConnection {
    ///Creates a new Connection with the given `target` and awaiting for a Handshake
    pub fn new(target: SocketAddr) -> Self {
        Self {
            notifications: HashMap::new(),
            target,
            state: ConnectionState::Handshake,
            writer: ConnectionWriter::new(),
            frames: VecDeque::new(),
        }
    }

    #[inline]
    ///Sets the state of this connection to be the given `state`
    pub fn set_state(&mut self, state: ConnectionState) {
        self.state = state;
    }

    #[inline]
    ///Retrieves the state of this connection
    pub fn state(&self) -> ConnectionState {
        self.state
    }

    ///Retrieves the initial packet used to start the establishment of a handshake
    pub fn retrieve_initial(&self) -> QuicPacket {
        let mut initial = QuicLongPacket::initial();
        initial.push_frame(frames::Frame::new_crypto(Crypto::new(
            0,
            Bytes::from("Hello World"),
        )));

        QuicPacket::Long(initial)
    }

    pub fn retrieve_handshake(&self) -> QuicLongPacket {
        let mut handshake = QuicLongPacket::handshake();
        handshake.push_frame(frames::Frame::new_ping());
        handshake
    }

    ///If some content on the queue of frames, they are written into this connection buffer and sent back
    pub(crate) fn pending(&mut self) -> Option<Bytes> {
        if self.state == ConnectionState::Closed {
            return None;
        }

        if let Some(ref mut packet) = self.frames.pop_front() {
            Some(self.writer.write_packet(packet))
        } else {
            None
        }
    }

    ///Handles the given packet assuming this connection is at HandshakeState and returns the
    ///packets to be sent back
    pub fn handle_initial(
        &mut self,
        long: LongHeader,
        token: Vec<u8>,
        packet_number: u32,
        packets: Vec<frames::Frame>,
    ) -> Result<(), ConnectionError> {
        let mut packet = QuicLongPacket::handshake();

        packet.push_frame(frames::Frame::Ping);
        self.frames.push_back(QuicPacket::Long(packet));
        println!("{}", packets.len());
        Ok(())
    }

    pub fn receive(
        &mut self,
        bytes: &mut Bytes,
        sender: SocketAddr,
    ) -> Result<(), ConnectionError> {
        if self.state == ConnectionState::Closed {
            return Err(ConnectionError::ClosedConnection);
        }

        let packet = QuicLongPacket::decode(bytes).map_err(ConnectionError::PacketError)?;

        println!("Server = {packet:?}",);
        match packet {
            QuicLongPacket::Handshake {
                long,
                packet_number,
                packets,
            } => {
                if let Some(notification) = self.notifications.get_mut(&sender) {
                    self.state = ConnectionState::Established;
                    notification.notify_waiters();
                    Ok(())
                } else {
                    Err(ConnectionError::NoHandshakeInWait(sender))
                }
            }
            QuicLongPacket::Initial {
                long,
                token,
                packet_number,
                packets,
            } => {
                self.handle_initial(long, token, packet_number, packets)?;
                Ok(())
            }
            kind => todo!("Must implement support for kind {kind:?}"),
        }
    }

    pub(crate) fn create_notification_for(&mut self, target: SocketAddr) -> Arc<Notify> {
        let notify = Arc::new(Notify::new());
        self.notifications.insert(target, notify.clone());
        notify
    }

    ///Function used by clients to simply wait for an incomming handshake packet comming from the provided `target`
    pub(crate) async fn wait_for_handshake(
        &mut self,
        target: SocketAddr,
    ) -> Result<(), ConnectionError> {
        let Some(notification) = self.notifications.get(&target) else {
            return Err(ConnectionError::NoHandshakeInWait(target));
        };
        notification.notified().await;
        Ok(())
    }
}

unsafe impl Send for QuicConnection {}
unsafe impl Sync for QuicConnection {}
