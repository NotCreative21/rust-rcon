// Copyright (c) 2015 [rust-rcon developers]
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

extern crate bufstream;
extern crate podio;

use std::net::{TcpStream, ToSocketAddrs};
use std::io;
use packet::{Packet, PacketType};
use bufstream::BufStream;

pub use error::Result;
pub use error::Error;

mod packet;
mod error;

pub struct Connection {
    stream: BufStream<TcpStream>,
    next_packet_id: i32,
}

const INITIAL_PACKET_ID: i32 = 1;

impl Connection {
    pub fn connect<T: ToSocketAddrs>(address: T, password: &str) -> Result<Connection> {
        let tcp_stream = TcpStream::connect(address)?;
        let mut conn = Connection {
            stream: BufStream::new(tcp_stream),
            next_packet_id: INITIAL_PACKET_ID,
        };

        conn.auth(password)?;

        Ok(conn)
    }

    pub fn cmd(&mut self, cmd: &str) -> Result<i32> {
        Connection::check_len(cmd.len());

        Ok(self.send(PacketType::ExecCommand, cmd)?)
    }


    pub fn response(&mut self, cmd: &str) -> Result<String> {
        Connection::check_len(cmd.len());

        self.send(PacketType::ExecCommand, cmd)?;

        // the server processes packets in order, so send an empty packet and
        // remember its id to detect the end of a multi-packet response
        let end_id = self.send(PacketType::ExecCommand, "")?;

        let mut result = String::new();

        loop {
            let received_packet = self.recv()?;

            if received_packet.get_id() == end_id {
                // This is the response to the end-marker packet
                break;
            }

            result += received_packet.get_body();
        }

        Ok(result)
    }

    fn check_len(length: usize) -> Option<Result<String>> {
        // Minecraft only supports a request payload length of max 1446 byte.
        // However some tests showed that only requests with a payload length
        // of 1413 byte or lower work reliable.
        if length > 1413 {
            return Some(Err(Error::CommandTooLong));
        }
        None
    }

    fn auth(&mut self, password: &str) -> Result<()> {
        self.send(PacketType::Auth, password)?;
        let received_packet = self.recv()?;

        if received_packet.is_error() {
            Err(Error::Auth)
        } else {
            Ok(())
        }
    }

    fn send(&mut self, ptype: PacketType, body: &str) -> io::Result<i32> {
        let id = self.generate_packet_id();

        let packet = Packet::new(id, ptype, body.into());
        packet.serialize(&mut self.stream)?;

        Ok(id)
    }

    fn recv(&mut self) -> io::Result<Packet> {
        Packet::deserialize(&mut self.stream)
    }

    fn generate_packet_id(&mut self) -> i32 {
        let id = self.next_packet_id;

        // only use positive ids as the server uses negative ids to signal
        // a failed authentication request
        self.next_packet_id =
            self.next_packet_id.checked_add(1).unwrap_or(INITIAL_PACKET_ID);

        id
    }
}
