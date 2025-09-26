mod server;
use std::net::SocketAddr;

use bytes::{Bytes, BytesMut};
pub use server::*;
mod client;
pub use client::*;
use tokio::net::UdpSocket;

use crate::{
    connections::{DataType, LongHeaderType, QuicPacket},
    dencode::Dencode,
};

///A Trait for generic implementations that both ends MUST implement. Simply for not DRY
pub trait QuicEnd {
    fn socket(&mut self) -> &mut UdpSocket;
    ///Encoded and sends the packet immediatly, assuming it follows the deffinitions set on the specifications for the MTU.
    ///If the `packet` size is higher than the defined, the ones that overflowed, won't be sent. If none is passed, then anything is even sent
    async fn send_immediatly(
        &mut self,
        packet: &[QuicPacket],
        addr: SocketAddr,
    ) -> std::io::Result<usize> {
        let mut buf = BytesMut::new();
        for pckt in packet {
            if let QuicPacket::Long(long) = pckt
                && long.kind() == LongHeaderType::Initial
            {
                let mut buf = BytesMut::new();
                pckt.encode(&mut buf);
                self.socket().send_to(&buf[..1200], addr).await?;
            } else {
                if buf.len() >= 1200 {
                    break;
                }
                pckt.encode(&mut buf);
            }
        }
        if buf.len() != 0 {
            self.socket()
                .send_to(&buf[..buf.len().min(1200)], addr)
                .await
        } else {
            Ok(0)
        }
    }

    ///Reads the incomming bytes and returns their packet as well as the address of the one sent
    async fn recv_packet(&mut self) -> std::io::Result<(QuicPacket, SocketAddr)> {
        let mut buf = BytesMut::with_capacity(1200);
        let (read_amount, addr) = self.socket().recv_from(&mut buf).await?;
        let buf = &mut buf.split().freeze().slice(..read_amount);
        Ok((QuicPacket::decode(buf).unwrap(), addr))
    }

    ///Receives the incomming bytes and checks returns them as well as the type they're for QUIC and the address of the one who sent them
    async fn recv(&mut self) -> std::io::Result<(Bytes, DataType, SocketAddr)> {
        let (buf, addr) = {
            let mut out = BytesMut::zeroed(1200);
            let (size, addr) = self.socket().recv_from(&mut out).await?;
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
