use crate::filesystem::open_data_file;
use anyhow::Result;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fs;
use std::io::prelude::*;
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
    pub fn new() -> Result<PeerCollection, CollectionError> {
        if PEER_COLLECTION_IN_USE.compare_and_swap(false, true, Ordering::SeqCst) == false {
            Ok(PeerCollection {
                found: HashMap::new(),
            })
        } else {
            Err(CollectionError::AlreadyInUse)
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
    pub fn new() -> Result<PeerSharing, CollectionError> {
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
pub enum CollectionError {
    #[error("PublicCollection already in use!")]
    AlreadyInUse,
}

fn generate_fake_data() -> String {
    let key = "USER";
    let val = env::var(key).unwrap();
    String::from(val)
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum FunType {
    OPEN = 0,
    FINISHED = 1,
    ERR = 2,
}

#[derive(Debug, Copy, Clone, PartialEq, Error)]
pub enum SerializerErrors {
    #[error("Invalid bytes found while parsing")]
    InvalidBytes,
}

trait Byteable {
    fn to_be_bytes(&self) -> [u8; 1];
}

impl Byteable for FunType {
    fn to_be_bytes(&self) -> [u8; 1] {
        [*self as u8]
    }
}

impl TryFrom<u8> for FunType {
    type Error = SerializerErrors;
    fn try_from(v: u8) -> Result<Self, SerializerErrors> {
        match v {
            x if x == FunType::OPEN as u8 => Ok(FunType::OPEN),
            x if x == FunType::FINISHED as u8 => Ok(FunType::FINISHED),
            x if x == FunType::ERR as u8 => Ok(FunType::ERR),
            _ => Err(SerializerErrors::InvalidBytes),
        }
    }
}

#[derive(Debug)]
pub struct FunHeader {
    name_len: u8,
    name: Vec<u8>,
    conn_type: FunType,
    wnd_size: u32,
    seq_nr: u16,
    ack_nr: u16,
}

impl FunHeader {
    pub fn serialize(&self) -> Vec<Vec<u8>> {
        let buffer = vec![
            self.name_len.to_be_bytes().to_vec(),
            self.name.to_vec(),
            self.conn_type.to_be_bytes().to_vec(),
            self.wnd_size.to_be_bytes().to_vec(),
            self.seq_nr.to_be_bytes().to_vec(),
            self.ack_nr.to_be_bytes().to_vec(),
        ];
        buffer
    }
    pub fn new(
        name_len: u8,
        name: Vec<u8>,
        conn_type: FunType,
        wnd_size: u32,
        seq_nr: u16,
        ack_nr: u16,
    ) -> FunHeader {
        FunHeader {
            name_len,
            name,
            conn_type,
            wnd_size,
            seq_nr,
            ack_nr,
        }
    }
    pub fn deserialize(&self, ser_header: Vec<Vec<u8>>) -> Result<FunHeader, SerializerErrors> {
        let mut header = ser_header.iter();
        let name_len = self.match_as_u8(header.next())?;
        let name = self.match_as_byte_array(header.next())?;
        let conn_type = self.match_as_u8(header.next())?.try_into()?;
        let wnd_size = self.match_as_u32(header.next())?;
        let seq_nr = self.match_as_u16(header.next())?;
        let ack_nr = self.match_as_u16(header.next())?;
        return Ok(FunHeader {
            name_len,
            name,
            conn_type,
            wnd_size,
            seq_nr,
            ack_nr,
        });
    }
    fn match_as_byte_array(&self, value: Option<&Vec<u8>>) -> Result<Vec<u8>, SerializerErrors> {
        match value {
            Some(byte_array) => return Ok(byte_array.to_vec()),
            None => return Err(SerializerErrors::InvalidBytes),
        }
    }
    fn match_as_u8(&self, value: Option<&Vec<u8>>) -> Result<u8, SerializerErrors> {
        match value {
            Some(num) => {
                let num_slice = num.as_slice();
                return Ok(u8::from_be_bytes(num_slice.try_into().unwrap()));
            }
            None => return Err(SerializerErrors::InvalidBytes),
        };
    }
    fn match_as_u32(&self, value: Option<&Vec<u8>>) -> Result<u32, SerializerErrors> {
        match value {
            Some(num) => {
                let num_slice = num.as_slice();
                return Ok(u32::from_be_bytes(num_slice.try_into().unwrap()));
            }
            None => return Err(SerializerErrors::InvalidBytes),
        };
    }
    fn match_as_u16(&self, value: Option<&Vec<u8>>) -> Result<u16, SerializerErrors> {
        match value {
            Some(num) => {
                let num_slice = num.as_slice();
                return Ok(u16::from_be_bytes(num_slice.try_into().unwrap()));
            }
            None => return Err(SerializerErrors::InvalidBytes),
        };
    }

    // TODO Don't store everything in the heap
    // pub fn serialize(&self) -> u32 {
    //     let buffer = vec![
    //         self.name_len.to_be_bytes(),
    //         self.name.to_be_bytes(),
    //         self.conn_type.to_be_bytes(),
    //         self.wnd_size.to_be_bytes(),
    //         self.seq_nr.to_be_bytes(),
    //         self.ack_nr.to_be_bytes(),
    //     ];
    // }
}

pub fn encode() -> Box<Vec<u8>> {
    let mut file_handle = open_data_file("test.txt").unwrap();
    let name = String::from("test.txt");
    let name_len = name.as_bytes().len();
    let mut contents = vec![];
    let mut file_contents = vec![];
    contents.push(u8::try_from(name_len).unwrap());
    contents.extend_from_slice(name.as_bytes());
    file_handle.read_to_end(&mut file_contents).unwrap();
    contents.extend_from_slice(file_contents.as_slice());
    Box::new(contents)
}

pub fn decode(buffer: Box<Vec<u8>>) {
    let name_len = buffer[0] as usize;
    let name_bytes = &buffer[1..=name_len];
    let name_string = String::from_utf8(name_bytes.to_vec()).unwrap();
    let file = &buffer[name_len + 1..];
    fs::write(name_string, file).unwrap();
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
    #[test]
    fn can_serialize_and_serialize_fun_headers() {
        let mut file_handle = open_data_file("test.txt").unwrap();
        let name = String::from("test.txt");
        let name_len = u8::try_from(name.as_bytes().len()).unwrap();
        let conn_type = FunType::OPEN;
        let mut file_contents = vec![];
        file_handle.read_to_end(&mut file_contents).unwrap();
        let wnd_size = u32::try_from(file_contents.len()).unwrap();
        let seq_nr = 1;
        let ack_nr = 1;
        let test_header = FunHeader::new(
            name_len,
            name.as_bytes().to_vec(),
            conn_type,
            wnd_size,
            seq_nr,
            ack_nr,
        );

        // Make sure we load things as expected
        assert_eq!(test_header.wnd_size, 10);
        assert_eq!(
            String::from_utf8(test_header.name.to_vec()).unwrap(),
            "test.txt"
        );

        let serialized_header = test_header.serialize();
        let deserialize_header = test_header.deserialize(serialized_header).unwrap();
        assert_eq!(
            format!("{:?}", deserialize_header),
            format!("{:?}", test_header)
        );
    }
}
