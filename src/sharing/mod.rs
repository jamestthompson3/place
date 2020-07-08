use anyhow::Result;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time};

static MULTI_CAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 1);
// Make sure you can only have one PeerCollection at a time
static PEER_COLLECTION_IN_USE: AtomicBool = AtomicBool::new(false);

pub struct PeerCollection {
    found: HashMap<String, String>,
}

impl PeerCollection {
    pub fn new() -> Result<PeerCollection, Error> {
        if PEER_COLLECTION_IN_USE.compare_and_swap(false, true, Ordering::SeqCst) == false {
            Ok(PeerCollection {
                found: HashMap::new(),
            })
        } else {
            Err(Error::AlreadyInUse)
        }
    }
}

impl Drop for PeerCollection {
    fn drop(&mut self) {
        PEER_COLLECTION_IN_USE.store(false, Ordering::SeqCst);
    }
}

// The various error cases that may be encountered while using this library.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    AlreadyInUse,
}

fn generate_fake_data() -> String {
    String::from("John Doe")
}

pub fn listen() -> Result<()> {
    let socket_address: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 9778);
    let bind_addr = Ipv4Addr::new(0, 0, 0, 0);
    let socket = UdpSocket::bind(socket_address)?;
    println!("Listening on: {}", socket.local_addr().unwrap());
    socket.join_multicast_v4(&MULTI_CAST_ADDR, &bind_addr)?;
    // set up message buffer
    loop {
        let mut buf = [0; 120];

        let (amt, origin) = socket.recv_from(&mut buf)?;
        let buf = &mut buf[..amt];
        let message = String::from_utf8(buf.to_vec()).unwrap();
        println!("{}, {}", message, origin);
    }
}

pub fn cast() -> Result<()> {
    let socket_address: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);
    let socket = UdpSocket::bind(socket_address)?;
    // TODO turn this back on when working from multiple PCs
    // socket.set_multicast_loop_v4(false)?;
    socket.connect(SocketAddrV4::new(MULTI_CAST_ADDR, 9778))?;
    let data = generate_fake_data();
    println!("\n[broadcasting at {} ]", socket.local_addr().unwrap());
    loop {
        socket.send(data.as_bytes())?;
        thread::sleep(time::Duration::from_secs(2));
    }
}

pub fn become_discoverable() {
    thread::spawn(|| {
        listen().unwrap();
    });
    cast().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cant_create_multiple_collection_handles_at_once() {
        let first_collection = PeerCollection::new().unwrap();

        assert!(PEER_COLLECTION_IN_USE.load(Ordering::SeqCst));

        assert!(PeerCollection::new().is_err());

        drop(first_collection);

        assert!(!PEER_COLLECTION_IN_USE.load(Ordering::SeqCst));

        let _another = PeerCollection::new().unwrap();
    }
}
