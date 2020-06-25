use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::io::Result;
use std::net::UdpSocket;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::{thread, time};

static MULTI_CAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 1);

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum RequestType {
    Idle = 0,
    Ping = 1,
    Pong = 2,
    Transmitting = 3,
}

impl ToString for RequestType {
    fn to_string(&self) -> String {
        match self {
            RequestType::Idle => String::from("0"),
            RequestType::Ping => String::from("1"),
            RequestType::Pong => String::from("2"),
            RequestType::Transmitting => String::from("3"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Request {
    pub username: String,
    pub request_state: RequestType,
}

fn generate_fake_data() -> String {
    let data = r#"
    {
        "username": "John Doe",
        "request_state": 1
    }"#;
    data.to_string()
}

pub fn listen() -> Result<()> {
    let socket_address: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 9778);
    let bind_addr = Ipv4Addr::new(0, 0, 0, 0);
    let socket = UdpSocket::bind(socket_address)?;
    println!("Listening on: {}", socket.local_addr().unwrap());
    socket.join_multicast_v4(&MULTI_CAST_ADDR, &bind_addr)?;
    // TODO turn this back on when working from multiple PCs
    // socket.set_multicast_loop_v4(false)?;
    // set up message buffer
    loop {
        let mut buf = [0; 120];

        let (amt, origin) = socket.recv_from(&mut buf)?;
        let buf = &mut buf[..amt];
        let message = String::from_utf8(buf.to_vec()).unwrap();
        let peer_info: Request = serde_json::from_str(&message)?;
        println!("{}", peer_info.username);
    }
}

pub fn cast() -> Result<()> {
    let socket_address: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);
    let socket = UdpSocket::bind(socket_address)?;
    socket.connect(SocketAddrV4::new(MULTI_CAST_ADDR, 9778))?;
    let data = generate_fake_data();
    println!("\n[broadcasting at {} ]", socket.local_addr().unwrap());
    // loop {
    socket.send(data.as_bytes())?;
    thread::sleep(time::Duration::from_secs(2));
    // }
    Ok(())
}

pub fn become_discoverable() {
    thread::spawn(|| {
        listen().unwrap();
    });
    cast().unwrap();
}
