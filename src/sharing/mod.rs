use std::io::Result;
use std::net::UdpSocket;
use std::net::{Ipv4Addr, SocketAddrV4};

static MULTI_CAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 1);

fn generate_fake_data() -> String {
    let data = r#"
    {
        "username": "John Doe",
        "request_state": "1"
    }"#;
    data.to_string()
}

pub fn listen() -> Result<()> {
    let socket_address: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 9778);
    let bind_addr = Ipv4Addr::new(0, 0, 0, 0);
    let socket = UdpSocket::bind(socket_address)?;
    println!("Listening on: {}", socket.local_addr().unwrap());
    socket.join_multicast_v4(&MULTI_CAST_ADDR, &bind_addr)?;
    // set up message buffer
    let mut buf = [0; 60];

    let (amt, origin) = socket.recv_from(&mut buf)?;
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
    let socket = UdpSocket::bind(socket_address)?;
    socket.connect(SocketAddrV4::new(MULTI_CAST_ADDR, 9778))?;
    let data = generate_fake_data();
    socket.send(data.as_bytes())?;
    println!("\n[broadcasting at {} ]", socket.local_addr().unwrap());
    Ok(())
}
