// use serde::Deserialize;
// use serde_json::{Result, Value};
use std::io::prelude::*;
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:5000")?;
    stream.set_nodelay(true)?;

    let mut name_buffer: [u8; 64] = [0; 64];
    let name = "rust-serde";
    name_buffer[0..name.len()].copy_from_slice(name.as_bytes());

    stream
        .write_all(&name_buffer)
        .expect("Rust: Could not write name");

    let mut read_buffer: Box<Vec<u8>> = Box::new(Vec::new());
    let mut write_buffer: Box<Vec<u8>> = Box::new(Vec::new());

    loop {
        let mut header: [u8; 8] = [0; 8];

        stream
            .read_exact(&mut header)
            .expect("Could not read buffer size");

        let buffer_size = u32::from_le_bytes(header[0..4].try_into().unwrap());
        let payload_size = u16::from_le_bytes(header[4..6].try_into().unwrap());
        let batch_size = u16::from_le_bytes(header[6..8].try_into().unwrap());

        if read_buffer.len() != usize::from(batch_size) * usize::from(payload_size) {
            read_buffer = Box::new(vec![0; usize::from(batch_size) * usize::from(payload_size)]);
        }

        if write_buffer.len() != (buffer_size as usize) {
            write_buffer = Box::new(vec![0; buffer_size.try_into().unwrap()]);
        }

        match stream.read_exact(&mut read_buffer) {
            Ok(()) => {
                // let size_buffer: &[u8] = &payload_size.to_le_bytes();
                let mut byte_offset: usize = 4;

                for batch in 0..usize::from(batch_size) {
                    let data: &[u8] = &read_buffer[(batch * usize::from(payload_size))
                        ..((batch + 1) * usize::from(payload_size))];

                    let message = match serde_json::from_slice::<serde_json::Value>(data) {
                        Ok(parsed) => match parsed.get("q") {
                            Some(q) => q.to_string(),
                            None => String::from("KEY_NOT_FOUND"),
                        },
                        Err(_) => String::from("PARSE_ERROR"),
                    };

                    let size_buffer = (message.len() as u16).to_le_bytes();
                    write_buffer[byte_offset..(byte_offset + 2)].copy_from_slice(&size_buffer);
                    byte_offset += 2;

                    write_buffer[byte_offset..(byte_offset + message.len())]
                        .copy_from_slice(message.as_bytes());
                    byte_offset += message.len();

                    // println!("{} {}", std::str::from_utf8(data).unwrap_or("err"), message,);

                    // println!("{:02x?}", data);
                    // write_buffer[byte_offset] = 0x1;
                    // byte_offset += 1;
                    // write_buffer[byte_offset] = 0x0;
                    // byte_offset += 1;
                    // write_buffer[byte_offset] = 0x1;
                    // byte_offset += 1;

                    // write_buffer[byte_offset..(byte_offset + 2)].copy_from_slice(&size_buffer);
                    // byte_offset += 2;
                    //
                    // write_buffer[byte_offset..(byte_offset + data.len())].copy_from_slice(&data);
                    // byte_offset += data.len();
                }

                write_buffer[0..4]
                    .copy_from_slice(&u32::try_from(byte_offset - 4).unwrap().to_le_bytes());

                stream
                    .write_all(&write_buffer[0..byte_offset])
                    .expect("Write error");
            }
            Err(_e) => {
                println!("Connection closed");
                return Ok(());
            }
        }
    }
}
