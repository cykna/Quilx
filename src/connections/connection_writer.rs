use bytes::{BufMut, Bytes, BytesMut};

use crate::{
    connections::{LongHeaderType, QuicPacket},
    dencode::Dencode,
    frames,
};

#[derive(Debug)]
///A Struct used mainly to simple write and headers and packets as well as modify them as necessary for the next write
pub struct ConnectionWriter {
    main_buffer: BytesMut,
    packets_buffer: BytesMut,
}

impl ConnectionWriter {
    pub fn new() -> Self {
        Self {
            main_buffer: BytesMut::with_capacity(1200),
            packets_buffer: BytesMut::with_capacity(1200),
        }
    }
    ///Writes the given `packet` and modifies it's frames to handle a new write
    pub fn write_packet(&mut self, packet: &mut QuicPacket) -> Bytes {
        self.main_buffer.clear();
        self.packets_buffer.clear();
        match packet {
            QuicPacket::Long(long) => {
                let hsize = long.header_size();

                for frame in long.frames_mut() {
                    let limit = self.packets_buffer.len().max(hsize) - hsize;
                    if limit < 1200 {
                        match frame {
                            frames::Frame::Padding | frames::Frame::Ping => {
                                frame.encode(&mut self.packets_buffer);
                            }
                            crate::frames::Frame::Crypto(crypt) => {
                                let write_amount = if crypt.len() + limit > 1200 {
                                    1200 - limit
                                } else {
                                    crypt.len()
                                };
                                crypt
                                    .slice(0..write_amount)
                                    .encode(&mut self.packets_buffer);
                                crypt.move_offset(write_amount as u64);
                            }
                            crate::frames::Frame::Stream(stream) => {
                                let write_amount = if stream.len() + limit > 1200 {
                                    1200 - limit
                                } else {
                                    stream.len()
                                };
                                stream
                                    .slice(0..write_amount)
                                    .encode(&mut self.packets_buffer);
                                stream.move_offset(write_amount as u64);
                            }
                        }
                    } else {
                        break;
                    }
                }
                if let Some(frames::Frame::Padding) = long.frames().last()
                    && long.kind() == LongHeaderType::Initial
                {
                    println!("AMOUNT = 1200 - {} - {hsize}", self.packets_buffer.len());
                    let amount = 1200 - self.packets_buffer.len() - hsize;
                    self.packets_buffer.put_bytes(0, amount);
                }
                long.encode_header_with_len(self.packets_buffer.len(), &mut self.main_buffer);
                self.main_buffer.put_slice(&self.packets_buffer);
            }
        };

        self.main_buffer.split().freeze()
    }
}
