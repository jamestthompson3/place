use anyhow::Result;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{env, thread, time};
use thiserror::Error;

static MULTI_CAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 1);
// Make sure you can only have one PeerCollection at a time
static PEER_COLLECTION_IN_USE: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
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
    pub fn add_peer(&mut self, peer: String, origin: String) {
        if !self.found.contains_key(&peer) {
            self.found.insert(peer, origin);
        }
    }
    pub fn remove_peer(&mut self, peer: &str) {
        self.found.remove(peer);
    }
    pub fn inspect_entries(&self) {
        for (peer, origin) in &self.found {
            println!("{} is located {}", peer, origin);
        }
    }
}

impl Drop for PeerCollection {
    fn drop(&mut self) {
        PEER_COLLECTION_IN_USE.store(false, Ordering::SeqCst);
    }
}

fn cast() -> Result<()> {
    let socket_address: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);
    let socket = UdpSocket::bind(socket_address)?;
    // TODO turn this back on when working from multiple PCs
    socket.set_multicast_loop_v4(false)?;
    socket.connect(SocketAddrV4::new(MULTI_CAST_ADDR, 9778))?;
    let data = generate_fake_data();
    println!("[ broadcasting at {} ]", socket.local_addr().unwrap());
    loop {
        socket.send(data.as_bytes())?;
        thread::sleep(time::Duration::from_secs(2));
    }
}

// TODO keep chunks in order
fn file_listen() -> Result<()> {
    let socket_address: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 9779);
    let socket = UdpSocket::bind(socket_address)?;
    let mut recv_files: HashMap<String, Vec<u64>> = HashMap::new();
    println!("Listening for files");
    // set up file buffer
    loop {
        let mut buf = [0; 512];

        let (amt, origin) = socket.recv_from(&mut buf)?;
        let buf = &mut buf[..amt];
        let message = String::from_utf8(buf.to_vec()).unwrap();
        println!("{}, {}", message, origin);
    }
}

#[derive(Debug)]
pub struct PeerSharing {
    peers: PeerCollection,
}

// TODO
// * sweep for dropped peers.
// * clean up spawned processes
impl PeerSharing {
    pub fn new() -> Result<PeerSharing, Error> {
        let collection = PeerCollection::new().unwrap();
        Ok(PeerSharing { peers: collection })
    }

    fn multicast_listen(&mut self) -> Result<()> {
        let socket_address: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 9778);
        let bind_addr = Ipv4Addr::new(0, 0, 0, 0);
        let socket = UdpSocket::bind(socket_address)?;
        socket.join_multicast_v4(&MULTI_CAST_ADDR, &bind_addr)?;
        println!("Joined multicast group...");
        // set up message buffer
        loop {
            let mut buf = [0; 120];

            let (amt, origin) = socket.recv_from(&mut buf)?;
            let buf = &mut buf[..amt];
            let message = String::from_utf8(buf.to_vec()).unwrap();
            println!("{}, {}", message, origin);
            self.peers.add_peer(message, origin.to_string());
        }
    }

    pub fn make_discoverable(&mut self) {
        thread::spawn(|| {
            cast().unwrap();
        });
        thread::spawn(|| {
            file_listen().unwrap();
        });
        self.multicast_listen().unwrap();
    }

    pub fn list_peers(&self) {
        self.peers.inspect_entries();
    }
}

// The various error cases that may be encountered while using this library.
#[derive(Debug, Copy, Clone, PartialEq, Error)]
pub enum Error {
    #[error("PublicCollection already in use!")]
    AlreadyInUse,
}

fn generate_fake_data() -> String {
    let key = "USER";
    let val = env::var(key).unwrap();
    String::from(val)
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
