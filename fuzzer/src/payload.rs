use serde::Deserialize;
use std::{collections::HashMap, fs};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub payloads: Vec<PayloadConfig>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PayloadConfig {
    pub name: String,
    pub payload: String,
    pub datatype: Datatype,
    pub key: Option<String>,
    #[serde(default)]
    pub fuzz: Vec<FuzzConfig>,
}

#[derive(Deserialize, Clone, Debug)]
pub enum Datatype {
    Int,
    Float,
    String,
    Object,
    Array,
    Null,
    Bool,
}

// Serde functions for defaults
fn one() -> usize {
    1
}

#[derive(Deserialize, Clone, Debug)]
pub struct FuzzConfig {
    // replace_characters: Option<String>,
    #[serde(default = "one")]
    pub bytes: usize,
    #[serde(default)]
    pub prefix: String,
    #[serde(default)]
    pub fuzz_mode: FuzzModeConfig,
    pub fuzz_range: Option<FuzzRangeConfig>,
    // pub min: Option<u32>,
    // pub max: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Fuzz {
    pub config: FuzzConfig,
    pub mode: FuzzMode,
    // i: usize,
    pub value: u32,
    pub range: FuzzRange,
    pub prefix: Vec<u8>,
    // pub characterset: Vec<u8>,
    // pub min: u32,
    // pub max: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub enum FuzzRangeConfig {
    Number { bytes: usize },
    MinMax { min: u32, max: u32 },
    Characters { chars: String, length: usize },
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone)]
pub enum FuzzRange {
    MinMax { min: u32, max: u32 },
    Characters { chars: Vec<char>, length: usize },
    Bytes(Vec<u8>),
}

impl From<&FuzzRangeConfig> for FuzzRange {
    fn from(config: &FuzzRangeConfig) -> Self {
        match config {
            FuzzRangeConfig::Number { bytes } => FuzzRange::MinMax {
                min: 0,
                max: ((u64::MAX & (1u64 << (bytes * 8))) - 1) as u32,
            },
            FuzzRangeConfig::MinMax { min, max } => FuzzRange::MinMax {
                min: *min,
                max: *max,
            },
            FuzzRangeConfig::Characters { chars, length } => FuzzRange::Characters {
                chars: chars.chars().collect(),
                length: *length,
            },
            FuzzRangeConfig::Bytes(bytes) => FuzzRange::Bytes(bytes.clone()),
        }
    }
}

