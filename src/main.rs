mod dencode;
use std::{collections::VecDeque, net::SocketAddr};

use bytes::Bytes;

use crate::{dencode::Dencode, frames::Stream};

mod connections;
mod frames;

#[derive(Debug)]
pub enum EndPointError {
    NoStream,
    NoTarget,
    Io(std::io::Error),
}

pub struct QuicEndpoint {
    addr: Option<SocketAddr>,
    udp: tokio::net::UdpSocket,
    queue: VecDeque<frames::Frame>,
}

impl QuicEndpoint {
    pub async fn new(addr: SocketAddr, target: Option<SocketAddr>) -> std::io::Result<Self> {
        Ok(Self {
            addr: target,
            udp: tokio::net::UdpSocket::bind(addr).await?,
            queue: VecDeque::new(),
        })
    }

    pub fn append_frame(&mut self, frame: frames::Frame) {
        self.queue.push_front(frame);
    }

    ///Sends the next stream on the queue and returns the amount of bytes sent
    pub async fn send(&mut self) -> Result<usize, EndPointError> {
        let Some(ref target) = self.addr else {
            return Err(EndPointError::NoTarget);
        };
        let next = self.queue.pop_front().ok_or(EndPointError::NoStream)?;
        let mut content = Vec::with_capacity(1200);
        content.resize(1200, 0);
        let amount = next.encode(&mut content);
        self.udp
            .send_to(&content[..amount], target)
            .await
            .map_err(EndPointError::Io)
    }

    ///Receives the incomming bytes and converts them into a stream vector
    pub async fn recv(&mut self) -> std::io::Result<Vec<frames::Frame>> {
        let mut buf = Vec::with_capacity(1200);
        buf.resize(1200, 0);
        let byte_amount = self.udp.recv(&mut buf).await?;
        let mut out = Vec::new();
        let mut idx = 0;
        while idx < byte_amount {
            let (decoded, size) = frames::Frame::decode(&buf[idx..byte_amount]).unwrap();
            idx += size;
            out.push(decoded);
        }
        Ok(out)
    }
}

#[tokio::main]
async fn main() {
    let rx = tokio::spawn(async move {
        let mut receiver = QuicEndpoint::new("0.0.0.0:5000".parse().unwrap(), None)
            .await
            .unwrap();
        while let Ok(buf) = receiver.recv().await {
            println!("{buf:#?}");
        }
    });

    let tx = tokio::spawn(async move {
        let mut writer = QuicEndpoint::new(
            "0.0.0.0:5001".parse().unwrap(),
            Some("0.0.0.0:5000".parse().unwrap()),
        )
        .await
        .unwrap();
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
    });
    rx.await.unwrap();
    tx.await.unwrap();
}
