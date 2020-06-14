use std::io::Result;
use std::net::UdpSocket;
use std::net::{Ipv4Addr, SocketAddrV4};

static RECV_ADDRESS: &str = &"127.0.0.1:9777";
// static INADDR_ANY: &str = &"127.0.0.1:9778";

fn generate_fake_data() -> String {
    let data = r#"
    {
        "username": "John Doe"
    }"#;
    data.to_string()
}

pub fn listen() -> Result<()> {
    let INADDR_ANY: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);
    let socket = UdpSocket::bind(RECV_ADDRESS)?;
    println!("Listening on: {}", socket.local_addr().unwrap());
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
    let INADDR_ANY: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);
    // TODO somehow bind it differently?
    let socket = UdpSocket::bind(INADDR_ANY)?;
    socket.set_broadcast(true)?;
    socket.connect(RECV_ADDRESS)?;
    let data = generate_fake_data();
    socket.send(data.as_bytes())?;
    Ok(())
}
