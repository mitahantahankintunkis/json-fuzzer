#![allow(unused)]

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
