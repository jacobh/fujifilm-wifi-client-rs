extern crate byteorder;
extern crate tokio;

use byteorder::ByteOrder;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::prelude::*;
// use tokio::reactor::{Reactor, Handle};
use tokio::net::TcpStream;

const CONTROL_SERVER_PORT: u16 = 55740;
const ASYNC_RESPONSE_SERVER_PORT: u16 = 55741;
const JPEG_STREAM_SERVER_PORT: u16 = 55742;
const HOST: &'static str = "192.168.0.1";

fn connect(port: u16) -> impl Future<Item = TcpStream, Error = tokio::io::Error> {
    let host_addr: Ipv4Addr = HOST.parse().unwrap();
    let host_socket_addr = SocketAddr::new(IpAddr::V4(host_addr), port);

    TcpStream::connect(&host_socket_addr)
        .map(|stream| {
            println!("connection established");
            stream
        })
        .map_err(|e| {
            println!("failed to connect!");
            e
        })
}

fn fuji_send<T: Into<Vec<u8>>>(
    stream: TcpStream,
    data: T,
) -> impl Future<Item = TcpStream, Error = tokio::io::Error> {
    let data = data.into();
    let data_length = data.len() as u32;
    let length_bytes = {
        let mut buf = [0; 4];
        byteorder::LittleEndian::write_u32(&mut buf, data_length);
        buf
    };

    tokio::io::write_all(stream, length_bytes)
        .and_then(|(stream, _)| tokio::io::write_all(stream, data))
        .map(|(stream, _)| stream)
}

fn fuji_receive(
    stream: TcpStream,
) -> impl Future<Item = (TcpStream, Vec<u8>), Error = tokio::io::Error> {
    tokio::io::read_exact(stream, [0u8; 4]).and_then(|(stream, length_bytes)| {
        let total_length = byteorder::LittleEndian::read_u32(&length_bytes);
        if total_length < 4 {
            println!("Invalid message, length less than header");
        }
        let body_length = total_length - 4;

        tokio::io::read_exact(stream, vec![0u8; body_length as usize])
    })
}

struct RegistrationMessage {
    client_name: String,
}
impl RegistrationMessage {
    fn new<T: Into<String>>(client_name: T) -> RegistrationMessage {
        RegistrationMessage {
            client_name: client_name.into(),
        }
    }
    fn bytes(&self) -> Vec<u8> {
        let mut x = self.header();
        x.append(&mut self.client_name());
        x
    }
    fn header(&self) -> Vec<u8> {
        vec![
            0x01, 0x00, 0x00, 0x00, 0xf2, 0xe4, 0x53, 0x8f, 0xad, 0xa5, 0x48, 0x5d, 0x87, 0xb2,
            0x7f, 0x0b, 0xd3, 0xd5, 0xde, 0xd0, 0x02, 0x78, 0xa8, 0xc0,
        ]
    }
    fn client_name(&self) -> Vec<u8> {
        let max_characters = 26;

        let mut client_name_bytes = vec![];
        for i in 0..max_characters {
            let byte = self.client_name.as_bytes().get(i).unwrap_or(&0);
            client_name_bytes.push(*byte);
            client_name_bytes.push(0);
        }
        client_name_bytes
    }
}

fn main() {
    tokio::run(
        connect(CONTROL_SERVER_PORT)
            .and_then(|stream| {
                println!("got stream");
                let registration_message = RegistrationMessage::new("JacobsLaptop");
                fuji_send(stream, registration_message.bytes())
            })
            .and_then(|stream| {
                println!("sent bytes");
                fuji_receive(stream)
            })
            .map(|(stream, resp)| {
                println!("got a message back");
                println!("{:?}", resp);
                ()
            })
            .map_err(|e| println!("{:?}", e)),
    );
}
