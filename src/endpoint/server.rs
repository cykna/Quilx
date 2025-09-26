use std::{
    collections::{HashMap, VecDeque},
    net::SocketAddr,
};

use bytes::BytesMut;

use crate::{
    connections::{ConnectionError, ConnectionState, QuicConnection},
    dencode::Dencode,
    endpoint::QuicEnd,
    frames,
};

#[derive(Debug)]
pub enum EndPointError {
    NoStream,
    NoTarget,
    Io(std::io::Error),
}

#[derive(Debug)]
pub struct QuicServer {
    connections: HashMap<SocketAddr, QuicConnection>,
    udp: tokio::net::UdpSocket,
    queue: VecDeque<frames::Frame>,
}

impl QuicServer {
    pub async fn new(addr: SocketAddr) -> Result<Self, EndPointError> {
        let data = Self {
            connections: HashMap::new(),
            udp: tokio::net::UdpSocket::bind(addr)
                .await
                .map_err(EndPointError::Io)?,
            queue: VecDeque::new(),
        };
        Ok(data)
    }
    ///Retrieves the connection with the given `addr`.
    pub fn retrieve_connection(&self, addr: SocketAddr) -> Option<&QuicConnection> {
        self.connections.get(&addr)
    }

    ///Listen for incomming data and sends them to each Connection it's received. If the sender is not recognized,
    ///a handshake is made and tries to resend the content to the one that sent the data. If it fails, the connection is closed
    pub async fn listen(&mut self) -> Result<(), ConnectionError> {
        loop {
            let (ref mut bytes, ty, addr) = match self.recv().await {
                Ok(v) => v,
                Err(e) => return Err(ConnectionError::Io(e)),
            };

            let connection = if let Some(conn) = self.connections.get_mut(&addr) {
                conn.receive(bytes, addr)?;
                conn
            } else {
                let mut conn = QuicConnection::new(addr);
                conn.receive(bytes, addr)?;
                self.connections.insert(addr, conn);
                self.connections.get_mut(&addr).unwrap()
            };

            while let Some(datagram) = connection.pending() {
                if let Err(e) = self.udp.send_to(&datagram, addr).await {
                    connection.set_state(ConnectionState::Closed);
                } else {
                };
            }
        }
    }

    pub fn append_frame(&mut self, frame: frames::Frame) {
        self.queue.push_front(frame);
    }

    ///Sends the next stream on the queue and returns the amount of bytes sent
    pub async fn send(&mut self, addr: SocketAddr) -> Result<usize, EndPointError> {
        let next = self.queue.pop_front().ok_or(EndPointError::NoStream)?;
        let mut content = BytesMut::with_capacity(1200);
        next.encode(&mut content);

        let slice = &content.split().freeze()[..];

        self.udp
            .send_to(slice, addr)
            .await
            .map_err(EndPointError::Io)?;
        Ok(slice.len())
    }
}

impl QuicEnd for QuicServer {
    fn socket(&mut self) -> &mut tokio::net::UdpSocket {
        &mut self.udp
    }
}
