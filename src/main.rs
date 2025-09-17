mod dencode;
mod helpers;
use std::{
    collections::{HashMap, VecDeque},
    net::SocketAddr,
};

use bytes::{Bytes, BytesMut};

use crate::{
    connections::{ConnectionError, DataType, LongHeaderType, QuicConnection, QuicPacket},
    dencode::Dencode,
};

mod connections;
mod frames;

#[derive(Debug)]
pub enum EndPointError {
    NoStream,
    NoTarget,
    Io(std::io::Error),
}

#[derive(Debug)]
pub struct QuicEndpoint {
    connections: HashMap<SocketAddr, QuicConnection>,
    udp: tokio::net::UdpSocket,
    queue: VecDeque<frames::Frame>,
}

impl QuicEndpoint {
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
    ///Encoded and sends the packet immediatly, assuming it follows the deffinitions set on the specifications for the MTU.
    ///If the `packet` size is higher than the defined, the ones that overflowed, won't be sent. If none is passed, then anything is even sent
    pub async fn send_immediatly(
        &mut self,
        packet: &[QuicPacket],
        addr: SocketAddr,
    ) -> std::io::Result<usize> {
        let mut buf = BytesMut::new();
        for pckt in packet {
            if pckt.kind() == LongHeaderType::Initial {
                let mut buf = BytesMut::new();
                pckt.encode(&mut buf);
                self.udp.send_to(&buf[..1200], addr).await?;
            } else {
                if buf.len() >= 1200 {
                    break;
                }
                pckt.encode(&mut buf);
            }
        }
        if buf.len() != 0 {
            self.udp.send_to(&buf[..buf.len().min(1200)], addr).await
        } else {
            Ok(0)
        }
    }

    ///Listen for incomming data and sends them to each Connection it's received. If the sender is not recognized,
    ///a handshake is made
    pub async fn listen(&mut self) -> Result<(), ConnectionError> {
        while let Ok((ref mut bytes, ty, addr)) = self.recv().await {
            dbg!(self.connections.contains_key(&addr), addr);
            if let Some(conn) = self.connections.get_mut(&addr) {
                conn.receive(bytes)
            } else {
                self.create_connection_with(addr, bytes, ty).await?;
            }
        }
        Ok(())
    }

    ///Creates a new connection with the client with the provided `addr` assuming the given `bytes` were it's first ones
    pub async fn create_connection_with(
        &mut self,
        addr: SocketAddr,
        bytes: &mut Bytes,
        ty: DataType,
    ) -> Result<(), ConnectionError> {
        let mut conn = QuicConnection::new(addr);
        match QuicPacket::decode(bytes) {
            Ok(packet) => match ty {
                DataType::Packet => {
                    let handshake_packet = conn.handle_initial(packet).unwrap();
                    self.send_immediatly(&handshake_packet, addr).await.unwrap();
                }
            },
            _ => return Err(ConnectionError::UnexpectedContent),
        }
        self.connections.insert(addr, conn);
        Ok(())
    }

    ///Attempts to connect this Endpoint with an endpoint with the provided address `addr` and returns the connection generated
    pub async fn connect_to(&mut self, addr: SocketAddr) -> std::io::Result<&QuicConnection> {
        let connection = QuicConnection::new(addr);
        let initial = connection.retrieve_initial();
        self.send_immediatly(&[initial], addr).await?;
        self.connections.insert(addr, connection);
        Ok(self.connections.get(&addr).unwrap())
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
    #[inline]
    ///Reads the incomming bytes and returns their packet as well as the address of the one sent
    pub async fn recv_packet(&mut self) -> std::io::Result<(QuicPacket, SocketAddr)> {
        let mut buf = BytesMut::with_capacity(1200);
        let (read_amount, addr) = self.udp.recv_from(&mut buf).await?;
        let buf = &mut buf.split().freeze().slice(..read_amount);
        Ok((QuicPacket::decode(buf).unwrap(), addr))
    }

    ///Receives the incomming bytes and checks returns them as well as the type they're for QUIC and the address of the one who sent them
    pub async fn recv(&mut self) -> std::io::Result<(Bytes, DataType, SocketAddr)> {
        let (buf, addr) = {
            let mut out = BytesMut::zeroed(1200);
            let (size, addr) = self.udp.recv_from(&mut out).await?;

            (out.split().freeze().slice(..size), addr)
        };

        //dt == datatype
        let dt = match buf[0] {
            n if n >> 7 == 1 => DataType::Packet,
            n => unimplemented!("ty '{n}' not implemented yet. Check if the content is correct"),
        };
        Ok((buf, dt, addr))
    }
}

#[tokio::main]
async fn main() {
    let rx = tokio::spawn(async move {
        let mut receiver = QuicEndpoint::new("0.0.0.0:5000".parse().unwrap()).await?;
        receiver.listen().await.unwrap();

        Ok(()) as Result<(), EndPointError>
    });

    let tx = tokio::spawn(async move {
        let target = "0.0.0.0:5000".parse().unwrap();
        let mut writer = QuicEndpoint::new("0.0.0.0:5001".parse().unwrap())
            .await
            .unwrap();
        writer.connect_to(target).await?;
        writer.listen().await.unwrap();

        Ok(()) as std::io::Result<()>
    });
    rx.await.unwrap().unwrap();
    tx.await.unwrap().unwrap();
}
