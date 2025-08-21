#![allow(unused)]
use core::panic::PanicInfo;
// use serde::Deserialize;
// use serde_json::{Result, Value};
use std::env;
use std::io::prelude::*;
use std::net::TcpStream;
use std::process::exit;

#[macro_use]
extern crate json;

const KEY_NOT_FOUND: &str = "KEY_NOT_FOUND";
const PARSE_ERROR: &str = "PARSE_ERROR";
const CRITICAL_ERROR: &str = "CRITICAL_ERROR";

enum Datatype {
    Int,
    Float,
    String,
    Object,
    Array,
    Null,
    Bool,
}

fn parse_serde(data: &[u8], key: &str, _datatype: &Datatype) -> String {
    match serde_json::from_slice::<serde_json::Value>(data) {
        Ok(parsed) => match parsed.get(key) {
            Some(q) => q.to_string(),
            None => String::from(KEY_NOT_FOUND),
        },
        Err(_) => String::from(PARSE_ERROR),
    }
}

fn parse_json(data: &[u8], key: &str, _datatype: &Datatype) -> String {
    let parsed = json::parse(&String::from_utf8_lossy(&data).to_string());

    match parsed {
        Ok(parsed) => parsed[key].to_string(),
        Err(_) => String::from(PARSE_ERROR),
    }
}

fn main() -> std::io::Result<()> {
    let mut stream: TcpStream;
    let args: Vec<String> = env::args().collect();

    let parser_number = if args.len() == 2 {
        args[1].parse::<usize>().unwrap()
    } else {
        0
    };

    let name = match parser_number {
        0 => "rust_serde",
        1 => "rust_json",
        _ => exit(1),
    };

    loop {
        if let Ok(s) = TcpStream::connect("127.0.0.1:5000") {
            stream = s;
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    stream.set_nodelay(true)?;

    let mut name_buffer: [u8; 64] = [0; 64];
    name_buffer[0..name.len()].copy_from_slice(name.as_bytes());

    stream
        .write_all(&name_buffer)
        .expect("Rust: Could not write name");

    let mut read_buffer: Box<Vec<u8>> = Box::new(Vec::new());
    let mut write_buffer: Box<Vec<u8>> = Box::new(Vec::new());

    loop {
        let mut header: [u8; 8] = [0; 8];

        if stream.read_exact(&mut header).is_err() {
            return Ok(());
        }

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
                let mut byte_offset: usize = 4;

                for batch in 0..usize::from(batch_size) {
                    let data: &[u8] = &read_buffer[(batch * usize::from(payload_size))
                        ..((batch + 1) * usize::from(payload_size))];

                    let message = match parser_number {
                        0 => parse_serde(data, "q", &Datatype::Int),
                        1 => parse_json(data, "q", &Datatype::Int),
                        _ => exit(1),
                    };

                    let size_buffer = (message.len() as u16).to_le_bytes();
                    write_buffer[byte_offset..(byte_offset + 2)].copy_from_slice(&size_buffer);
                    byte_offset += 2;

                    write_buffer[byte_offset..(byte_offset + message.len())]
                        .copy_from_slice(message.as_bytes());
                    byte_offset += message.len();
                }

                write_buffer[0..4]
                    .copy_from_slice(&u32::try_from(byte_offset - 4).unwrap().to_le_bytes());

                stream
                    .write_all(&write_buffer[0..byte_offset])
                    .expect("Write error");
            }
            Err(_e) => {
                return Ok(());
            }
        }
    }
}