impl FuzzRange {
    pub fn map(&self, i: u32, byte_offset: usize) -> Option<u8> {
        match self {
            FuzzRange::MinMax { min, max } => {
                if i + min > *max {
                    None
                } else {
                    let bytes = i + min;
                    Some((bytes >> (byte_offset * 8) & 0xff) as u8)
                }
            }
            FuzzRange::Characters { chars, length } => {
                if i >= chars.len().pow(*length as u32) as u32 {
                    None
                } else {
                    let len = chars.len();

                    let char =
                        chars[((i as usize) / (len.pow(byte_offset as u32)) as usize) % len] as u32;

                    // Some((char as u32 >> (byte_offset * 8) & 0xff) as u8)
                    Some((char & 0xff) as u8)
                }
            }
            FuzzRange::Bytes(bytes) => {
                if i >= bytes.len() as u32 {
                    None
                } else {
                    Some((bytes[i as usize] >> (byte_offset * 8) & 0xff) as u8)
                }
            }
        }
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
pub enum FuzzModeConfig {
    #[default]
    InsertAllPossible,
    ReplaceAllPossible,
    ReplaceCharacters(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum FuzzMode {
    Insert(usize),
    Replace(usize),
    ReplaceLocked(Vec<usize>),
    // LinkedReplace(usize),
}

#[derive(Debug)]
pub struct Payload {
    pub bytes: Vec<u8>,
    pub original_bytes: Vec<u8>,
    pub config: PayloadConfig,
    pub fuzz: Vec<Fuzz>,
    pub fuzz_insert_i: Vec<usize>,
    finished: bool,
}

impl Payload {
    // pub fn fuzzed_length(&self) -> usize {
    //     let mut length = self.bytes.len();
    //
    //     for fuzz in &self.fuzz {
    //         if let FuzzMode::Insert(_) = fuzz.mode {
    //             length += fuzz.config.bytes;
    //         }
    //     }
    //
    //     length
    // }

    pub fn advance(&mut self) -> Result<(), ()> {
        if self.finished {
            return Err(());
        }

        // Increase fuzz bytes
        for fuzz in self.fuzz.iter_mut() {
            if let Some(_) = fuzz.range.map(fuzz.value + 1, 0) {
                fuzz.value += 1;
            } else {
                fuzz.value = 0;
            }

            match &fuzz.mode {
                FuzzMode::Replace(i) => {
                    for (j, byte) in fuzz.prefix.iter().enumerate() {
                        self.bytes[i + j] = *byte;
                    }

                    for byte_offset in 0..fuzz.config.bytes {
                        let byte = fuzz.range.map(fuzz.value, byte_offset).unwrap();
                        // let value = (mapped >> (byte_offset * 8) & 0xff) as u8;
                        // println!("replace {} {}", i, byte_to_string(value));
                        self.bytes[i + (fuzz.config.bytes - byte_offset - 1) + fuzz.prefix.len()] =
                            byte;
                    }
                }
                FuzzMode::ReplaceLocked(indices) => {
                    for i in indices {
                        for (j, byte) in fuzz.prefix.iter().enumerate() {
                            self.bytes[i + j] = *byte;
                        }

                        for byte_offset in 0..fuzz.config.bytes {
                            let byte = fuzz.range.map(fuzz.value, byte_offset).unwrap();
                            // let value = (mapped >> (byte_offset * 8) & 0xff) as u8;
                            // println!("locked  {} {}", i, byte_to_string(value));
                            let byte_i =
                                i + (fuzz.config.bytes - byte_offset - 1) + fuzz.prefix.len();
                            self.bytes[byte_i] = byte;

                            // TODO - hack
                            self.original_bytes[byte_i] = byte;
                        }
                    }
                }
                // FuzzMode::Insert(i) => {
                //     println!("insert  {} {}", i, byte_to_string(fuzz.value as u8));
                // }
                _ => {}
            }

            // if let FuzzMode::ReplaceLocked(indices) = &fuzz.mode {}

            if fuzz.value != 0 {
                // println!("{} {:?}", fuzz.value, fuzz.range.map(fuzz.value + 1, 0));
                return Ok(());
            }
        }

        // Overflow - each fuzz byte is at their minimum value
        // Move fuzz bytes to right
        // FuzzMode::Insert

        let mut offset = 0;

        for fuzz_i in 0..self.fuzz.len() {
            match self.fuzz[fuzz_i].mode {
                FuzzMode::Insert(mut fuzz_index) => {
                    if fuzz_index < self.bytes.len() {
                        fuzz_index += 1;
                        self.fuzz[fuzz_i].mode = FuzzMode::Insert(fuzz_index);

                        for k in 0..fuzz_i {
                            if let FuzzMode::Insert(_) = self.fuzz[k].mode {
                                self.fuzz[k].mode = FuzzMode::Insert(fuzz_index);
                            }
                        }

                        return Ok(());
                    }
                }
                FuzzMode::Replace(fuzz_index0) => {
                    offset += self.fuzz[fuzz_i].config.bytes;
                    offset += self.fuzz[fuzz_i].prefix.len();

                    let new_fuzz_index0 = fuzz_index0 + 1;

                    if new_fuzz_index0 <= self.bytes.len() - offset {
                        self.fuzz[fuzz_i].mode = FuzzMode::Replace(new_fuzz_index0);

                        // for i in 0..self.bytes.len() {
                        //     self.bytes[i] = self.original_bytes[i];
                        // }

                        for (j, byte) in self.fuzz[fuzz_i].prefix.iter().enumerate() {
                            self.bytes[new_fuzz_index0 + j] = *byte;
                        }

                        self.bytes[fuzz_index0] = self.original_bytes[fuzz_index0];

                        for byte_offset in 0..self.fuzz[fuzz_i].config.bytes {
                            let byte = self.fuzz[fuzz_i]
                                .range
                                .map(self.fuzz[fuzz_i].value, byte_offset)
                                .unwrap();

                            // let value = (mapped >> (byte_offset * 8) & 0xff) as u8;
                            // println!("  new {} {}", fuzz_index0 + 1, byte_to_string(value));
                            self.bytes[new_fuzz_index0
                                + (self.fuzz[fuzz_i].config.bytes - byte_offset - 1)
                                + self.fuzz[fuzz_i].prefix.len()] = byte;
                        }

                        for fuzz_j in 0..fuzz_i {
                            if let FuzzMode::Replace(fuzz_index1) = self.fuzz[fuzz_j].mode {
                                let new_fuzz_index1 = new_fuzz_index0 + (fuzz_i - fuzz_j);
                                self.fuzz[fuzz_j].mode = FuzzMode::Replace(new_fuzz_index1);

                                self.bytes[fuzz_index1] = self.original_bytes[fuzz_index1];

                                for (j, byte) in self.fuzz[fuzz_j].prefix.iter().enumerate() {
                                    self.bytes[new_fuzz_index1 + j] = *byte;
                                }

                                for byte_offset in 0..self.fuzz[fuzz_j].config.bytes {
                                    let byte = self.fuzz[fuzz_j]
                                        .range
                                        .map(self.fuzz[fuzz_j].value, byte_offset)
                                        .unwrap();

                                    // let byte = (mapped >> (byte_offset * 8) & 0xff) as u8;
                                    self.bytes[new_fuzz_index1
                                        + (self.fuzz[fuzz_j].config.bytes - byte_offset - 1)
                                        + self.fuzz[fuzz_i].prefix.len()] = byte;
                                }
                            }
                        }

                        // Restart inserts
                        for k in 0..self.fuzz.len() {
                            if let FuzzMode::Insert(_) = self.fuzz[k].mode {
                                self.fuzz[k].mode = FuzzMode::Insert(0);
                            }
                        }

                        return Ok(());
                    }
                }
                // _ => return Err(()),
                _ => {}
            }
        }

        self.finished = true;
        Err(())
    }
}

impl From<PayloadConfig> for Payload {
    fn from(config: PayloadConfig) -> Self {
        // let mut fuzzed_payload: Vec<u8> = config.payload.bytes().collect();
        let mut fuzzed_payload: Vec<u8> = config.payload.as_bytes().to_vec();
        let original_payload: Vec<u8> = fuzzed_payload.clone();
        // let mut fuzzed_payload: Vec<u8> = Vec::with_capacity(config.payload.len());
        // let mut original_payload: Vec<u8> = Vec::with_capacity(config.payload.len());
        let mut fuzz_lookup: HashMap<char, usize> = HashMap::new();

        // Add non locked fuzz bytes
        // let mut replace_index = 0;
        // let mut num_of_replaces = 0;

        let mut offset = 0;
        let mut inserts: Vec<Fuzz> = Vec::new();
        let mut replaces: Vec<Fuzz> = Vec::new();

        for config in &config.fuzz {
            match config.fuzz_mode {
                FuzzModeConfig::InsertAllPossible => {
                    inserts.push(config.clone().into());
                }
                FuzzModeConfig::ReplaceAllPossible => {
                    let mut fuzz: Fuzz = config.clone().into();
                    fuzz.mode = FuzzMode::Replace(offset);

                    for byte in &fuzz.prefix {
                        fuzzed_payload[offset] = *byte;
                        offset += 1;
                    }

                    for byte_offset in 0..config.bytes {
                        let byte = fuzz.range.map(0, byte_offset).unwrap_or(0u8);
                        // let byte = (value >> (byte_offset * 8) & 0xff) as u8;
                        fuzzed_payload[offset] = byte;
                        offset += 1;
                    }

                    replaces.push(fuzz);
                    // replace_index += 1;
                }
                _ => {}
            }
        }

        replaces.reverse();

        let mut character_replaces: Vec<Fuzz> = Vec::new();

        // Add replaced characters
        for (i, char) in config.payload.chars().enumerate() {
            let fuzz_config = config.fuzz.iter().find(|&f| {
                match &f.fuzz_mode {
                    FuzzModeConfig::ReplaceCharacters(replace) => replace.contains(char),
                    _ => false,
                }
                // f.replace_characters
                //     .as_ref()
                //     .is_some_and(|r| r.contains(char))
            });

            match fuzz_config {
                Some(fuzz_config) => {
                    let mut indices = vec![i];
                    let mut fuzz: Fuzz = fuzz_config.clone().into();
                    let value = fuzz.value;
                    let bytes = fuzz_config.bytes;

                    if let Some(&fuzz_i) = fuzz_lookup.get(&char) {
                        if let FuzzMode::ReplaceLocked(old_indices) =
                            &character_replaces[fuzz_i].mode
                        {
                            indices.extend(old_indices.iter());
                            character_replaces[fuzz_i].mode =
                                FuzzMode::ReplaceLocked(indices.clone());
                        }

                        // fuzz.mode = FuzzMode::LinkedReplace(fuzz_i)
                    } else {
                        // fuzz.mode = FuzzMode::Replace;
                        fuzz.mode = FuzzMode::ReplaceLocked(indices.clone());

                        if let FuzzModeConfig::ReplaceCharacters(replace) = &fuzz_config.fuzz_mode {
                            for char in replace.chars() {
                                fuzz_lookup.insert(char, character_replaces.len());
                            }
                        }

                        character_replaces.push(fuzz);
                    }

                    for i in indices {
                        let prefix = &character_replaces.last().unwrap().prefix;

                        for (j, byte) in prefix.iter().enumerate() {
                            fuzzed_payload[i + j] = *byte;
                        }

                        for byte_offset in 0..bytes {
                            let byte = &character_replaces
                                .last()
                                .unwrap()
                                .range
                                .map(value, byte_offset)
                                .unwrap_or(0u8);

                            if i + byte_offset >= fuzzed_payload.len() {
                                panic!("ReplaceCharacters overflow");
                            }

                            fuzzed_payload[i + byte_offset + prefix.len()] = *byte;
                        }
                    }

                    // original_payload.append(&mut char.to_string().as_bytes().to_vec());
                }
                None => {
                    // let mut bytes = String::from(char).as_bytes().to_vec();
                    // fuzzed_payload.append(&mut bytes.clone());
                    // original_payload.append(&mut bytes);
                }
            }
        }
        // println!("{:?} {:?}", original_payload, fuzzed_payload);
        let fuzzes = [inserts, replaces, character_replaces].concat();
        // println!("{:?} {:?}", original_payload, fuzzed_payload);

        Payload {
            bytes: fuzzed_payload,
            original_bytes: original_payload,
            config,
            fuzz_insert_i: fuzzes
                .iter()
                .zip(0..fuzzes.len())
                .filter(|(f, _)| matches!(f.mode, FuzzMode::Insert(_)))
                .map(|(_, i)| i)
                .collect(),
            fuzz: fuzzes,
            finished: false,
        }
    }
}

impl From<FuzzConfig> for Fuzz {
    fn from(config: FuzzConfig) -> Self {
        // let min = config.min.unwrap_or(0);
        // let max = config
        //     .max
        //     .unwrap_or(((u64::MAX & (1 << (config.bytes * 8))) - 1) as u32);

        if config.bytes > 4 {
            panic!(
                "Too many bytes in fuzz config. Received {}, maximum is 4",
                config.bytes
            );
        }

        let mode = match config.fuzz_mode {
            FuzzModeConfig::InsertAllPossible => FuzzMode::Insert(0),
            FuzzModeConfig::ReplaceAllPossible => FuzzMode::Replace(0),
            FuzzModeConfig::ReplaceCharacters(_) => FuzzMode::ReplaceLocked(vec![]),
        };

        let range = match &config.fuzz_range {
            Some(range) => range.into(),
            None => FuzzRange::MinMax {
                min: 0,
                max: ((u64::MAX & (1 << (config.bytes * 8))) - 1) as u32,
            },
        };

        Fuzz {
            range,
            mode,
            value: 0,
            prefix: config.prefix.bytes().collect(),
            config,
        }
    }
}

pub struct PayloadIter<'a> {
    pub payload: &'a Payload,
    pub i: usize,
    pub fuzz_i: usize,
    pub fuzz_byte_offset: usize,
}

// Iterates payload bytes.
// Either replaces or inserts fuzzed bytes to the original payload
impl<'a> Iterator for PayloadIter<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.fuzz_i > 0 {
            let i = self.payload.fuzz_insert_i[self.fuzz_i - 1];
            let fuzz = &self.payload.fuzz[i];

            if let FuzzMode::Insert(j) = fuzz.mode {
                // println!("{} {}", self.i, j);

                if self.i == j {
                    let offset_target = fuzz.config.bytes - 1 + fuzz.prefix.len();
                    let byte_offset = offset_target - self.fuzz_byte_offset;

                    let next = if self.fuzz_byte_offset < fuzz.prefix.len() {
                        fuzz.prefix[self.fuzz_byte_offset]
                    } else {
                        let value = fuzz.range.map(fuzz.value, byte_offset).unwrap();
                        ((value >> (byte_offset * 8)) & 0xff) as u8
                    };

                    // let offset = self.fuzz_byte_offset;
                    self.fuzz_byte_offset += 1;

                    if self.fuzz_byte_offset > offset_target {
                        self.fuzz_i -= 1;
                        self.fuzz_byte_offset = 0;
                    }

                    return Some(next);
                }
            }
        }

        if self.i >= self.payload.bytes.len() {
            return None;
        }

        self.i += 1;
        Some(self.payload.bytes[self.i - 1])
    }
}

