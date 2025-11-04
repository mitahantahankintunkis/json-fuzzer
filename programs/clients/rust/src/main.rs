#![allow(unused)]
use core::panic::PanicInfo;
use std::env;
use std::io::prelude::*;
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::process::exit;
use std::time::Instant;

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
    let mut stream: UnixStream;
    let args: Vec<String> = env::args().collect();

    let parser_number = if args.len() == 2 {
        args[1].parse::<usize>().unwrap()
    } else {
        0
    };

    let (name, parse_fn): (&str, fn(&[u8], &str, &Datatype) -> String) = match parser_number {
        0 => ("rust_serde", parse_serde),
        1 => ("rust_json", parse_json),
        _ => exit(1),
    };

    loop {
        if let Ok(s) = UnixStream::connect("/tmp/fuzzer.sock") {
            // if let Ok(s) = TcpStream::connect("127.0.0.1:5000") {
            stream = s;
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let mut name_buffer: [u8; 64] = [0; 64];
    name_buffer[0..name.len()].copy_from_slice(name.as_bytes());

    stream
        .write_all(&name_buffer)
        .expect("Rust: Could not write name");

    let mut read_buffer: Box<Vec<u8>> = Box::new(Vec::new());
    let mut write_buffer: Box<Vec<u8>> = Box::new(Vec::new());

    loop {
        let mut header = [0u8; 9];

        if stream.read_exact(&mut header).is_err() {
            return Ok(());
        }

        let input_buffer_size = u32::from_le_bytes(header[0..4].try_into().unwrap()) as usize;
        // let datatype = header[4];
        let key_len = u32::from_le_bytes(header[5..9].try_into().unwrap()) as usize;

        if read_buffer.len() <= std::cmp::max(input_buffer_size, key_len) {
            read_buffer = Box::new(vec![0; std::cmp::max(input_buffer_size, key_len)]);
        }

        if write_buffer.len() <= input_buffer_size << 2 {
            write_buffer = Box::new(vec![0; input_buffer_size << 2]);
        }

        stream.read_exact(&mut read_buffer[0..key_len]).unwrap();
        let key = String::from_utf8(read_buffer[0..key_len].to_vec())
            .expect("Rust client: Key should be valid UTF-8");

        match stream.read_exact(&mut read_buffer[0..input_buffer_size]) {
            Ok(()) => {
                let mut read_offset = 0;
                let mut write_offset: usize = 4;

                while read_offset < input_buffer_size {
                    let json_size = u16::from_le_bytes(
                        read_buffer[read_offset..read_offset + 2]
                            .try_into()
                            .unwrap(),
                    ) as usize;
                    read_offset += 2;

                    let data: &[u8] = &read_buffer[read_offset..read_offset + json_size];
                    read_offset += json_size;

                    let start = Instant::now();
                    let message = parse_fn(data, &key, &Datatype::Int);
                    let micros = start.elapsed().as_micros() as u32;

                    write_buffer[write_offset..write_offset + 4]
                        .copy_from_slice(&micros.to_le_bytes());
                    write_offset += 4;

                    write_buffer[write_offset..write_offset + 2]
                        .copy_from_slice(&(message.len() as u16).to_le_bytes());
                    write_offset += 2;

                    write_buffer[write_offset..write_offset + message.len()]
                        .copy_from_slice(message.as_bytes());
                    write_offset += message.len();
                }

                write_buffer[0..4]
                    .copy_from_slice(&u32::try_from(write_offset - 4).unwrap().to_le_bytes());

                stream
                    .write_all(&write_buffer[0..write_offset])
                    .expect("Write error");
            }
            Err(_e) => {
                return Ok(());
            }
        }
    }
}
