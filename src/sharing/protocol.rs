use crate::filesystem::open_data_file;
use anyhow::Result;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use thiserror::Error;

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
}

fn match_as_byte_array(value: Option<&Vec<u8>>) -> Result<Vec<u8>, SerializerErrors> {
    match value {
        Some(byte_array) => return Ok(byte_array.to_vec()),
        None => return Err(SerializerErrors::InvalidBytes),
    }
}
fn match_as_u8(value: Option<&Vec<u8>>) -> Result<u8, SerializerErrors> {
    match value {
        Some(num) => {
            let num_slice = num.as_slice();
            return Ok(u8::from_be_bytes(num_slice.try_into().unwrap()));
        }
        None => return Err(SerializerErrors::InvalidBytes),
    };
}
fn match_as_u32(value: Option<&Vec<u8>>) -> Result<u32, SerializerErrors> {
    match value {
        Some(num) => {
            let num_slice = num.as_slice();
            return Ok(u32::from_be_bytes(num_slice.try_into().unwrap()));
        }
        None => return Err(SerializerErrors::InvalidBytes),
    };
}
fn match_as_u16(value: Option<&Vec<u8>>) -> Result<u16, SerializerErrors> {
    match value {
        Some(num) => {
            let num_slice = num.as_slice();
            return Ok(u16::from_be_bytes(num_slice.try_into().unwrap()));
        }
        None => return Err(SerializerErrors::InvalidBytes),
    };
}

pub fn deserialize_to_fun_header(ser_header: Vec<Vec<u8>>) -> Result<FunHeader, SerializerErrors> {
    let mut header = ser_header.iter();
    let name_len = match_as_u8(header.next())?;
    let name = match_as_byte_array(header.next())?;
    let conn_type = match_as_u8(header.next())?.try_into()?;
    let wnd_size = match_as_u32(header.next())?;
    let seq_nr = match_as_u16(header.next())?;
    let ack_nr = match_as_u16(header.next())?;
    return Ok(FunHeader {
        name_len,
        name,
        conn_type,
        wnd_size,
        seq_nr,
        ack_nr,
    });
}

type SerializedFunHeader = Vec<Vec<u8>>;
// Add another type alias for legibility
type SerializedFunPacket = Vec<Vec<u8>>;

// TODO Don't store everything in the heap
pub fn serialize_as_fun_header(
    name_len: u8,
    name: Vec<u8>,
    conn_type: FunType,
    wnd_size: u32,
    seq_nr: u16,
    ack_nr: u16,
) ->  SerializedFunHeader {
    let buffer = vec![
        name_len.to_be_bytes().to_vec(),
        name.to_vec(),
        conn_type.to_be_bytes().to_vec(),
        wnd_size.to_be_bytes().to_vec(),
        seq_nr.to_be_bytes().to_vec(),
        ack_nr.to_be_bytes().to_vec(),
    ];
    buffer
}

// Basically we need to do two things to send this over the network
// 1: flatten the header array
//      * This means changing the deserialization and serialization logic
// 2: create a function that returns a Vec of Vecs that are the split chunks
//    so we can do `for chunk in chunks ...` and send them packet by packet
pub fn chunk_file(file_handle: File) -> Result<<Vec<u8>>, ()> {
    let name_len = name.as_bytes().len();
    let mut contents = vec![];
    let mut file_contents = vec![];
    // First push the name_len
    contents.push(u8::try_from(name_len).unwrap());
    contents.extend_from_slice(name.as_bytes());
    file_handle.read_to_end(&mut file_contents).unwrap();
    contents.extend_from_slice(file_contents.as_slice());
    Ok(contents)
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
        let test_header = serialize_as_fun_header(
            name_len,
            name.as_bytes().to_vec(),
            conn_type,
            wnd_size,
            seq_nr,
            ack_nr,
        );

        // Make sure we load things as expected
        assert_eq!(test_header[3], u32::to_be_bytes(10));
        assert_eq!(
            String::from_utf8(test_header[1].to_vec()).unwrap(),
            "test.txt"
        );

        let deserialize_header = deserialize_to_fun_header(test_header).unwrap();

        let instantiated_header = FunHeader::new(
            name_len,
            name.as_bytes().to_vec(),
            conn_type,
            wnd_size,
            seq_nr,
            ack_nr,
        );

        assert_eq!(
            format!("{:?}", deserialize_header),
            format!("{:?}", instantiated_header)
        );
    }
}