impl<'a> IntoIterator for &'a Payload {
    type Item = u8;
    type IntoIter = PayloadIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PayloadIter {
            payload: self,
            i: 0,
            fuzz_i: self.fuzz_insert_i.len(),
            fuzz_byte_offset: 0,
        }
    }
}

pub fn load_payloads() -> Config {
    let payloads_string =
        fs::read_to_string("../payloads.toml").expect("Could not read payloads.toml");
    toml::from_str(&payloads_string).expect("Error while parsing payloads.toml")
}

#[cfg(test)]
mod tests {
    use crate::util::byte_to_string;

    use super::*;

    fn buffer_as_string(name: &str, buffer: &[u8]) -> String {
        format!(
            "{:12} {}",
            name,
            buffer
                .into_iter()
                .map(|c| byte_to_string(*c))
                .collect::<Vec<String>>()
                .join("")
        )
    }

    fn print_buffer(name: &str, buffer: &[u8]) {
        println!("{}", buffer_as_string(name, buffer));
    }

    #[test]
    fn creation() {
        let _config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "empty"
                payload = 'null'
                datatype = "Null"
                key = "q"

                [[payloads]]
                name = "duplicate_keys_single_byte"
                payload = '{"q":2,"q":3}'
                datatype = "Int"
                key = "q"
                [[payloads.fuzz]]

                [[payloads]]
                name = "duplicate_keys_two_bytes"
                payload = '{"q":2,"q":3}'
                datatype = "Int"
                key = "q"
                [[payloads.fuzz]]
                [[payloads.fuzz]]

                [[payloads]]
                name = "all_fuzz_modes_single_byte"
                payload = '{"q":"a"}'
                datatype = "String"
                key = "q"
                [[payloads.fuzz]]
                fuzz_mode = "ReplaceAllPossible"
                [[payloads.fuzz]]
                fuzz_mode = "InsertAllPossible"
                [[payloads.fuzz]]
                fuzz_mode = { ReplaceCharacters = "@" }

                [[payloads]]
                name = "large_float"
                payload = '1.79e308'
                datatype = "Float"
                [[payloads.fuzz]]
                fuzz_mode = "ReplaceAllPossible"
                # Digits
                fuzz_range = { Characters = { chars = '0123456789', length = 1 } }
                [[payloads.fuzz]]
                bytes = 2
        "#,
        )
        .unwrap();
    }

    #[test]
    fn advance_single_locked() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                payload = '{"@":2,"q":3}'
                datatype = "Int"
                key = "q"
                [[payloads.fuzz]]
                fuzz_mode = { ReplaceCharacters = "@" }
                # replace_characters = "@"
        "#,
        )
        .unwrap();

        let mut payload: Payload = config.payloads[0].clone().into();
        let mut bytes: Vec<u8> = payload.config.payload.bytes().collect();
        let mut byte = 0u8;

        loop {
            bytes[2] = byte;

            // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
            // print_buffer("Expected", &bytes);

            for (i, byte) in payload.into_iter().enumerate() {
                assert_eq!(byte, bytes[i]);
            }

            if let Err(_) = payload.advance() {
                break;
            }

            byte += 1;
        }
    }

    #[test]
    fn insert_one_byte() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                payload = '{"q":2,"q":3}'
                datatype = "Int"
                key = "q"
                [[payloads.fuzz]]
                bytes = 1
        "#,
        )
        .unwrap();
        let mut buffer: Vec<u8> =
            [vec![0], config.payloads[0].payload.as_bytes().to_vec()].concat();
        let mut payload: Payload = config.payloads[0].clone().into();

        println!("{:?}", config);
        println!("{:?}", payload);

        for i in 0..buffer.len() {
            buffer[i] = 0;

            for _ in 0..=255 {
                // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
                // print_buffer("Expected", &buffer);

                assert_eq!(buffer, payload.into_iter().collect::<Vec<u8>>());

                buffer[i] = buffer[i].wrapping_add(1);
                let _ = payload.advance();
            }

            if i < buffer.len() - 1 {
                buffer[i] = buffer[i + 1];
            }
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
        );
    }

    #[test]
    fn insert_two_bytes() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                payload = '{"q":"a"}'
                datatype = "String"
                key = "q"
                [[payloads.fuzz]]
                [[payloads.fuzz]]
        "#,
        )
        .unwrap();

        let original_buffer: Vec<u8> = config.payloads[0].payload.as_bytes().to_vec();
        let mut payload: Payload = config.payloads[0].clone().into();

        println!("{:?}", payload);

        let mut byte0_i = 0;
        let mut byte1_i = 0;

        loop {
            for b0 in 0..=255 {
                for b1 in 0..=255 {
                    // println!("{} {}", byte0_i, byte1_i);
                    let expected = [
                        original_buffer[0..byte0_i].to_vec(),
                        vec![b0],
                        original_buffer[byte0_i..byte1_i].to_vec(),
                        vec![b1],
                        original_buffer[byte1_i..].to_vec(),
                    ]
                    .concat();

                    // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
                    // print_buffer("Expected", &expected);

                    assert_eq!(
                        expected,
                        payload.into_iter().collect::<Vec<u8>>(),
                        "{}\n{}",
                        buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
                        buffer_as_string("Expected", &expected),
                    );

                    // assert!(payload.advance().is_err(), "Should not have next",);

                    let _ = payload.advance();
                }
            }

            // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
            // print_buffer("Expected", &expected);

            if byte1_i >= original_buffer.len() {
                if byte0_i >= original_buffer.len() {
                    break;
                }
                byte0_i += 1;
                byte1_i = byte0_i;
            } else {
                byte1_i += 1;
            }
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
        );
    }

    #[test]
    fn replace_two_bytes() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                payload = '{"q":"a"}'
                datatype = "String"
                key = "q"
                [[payloads.fuzz]]
                fuzz_mode = "ReplaceAllPossible"
                [[payloads.fuzz]]
                fuzz_mode = "ReplaceAllPossible"
        "#,
        )
        .unwrap();

        let original_buffer: Vec<u8> = config.payloads[0].payload.as_bytes().to_vec();
        let mut payload: Payload = config.payloads[0].clone().into();

        println!("{:?}", payload);

        let mut byte0_i = 0;
        let mut byte1_i = 1;

        loop {
            for b0 in 0..=255 {
                // original_buffer[byte0_i] = b0;
                for b1 in 0..=255 {
                    // original_buffer[byte1_i] = b1;

                    // println!("{} {}", byte0_i, byte1_i);
                    let expected = [
                        original_buffer[0..byte0_i].to_vec(),
                        vec![b0],
                        original_buffer[byte0_i + 1..byte1_i].to_vec(),
                        vec![b1],
                        original_buffer[byte1_i + 1..].to_vec(),
                    ]
                    .concat();

                    // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
                    // print_buffer("Expected", &expected);

                    assert_eq!(
                        expected,
                        payload.into_iter().collect::<Vec<u8>>(),
                        "\n{}\n{}",
                        buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
                        buffer_as_string("Expected", &expected),
                    );

                    // assert!(payload.advance().is_err(), "Should not have next",);

                    let _ = payload.advance();
                }
            }

            // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
            // print_buffer("Expected", &expected);

            if byte1_i >= original_buffer.len() - 1 {
                if byte0_i >= original_buffer.len() - 2 {
                    break;
                }
                byte0_i += 1;
                byte1_i = byte0_i + 1;
            } else {
                byte1_i += 1;
            }
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
        );
    }

    #[test]
    fn insert_one_locked_word() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                payload = '{"q":2@"q":3}'
                datatype = "Int"
                key = "q"
                [[payloads.fuzz]]
                fuzz_mode = { ReplaceCharacters = "@" }
                bytes = 2
        "#,
        )
        .unwrap();
        let buffer: Vec<u8> = config.payloads[0].payload.as_bytes().to_vec();
        let mut payload: Payload = config.payloads[0].clone().into();
        let i = 6;

        println!("{:?}", payload);
        // assert_eq!(payload.fuzz[0].max, u16::MAX as u32);

        for byte in 0..=(u16::MAX as u32) {
            let expected = [
                buffer[0..i].to_vec(),
                byte.to_be_bytes()[2..4].to_vec(),
                buffer[i + payload.fuzz[0].config.bytes..].to_vec(),
            ]
            .concat();

            // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
            // print_buffer("Expected", &expected);

            assert_eq!(
                expected,
                payload.into_iter().collect::<Vec<u8>>(),
                "\n{}\n{}",
                buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
                buffer_as_string("Expected", &expected),
            );

            let _ = payload.advance();
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
        );
    }

    #[test]
    fn insert_one_word() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                payload = '{"q":2,"q":3}'
                datatype = "Int"
                key = "q"
                [[payloads.fuzz]]
                bytes = 2
        "#,
        )
        .unwrap();
        let buffer: Vec<u8> = config.payloads[0].payload.as_bytes().to_vec();
        let mut payload: Payload = config.payloads[0].clone().into();

        // assert_eq!(payload.fuzz[0].max, u32::MAX & 0xffff);

        for i in 0..buffer.len() + 1 {
            for byte in 0..=(u16::MAX as u32) {
                let expected = [
                    buffer[0..i].to_vec(),
                    byte.to_be_bytes()[2..4].to_vec(),
                    buffer[i..].to_vec(),
                ]
                .concat();

                // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
                // print_buffer("Expected", &expected);

                assert_eq!(
                    expected,
                    payload.into_iter().collect::<Vec<u8>>(),
                    "{}\n{}",
                    buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
                    buffer_as_string("Expected", &expected),
                );

                // buffer[i] = buffer[i].wrapping_add(1);
                let _ = payload.advance();
            }

            // if i < buffer.len() - 1 {
            //     buffer[i] = buffer[i + 1];
            // }
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
        );
    }

    #[test]
    fn replace_quotes_and_insert_one() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                payload = '{"q":"a"}'
                datatype = "String"
                key = "q"
                [[payloads.fuzz]]
                fuzz_mode = { ReplaceCharacters = '"' }
                [[payloads.fuzz]]
        "#,
        )
        .unwrap();

        let mut buffer: Vec<u8> = config.payloads[0].payload.as_bytes().to_vec();
        let mut payload: Payload = config.payloads[0].clone().into();

        // println!(
        //     "{:?}",
        //     payload
        //         .fuzz
        //         .iter()
        //         .map(|f| (f.i, f.value, f.replace_value_with_i))
        //         .collect::<Vec<(usize, u32, Option<usize>)>>()
        // );

        let mut byte0_i = 0;
        // let mut byte1_i = 0;

        loop {
            for quote_byte in 0..=255 {
                buffer[1] = quote_byte;
                buffer[3] = quote_byte;
                buffer[5] = quote_byte;
                buffer[7] = quote_byte;

                for insert_byte in 0..=255 {
                    let expected = [
                        buffer[0..byte0_i].to_vec(),
                        vec![insert_byte],
                        buffer[byte0_i..].to_vec(),
                        // buffer[byte0_i..byte1_i].to_vec(),
                        // vec![b1],
                        // buffer[byte1_i..].to_vec(),
                    ]
                    .concat();

                    // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
                    // print_buffer("Expected", &expected);

                    assert_eq!(
                        expected,
                        payload.into_iter().collect::<Vec<u8>>(),
                        "{}\n{}",
                        buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
                        buffer_as_string("Expected", &expected),
                    );

                    // assert!(payload.advance().is_err(), "Should not have next",);

                    let _ = payload.advance();
                }
            }

            // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
            // print_buffer("Expected", &expected);

            // if byte1_i >= buffer.len() {
            if byte0_i >= buffer.len() {
                break;
            }
            byte0_i += 1;
            //     byte1_i = byte0_i;
            // } else {
            //     byte1_i += 1;
            // }
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
        );
    }

    #[test]
    fn insert_one_and_replace_word() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                payload = '42'
                datatype = "String"
                key = "q"
                [[payloads.fuzz]]
                [[payloads.fuzz]]
                fuzz_mode = "ReplaceAllPossible"
                bytes = 2
        "#,
        )
        .unwrap();

        let mut buffer: Vec<u8> = config.payloads[0].payload.as_bytes().to_vec();
        let original_buffer: Vec<u8> = config.payloads[0].payload.as_bytes().to_vec();
        let mut payload: Payload = config.payloads[0].clone().into();

        println!("{:?}", payload);

        let mut insert_i = 0;
        let mut replace_i = 0;

        loop {
            for replace_byte in 0..=u16::MAX {
                buffer[replace_i] = ((replace_byte >> 8) & 0xff) as u8;
                buffer[replace_i + 1] = (replace_byte & 0xff) as u8;

                for insert_byte in 0..=255 {
                    let expected = [
                        buffer[0..insert_i].to_vec(),
                        vec![insert_byte],
                        buffer[insert_i..].to_vec(),
                    ]
                    .concat();

                    // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
                    // print_buffer("Expected", &expected);

                    assert_eq!(
                        expected,
                        payload.into_iter().collect::<Vec<u8>>(),
                        "{}\n{}",
                        buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
                        buffer_as_string("Expected", &expected),
                    );

                    // assert!(payload.advance().is_err(), "Should not have next",);

                    let _ = payload.advance();
                }

                buffer[replace_i] = original_buffer[replace_i];
                buffer[replace_i + 1] = original_buffer[replace_i + 1];
            }

            // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
            // print_buffer("Expected", &expected);

            if insert_i < buffer.len() {
                insert_i += 1;
            } else {
                if replace_i >= buffer.len() - 2 {
                    break;
                }
                insert_i = 0;
                replace_i += 1;
            }
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
        );
    }

    #[test]
    fn combine_all_modes() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                payload = '[0]'
                datatype = "String"
                key = "q"
                [[payloads.fuzz]]
                [[payloads.fuzz]]
                fuzz_mode = "ReplaceAllPossible"
                [[payloads.fuzz]]
                fuzz_mode = { ReplaceCharacters = '[]'}
        "#,
        )
        .unwrap();

        let mut buffer: Vec<u8> = config.payloads[0].payload.as_bytes().to_vec();
        let original_buffer: Vec<u8> = config.payloads[0].payload.as_bytes().to_vec();
        let mut payload: Payload = config.payloads[0].clone().into();

        println!("{:?}", payload.fuzz);

        let mut insert_i = 0;
        let mut replace_i = 0;

        loop {
            for replace_chars_byte in 0..=255 {
                buffer[0] = replace_chars_byte;
                buffer[2] = replace_chars_byte;

                for replace_byte in 0..=255 {
                    // TODO - does some unnecessary work when replace and
                    // replace locked share an index
                    if !(replace_byte == 0 && (replace_i == 0 || replace_i == 2)) {
                        buffer[replace_i] = replace_byte;
                    }

                    for insert_byte in 0..=255 {
                        let expected = [
                            buffer[0..insert_i].to_vec(),
                            vec![insert_byte],
                            buffer[insert_i..].to_vec(),
                        ]
                        .concat();

                        // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
                        // print_buffer("Expected", &expected);

                        assert_eq!(
                            expected,
                            payload.into_iter().collect::<Vec<u8>>(),
                            "{} {}\n{}\n{}",
                            replace_i,
                            insert_i,
                            buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
                            buffer_as_string("Expected", &expected),
                        );

                        // assert!(payload.advance().is_err(), "Should not have next",);

                        let _ = payload.advance();
                    }
                }
                buffer[replace_i] = original_buffer[replace_i];
            }

            // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
            // print_buffer("Expected", &expected);

            if insert_i < buffer.len() {
                insert_i += 1;
            } else {
                insert_i = 0;

                if replace_i >= buffer.len() - 1 {
                    break;
                }
                replace_i += 1;
            }
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
        );
    }

    #[test]
    fn min_max() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                payload = '[0]'
                datatype = "String"
                key = "q"
                [[payloads.fuzz]]
                fuzz_range = {MinMax = { min = 7, max = 13 }}
                [[payloads.fuzz]]
                fuzz_mode = "ReplaceAllPossible"
                fuzz_range = {MinMax = { min = 48, max = 57 }}
        "#,
        )
        .unwrap();

        let mut buffer: Vec<u8> = config.payloads[0].payload.as_bytes().to_vec();
        let original_buffer: Vec<u8> = config.payloads[0].payload.as_bytes().to_vec();
        let mut payload: Payload = config.payloads[0].clone().into();

        println!("{:?}", payload.fuzz);

        let mut insert_i = 0;
        let mut replace_i = 0;

        loop {
            for replace_byte in 48..=57 {
                buffer[replace_i] = replace_byte;

                for insert_byte in 7..=13 {
                    let expected = [
                        buffer[0..insert_i].to_vec(),
                        vec![insert_byte],
                        buffer[insert_i..].to_vec(),
                    ]
                    .concat();

                    // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
                    // print_buffer("Expected", &expected);

                    assert_eq!(
                        expected,
                        payload.into_iter().collect::<Vec<u8>>(),
                        "{} {}\n{}\n{}",
                        replace_i,
                        insert_i,
                        buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
                        buffer_as_string("Expected", &expected),
                    );

                    // assert!(payload.advance().is_err(), "Should not have next",);

                    let _ = payload.advance();
                }
                buffer[replace_i] = original_buffer[replace_i];
            }

            // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
            // print_buffer("Expected", &expected);

            if insert_i < buffer.len() {
                insert_i += 1;
            } else {
                insert_i = 0;

                if replace_i >= buffer.len() - 1 {
                    break;
                }
                replace_i += 1;
            }
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
        );
    }

    #[test]
    fn prefix() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                payload = '{"q":1}'
                datatype = "String"
                key = "q"
                [[payloads.fuzz]]
                prefix = '%'
                [[payloads.fuzz]]
                prefix = '\u00'
                fuzz_mode = "ReplaceAllPossible"
                fuzz_range = { Characters = { chars = '0123456789abcdef', length = 2 } }
                bytes = 2
        "#,
        )
        .unwrap();

        let mut buffer: Vec<u8> = config.payloads[0].payload.as_bytes().to_vec();
        let original_buffer: Vec<u8> = config.payloads[0].payload.as_bytes().to_vec();
        let mut payload: Payload = config.payloads[0].clone().into();

        println!("{:?}", payload.fuzz);

        let mut insert_i = 0;
        let mut replace_i = 4;

        let chars: Vec<u8> = "0123456789abcdef".bytes().collect();

        loop {
            for replace_byte in 0..16 * 16 {
                let r0 = chars[replace_byte / 16];
                let r1 = chars[replace_byte % 16];
                buffer[replace_i - 4] = '\\' as u8;
                buffer[replace_i - 3] = 'u' as u8;
                buffer[replace_i - 2] = '0' as u8;
                buffer[replace_i - 1] = '0' as u8;
                buffer[replace_i + 0] = r0;
                buffer[replace_i + 1] = r1;

                for insert_byte in 0..=255 {
                    let expected = [
                        buffer[0..insert_i].to_vec(),
                        vec!['%' as u8],
                        vec![insert_byte],
                        buffer[insert_i..].to_vec(),
                    ]
                    .concat();

                    print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
                    print_buffer("Expected", &expected);

                    assert_eq!(
                        expected,
                        payload.into_iter().collect::<Vec<u8>>(),
                        "{} {}\n{}\n{}",
                        replace_i,
                        insert_i,
                        buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
                        buffer_as_string("Expected", &expected),
                    );

                    // assert!(payload.advance().is_err(), "Should not have next",);

                    let _ = payload.advance();
                }

                buffer[replace_i - 4] = original_buffer[replace_i - 4];
                buffer[replace_i - 3] = original_buffer[replace_i - 3];
                buffer[replace_i - 2] = original_buffer[replace_i - 2];
                buffer[replace_i - 1] = original_buffer[replace_i - 1];
                buffer[replace_i + 0] = original_buffer[replace_i + 0];
                buffer[replace_i + 1] = original_buffer[replace_i + 1];
            }

            // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
            // print_buffer("Expected", &expected);

            if insert_i < buffer.len() {
                insert_i += 1;
            } else {
                insert_i = 0;

                if replace_i >= buffer.len() - 2 {
                    break;
                }
                replace_i += 1;
            }
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
        );
    }

    #[test]
    fn prefix2() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                payload = '{"@.....":1}'
                datatype = "String"
                key = "q"
                [[payloads.fuzz]]
                prefix = '\u'
                fuzz_mode = { ReplaceCharacters = '@'}
                fuzz_range = { Characters = { chars = '0123456789abcdef', length = 4 } }
                bytes = 4
        "#,
        )
        .unwrap();

        let mut buffer: Vec<u8> = config.payloads[0].payload.as_bytes().to_vec();
        let mut payload: Payload = config.payloads[0].clone().into();

        println!("{:?}", payload.fuzz);

        let chars: Vec<u8> = "0123456789abcdef".bytes().collect();

        for r0 in 0..16 {
            buffer[2] = '\\' as u8;
            buffer[3] = 'u' as u8;
            buffer[4] = chars[r0];
            for r1 in 0..16 {
                buffer[5] = chars[r1];
                for r2 in 0..16 {
                    buffer[6] = chars[r2];
                    for r3 in 0..16 {
                        buffer[7] = chars[r3];

                        // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
                        // print_buffer("Expected", &buffer);

                        assert_eq!(
                            buffer,
                            payload.into_iter().collect::<Vec<u8>>(),
                            "\n{}\n{}",
                            buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
                            buffer_as_string("Expected", &buffer),
                        );

                        // assert!(payload.advance().is_err(), "Should not have next",);

                        let _ = payload.advance();
                    }
                }
            }
        }

        // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
        // print_buffer("Expected", &expected);

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
        );
    }
}
