use std::net::SocketAddr;

use bytes::{Bytes, BytesMut};
use tokio::net::UdpSocket;

use crate::{
    connections::{ConnectionError, QuicConnection, QuicPacket},
    dencode::Dencode,
    endpoint::{EndPointError, QuicEnd},
};

pub struct QuicClient {
    socket: UdpSocket,
    target: SocketAddr,
    connection: QuicConnection,
}

impl QuicClient {
    pub async fn new(addr: SocketAddr, target: SocketAddr) -> Result<Self, EndPointError> {
        Ok(Self {
            socket: UdpSocket::bind(addr).await.map_err(EndPointError::Io)?,
            target,
            connection: QuicConnection::new(target),
        })
    }

    ///Awaits for some content from the server, when some arrives, returns it.
    pub async fn wait_content_from_server(&mut self) -> std::io::Result<Bytes> {
        let mut buffer = BytesMut::with_capacity(1200);
        buffer.resize(1200, 0);
        loop {
            let (amount, sender) = self.socket.recv_from(&mut buffer).await?;

            if sender == self.target {
                buffer.truncate(amount);
                return Ok(buffer.split().freeze());
            }
        }
    }

    ///Attempts to connect this Endpoint with an endpoint with the provided address `addr` and returns the connection generated
    pub async fn connect_to(&mut self) -> std::io::Result<Result<(), ConnectionError>> {
        let initial = self.connection.retrieve_initial();
        self.send_immediatly(&[initial], self.target).await?;
        let mut bytes = self.wait_content_from_server().await?;
        Ok({
            let packet = QuicPacket::decode(&mut bytes).map_err(ConnectionError::PacketError);
            println!("Client: {packet:#?}");
            Ok(())
        })
    }
}

impl QuicEnd for QuicClient {
    fn socket(&mut self) -> &mut UdpSocket {
        &mut self.socket
    }
}
