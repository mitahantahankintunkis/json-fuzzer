#![allow(unused)]
use std::time::{SystemTime, UNIX_EPOCH};

use regex::Regex;

pub fn byte_to_string(byte: u8) -> String {
    match byte {
        0x00..0x07 | 0x0e..0x20 | 0x7f.. => format!("\\x{:02x}", byte),
        0x07 => "\\a".to_string(),
        0x08 => "\\b".to_string(),
        0x09 => "\\t".to_string(),
        0x0a => "\\n".to_string(),
        0x0b => "\\v".to_string(),
        0x0c => "\\f".to_string(),
        0x0d => "\\r".to_string(),
        _ => (byte as char).to_string(),
    }
}

// String to UTF-8 bytes. Decodes '\xAA' notation
pub fn decode_str(s: &str) -> Vec<u8> {
    let re = Regex::new(r"\\x[0-9a-fA-F]{2}").unwrap();
    let mut prev = 0;
    let mut bytes = Vec::new();
    for mat in re.find_iter(&s) {
        bytes.extend_from_slice(s[prev..mat.start()].as_bytes());
        let byte = u8::from_str_radix(&s[(mat.start() + 2)..mat.end()], 16).unwrap();

        bytes.push(byte);
        prev = mat.end();
    }

    bytes.extend_from_slice(s[prev..].as_bytes());

    bytes
}

pub enum TimeType {
    Nanos,
    Micros,
    Millis,
    Secs,
    Mins,
}

pub fn ns_to_string(ns: f64) -> (String, TimeType) {
    match ns.abs() {
        0.0..1000.0 => (format!("{:.0}ns", ns), TimeType::Nanos),
        1000.0..1000_000.0 => (format!("{:.0}µs", ns / 1000.0), TimeType::Micros),
        1000_000.0..1000_000_000.0 => (format!("{:.0}ms", ns / 1000_000.0), TimeType::Millis),
        1000_000_000.0..60_000_000_000.0 => {
            (format!("{:.0}s", ns / 1000_000_000.0), TimeType::Secs)
        }
        _ => (format!("{:.0}m", ns / 60_000_000_000.0), TimeType::Mins),
    }
}

pub fn unixtime() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
