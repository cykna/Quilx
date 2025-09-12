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
    frames::{Crypto, Stream},
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
    pub async fn new(addr: SocketAddr) -> std::io::Result<Self> {
        let data = Self {
            connections: HashMap::new(),
            udp: tokio::net::UdpSocket::bind(addr).await?,
            queue: VecDeque::new(),
        };
        Ok(data)
    }

    ///Encoded and sends the packet immediatly, assuming it follows the deffinitions set on the specifications for the MTU
    pub async fn send_immediatly(&mut self, packet: QuicPacket) -> std::io::Result<usize> {
        let mut buf = BytesMut::new();
        packet.encode(&mut buf);
        self.udp.send(&buf[..]).await
    }

    ///Attempts to connect this Endpoint with an endpoint with the provided address `addr` and returns the connection generated
    pub async fn connect_to(&mut self, addr: SocketAddr) -> std::io::Result<()> {
        self.udp.connect(addr).await?;
        let mut initial = InitialPacket::new();
        initial.push_frame(frames::Frame::new_crypto(Crypto::new(
            0,
            Bytes::from("Hello World"),
        )));
        initial.fill_padding();
        let size = self.send_immediatly(QuicPacket::Initial(initial)).await?;

        let (data, ty) = self.recv().await?;

        println!("{ty:?} {data:?} ue",);
        Ok(())
    }

    pub fn append_frame(&mut self, frame: frames::Frame) {
        self.queue.push_front(frame);
    }

    ///Sends the next stream on the queue and returns the amount of bytes sent
    pub async fn send(&mut self) -> Result<usize, EndPointError> {
        let next = self.queue.pop_front().ok_or(EndPointError::NoStream)?;
        let mut content = BytesMut::with_capacity(1200);
        next.encode(&mut content);

        let slice = &content.split().freeze()[..];

        self.udp.send(slice).await.map_err(EndPointError::Io)?;
        Ok(slice.len())
    }

    #[inline]
    ///Reads bytes incoming and write them on the given `buf` and returns the amount of bytes written
    pub async fn recv_into(&mut self, buf: &mut BytesMut) -> std::io::Result<usize> {
        self.udp.recv(buf).await
    }

    #[inline]
    ///Reads the incomming bytes and returns their packet
    pub async fn recv_packet(&mut self) -> std::io::Result<QuicPacket> {
        let mut buf = BytesMut::with_capacity(1200);
        let read_amount = self.recv_into(&mut buf).await?;
        let mut buf = buf.split().freeze().slice(0..read_amount);
        Ok(QuicPacket::decode(&mut buf).unwrap())
    }

    ///Receives the incomming bytes and checks returns them as well as the type they're for QUIC
    pub async fn recv(&mut self) -> std::io::Result<(Bytes, DataType)> {
        let buf = {
            let mut out = BytesMut::zeroed(1200);
            self.recv_into(&mut out).await?;
            out.split().freeze()
        };

        let dt = match buf[0] {
            //dt == datatype
            n if n >> 7 == 1 => DataType::Packet,
            _ => unimplemented!(),
        };
        Ok((buf, dt))
    }
}

#[tokio::main]
async fn main() {
    let rx = tokio::spawn(async move {
        let mut receiver = QuicEndpoint::new("0.0.0.0:5000".parse().unwrap())
            .await
            .unwrap();
        while let Ok((mut buf, dt)) = receiver.recv().await {
            match dt {
                DataType::Packet => {
                    let data = QuicPacket::decode(&mut buf).unwrap();
                    let frames = data.frames();
                    println!("{frames:?}")
                }
            }
        }
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
            writer.send().await.unwrap();
            buf.clear();
        }
        Ok(()) as std::io::Result<()>
    });
    rx.await.unwrap();
    tx.await.unwrap();
}
