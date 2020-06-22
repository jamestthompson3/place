use std::io::Result;
use std::net::UdpSocket;
use std::net::{Ipv4Addr, SocketAddrV4};

static MULTI_CAST_ADDR: &str = &"224.0.0.1";
// static inaddr_any: &str = &"127.0.0.1:9778";

fn generate_fake_data() -> String {
    let data = r#"
    {
        "username": "John Doe"
    }"#;
    data.to_string()
}

pub fn listen() -> Result<()> {
    let socket_address: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 9778);
    let multicast_addr = Ipv4Addr::new(224, 0, 0, 1);
    let bind_addr = Ipv4Addr::new(0, 0, 0, 0);
    let socket = UdpSocket::bind(socket_address)?;
    println!("Listening on: {}", socket.local_addr().unwrap());
    socket.join_multicast_v4(&multicast_addr, &bind_addr)?;
    // the message, it will be cut off.
    let mut buf = [0; 60];

    let (amt, origin) = socket.recv_from(&mut buf)?; // Redecla re `buf` as slice of the received data and send reverse data back to origin.
    let buf = &mut buf[..amt];
    println!(
        "RECEIVED: \n{}\nfrom: \n{}",
        String::from_utf8(buf.to_vec()).unwrap(),
        origin
    );
    return Ok(());
}

pub fn cast() -> Result<()> {
    let socket_address: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);
    let multicast_addr = Ipv4Addr::new(224, 0, 0, 1);
    let bind_addr = Ipv4Addr::new(0, 0, 0, 0);
    // TODO somehow bind it differently?
    let socket = UdpSocket::bind(socket_address)?;
    socket.join_multicast_v4(&multicast_addr, &bind_addr)?;
    socket.connect(SocketAddrV4::new(multicast_addr, 9778))?;
    let data = generate_fake_data();
    socket.send(data.as_bytes())?;
    println!("\n[broadcasting at {} ]", socket_address);
    Ok(())
}
