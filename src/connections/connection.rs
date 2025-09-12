use std::net::SocketAddr;

pub enum ConnectionState {}

pub struct QuicConnection {
    target: SocketAddr,
}

impl QuicConnection {
    pub fn new(target: SocketAddr) -> Self {
        Self { target }
    }
}
