mod dencode;
use std::{
    collections::{HashMap, VecDeque},
    net::SocketAddr,
};

use bytes::{Bytes, BytesMut};

use crate::{connections::Connection, dencode::Dencode, frames::Stream};

mod connections;
mod frames;

#[derive(Debug)]
pub enum EndPointError {
    NoStream,
    NoTarget,
    Io(std::io::Error),
}

pub struct QuicEndpoint {
    connections: HashMap<SocketAddr, Connection>,
    udp: tokio::net::UdpSocket,
    queue: VecDeque<frames::Frame>,
}

impl QuicEndpoint {
    pub async fn new(addr: SocketAddr) -> std::io::Result<Self> {
        Ok(Self {
            connections: HashMap::new(),
            udp: tokio::net::UdpSocket::bind(addr).await?,
            queue: VecDeque::new(),
        })
    }

    ///Attempts to connect this Endpoint with an endpoint with the provided address `addr` and returns the connection generated
    pub fn connect_to(&mut self, addr: SocketAddr) /* -> Result<&Connection, ()>*/ {}

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
            .map_err(EndPointError::Io);
        Ok(slice.len())
    }

    #[inline]
    ///Reads bytes incoming and write them on the given `buf` and returns the amount of bytes written
    pub async fn recv_into(&mut self, buf: &mut BytesMut) -> std::io::Result<usize> {
        self.udp.recv(buf).await
    }

    ///Receives the incomming bytes and converts them into a stream vector
    pub async fn recv(&mut self) -> std::io::Result<Vec<frames::Frame>> {
        let mut buf = BytesMut::with_capacity(1200);
        let (mut buf, read_amount) = {
            let byte_amount = self.udp.recv(&mut buf).await?;
            (buf.split().freeze(), byte_amount)
        };
        let mut out = Vec::new();

        while buf.len() < read_amount {
            println!("{}", buf.len());
            let decoded = frames::Frame::decode(&mut buf).unwrap();
            out.push(decoded);
        }
        Ok(out)
    }
}

#[tokio::main]
async fn main() {
    let rx = tokio::spawn(async move {
        let mut receiver = QuicEndpoint::new("0.0.0.0:5000".parse().unwrap())
            .await
            .unwrap();
        while let Ok(buf) = receiver.recv().await {
            println!("{buf:#?}");
        }
    });

    let tx = tokio::spawn(async move {
        let target = "0.0.0.0:5000".parse().unwrap();
        let mut writer = QuicEndpoint::new("0.0.0.0:5001".parse().unwrap())
            .await
            .unwrap();
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
    });
    rx.await.unwrap();
    tx.await.unwrap();
}
