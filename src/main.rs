use std::{collections::VecDeque, net::SocketAddr};

use bytes::Bytes;

use crate::stream::Stream;

mod connections;
mod stream;

#[derive(Debug)]
pub enum EndPointError {
    NoStream,
    NoTarget,
    Io(std::io::Error),
}

pub struct QuicEndpoint {
    addr: Option<SocketAddr>,
    udp: tokio::net::UdpSocket,
    queue: VecDeque<stream::Stream>,
}

impl QuicEndpoint {
    pub async fn new(addr: SocketAddr, target: Option<SocketAddr>) -> std::io::Result<Self> {
        Ok(Self {
            addr: target,
            udp: tokio::net::UdpSocket::bind(addr).await?,
            queue: VecDeque::new(),
        })
    }
    ///Appends the given `content` on the streams queue for when being requested to be sent
    pub fn append(&mut self, content: &[u8]) {
        let stream = Stream::new_bidirectional(
            stream::stream_id::InitiatorType::Client,
            Bytes::copy_from_slice(content),
        );
        self.queue.push_back(stream);
    }

    ///Sends the next stream on the queue and returns the amount of bytes sent
    pub async fn send(&mut self) -> Result<usize, EndPointError> {
        if let Some(ref target) = self.addr {
            let next = self.queue.pop_front().ok_or(EndPointError::NoStream)?;
            let mut content = Vec::with_capacity(1200);
            next.serialize_into(&mut content);
            self.udp
                .send_to(&content, target)
                .await
                .map_err(EndPointError::Io)
        } else {
            Err(EndPointError::NoTarget)
        }
    }

    ///Receives the incomming bytes and converts them into a stream vector
    pub async fn recv(&mut self) -> std::io::Result<Vec<Stream>> {
        let mut buf = Vec::with_capacity(1200);
        buf.resize(1200, 0);
        let byte_amount = self.udp.recv(&mut buf).await?;
        let mut out = Vec::new();
        let mut idx = 0;
        while idx < byte_amount {
            let stream_id = u64::from_be_bytes(buf[idx..idx + 8].try_into().unwrap());
            idx += 8;
            let offset = u64::from_be_bytes(buf[idx..idx + 8].try_into().unwrap());
            idx += 8;
            let len = u64::from_be_bytes(buf[idx..idx + 8].try_into().unwrap()) as usize;
            idx += 8;
            let bytes = Bytes::copy_from_slice(&buf[idx..idx + len]);
            idx += len;
            out.push(Stream::from_raw(stream_id, offset, bytes));
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
            writer.append(buf.as_bytes());
            writer.send().await.unwrap();
        }
    });
    rx.await.unwrap();
    tx.await.unwrap();
}
