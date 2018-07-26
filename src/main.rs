extern crate tokio;

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

struct RegistrationMessage {
    client_name: String,
}
impl RegistrationMessage {
    fn new<T: Into<String>>(client_name: T) -> RegistrationMessage {
        RegistrationMessage {
            client_name: client_name.into(),
        }
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
    tokio::run(connect(CONTROL_SERVER_PORT).map(|_| ()).map_err(|_| ()));
}
