mod dencode;
mod helpers;
use std::{
    collections::{HashMap, VecDeque},
    net::SocketAddr,
};

use bytes::{Buf, Bytes, BytesMut};

use crate::{
    connections::{DataType, InitialPacket, QuicConnection, QuicPacket},
    dencode::Dencode,
    frames::{Crypto, Stream, stream_id::StreamId},
};

mod connections;
mod frames;

#[derive(Debug)]
pub enum EndPointError {
    NoStream,
    NoTarget,
    Io(std::io::Error),
}

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

    ///Encoded and sends the packet immediatly, assuming it follows the deffinitions set on the specifications for the MTU
    pub async fn send_immediatly(
        &mut self,
        packet: QuicPacket,
        addr: SocketAddr,
    ) -> std::io::Result<usize> {
        let mut buf = BytesMut::new();
        packet.encode(&mut buf);
        self.udp.send_to(&buf[..], addr).await
    }

    ///Attempts to connect this Endpoint with an endpoint with the provided address `addr` and returns the connection generated
    pub async fn connect_to(&mut self, addr: SocketAddr) -> std::io::Result<()> {
        let mut initial = InitialPacket::new();
        initial.push_frame(frames::Frame::new_crypto(Crypto::new(
            0,
            Bytes::from("Hello World"),
        )));
        initial.fill_padding();
        self.send_immediatly(QuicPacket::Initial(initial), addr)
            .await?;
        let (data, ty, addr) = self.recv().await?;

        println!("{ty:?} {data:?} ue",);
        Ok(())
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
            let (_, addr) = self.udp.recv_from(&mut out).await?;
            (out.split().freeze(), addr)
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
        while let Ok((mut buf, dt, addr)) = receiver.recv().await {
            match dt {
                DataType::Packet => {
                    let data = QuicPacket::decode(&mut buf).unwrap();
                    let frames = data
                        .frames()
                        .into_iter()
                        .filter(|v| !matches!(v, frames::Frame::Padding))
                        .collect::<Vec<_>>();
                    let frames::Frame::Crypto(ref crypto) = frames[0] else {
                        panic!("Not a crypto")
                    };
                    receiver.append_frame(frames::Frame::Stream(Stream::new(
                        StreamId::new(
                            frames::stream_id::InitiatorType::Server,
                            frames::stream_id::StreamType::BiDirectional,
                            0,
                        ),
                        0,
                        Bytes::from("cool your message"),
                    )));
                    receiver.send(addr).await?;
                }
            }
        }
        Ok(()) as Result<(), EndPointError>
    });

    let tx = tokio::spawn(async move {
        let target = "0.0.0.0:5000".parse().unwrap();
        let mut writer = QuicEndpoint::new("0.0.0.0:5001".parse().unwrap())
            .await
            .unwrap();
        writer.connect_to(target).await?;
        let stdin = std::io::stdin();
        let mut buf = String::new();
        while let Ok(s) = stdin.read_line(&mut buf) {
            writer.append_frame(frames::Frame::Stream(Stream::new_unidirectional(
                frames::stream_id::InitiatorType::Client,
                Bytes::copy_from_slice(&buf.as_bytes()[..s]),
            )));
            writer.send(target).await.unwrap();
            buf.clear();
        }
        Ok(()) as std::io::Result<()>
    });
    rx.await.unwrap();
    tx.await.unwrap();
}
