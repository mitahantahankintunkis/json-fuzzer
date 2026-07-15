#![allow(unused)]
use serde::Deserialize;

#[allow(unused)]
use std::{
    cmp::{max, min, Ordering},
    fs,
    ops::Range,
};

use crate::{
    fuzz::{Fuzzer, TestCase},
    util::decode_str,
};

#[derive(Deserialize, Debug)]
#[allow(unused)]
pub struct Config {
    pub payloads: Vec<PayloadConfig>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PayloadConfig {
    pub name: String,
    pub json: String,
    #[serde(default)]
    pub fuzz: Vec<FuzzConfig>,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub enum Datatype {
    #[default]
    Int,
    Float,
    String,
    Object,
    Array,
    Null,
    Bool,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct FuzzConfig {
    pub range: Option<Range<u32>>,
    pub indices: Option<Range<usize>>,
    #[serde(default)]
    pub prefix: String,
    // #[serde(default)]
    // pub suffix: String,
    pub map: Option<FuzzMapConfig>,
    #[serde(default)]
    pub remove_after: usize,
    pub id: Option<String>,
    pub inherit_value_from: Option<String>,
    pub right_of: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Fuzz {
    pub value: u32,
    pub index: usize,
    pub map: FuzzMap,
    pub range: Range<u32>,
    pub indices: Range<usize>,
    pub prefix: Vec<u8>,
    // pub suffix: Vec<u8>,
    pub remove_after: usize,
    pub inherit_value_from: Option<usize>,
    pub right_of: Option<usize>,
    pub left_of: Option<usize>,
    mapped_length: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub enum FuzzMapConfig {
    Word {
        byte_count: usize,
    },
    Characters {
        chars: String,
        byte_count: usize,
    },
    ReplaceCharacters {
        chars: String,
        replace_with: Box<FuzzMapConfig>,
    },
}

#[derive(Debug, Clone)]
pub enum FuzzMap {
    Word { byte_count: usize },
    Bytes { bytes: Vec<u8>, byte_count: usize },
}

impl FuzzMap {
    #[inline(always)]
    pub fn map(&self, i: u32, byte_offset: usize) -> u8 {
        match self {
            FuzzMap::Word { byte_count } => {
                (i >> ((byte_count - byte_offset - 1) * 8) & 0xff) as u8
            }
            FuzzMap::Bytes { bytes, byte_count } => {
                let len = bytes.len();
                bytes[((i as usize) / (len.pow((byte_count - byte_offset - 1) as u32)) as usize)
                    % len]
            }
        }
    }
}

#[derive(Debug)]
pub struct OKFuzz {
    pub byte_count: usize,
    pub bytes: Vec<u8>,
    pub config: PayloadConfig,
    pub fuzz: Vec<Fuzz>,
    sorted_fuzz_lookup: Vec<usize>,
    inverse_sorted_fuzz_lookup: Vec<usize>,
    finished: bool,
    id: String,
}

impl OKFuzz {
    pub fn insert_single(test_case: &TestCase) -> Self {
        PayloadConfig {
            name: "insert_single".to_string(),
            json: test_case.json.clone(),
            fuzz: vec![FuzzConfig::default()],
        }
        .into()
    }

    pub fn insert_two(test_case: &TestCase) -> Self {
        let mut ret = PayloadConfig {
            name: "insert_two".to_string(),
            json: test_case.json.clone(),
            fuzz: vec![FuzzConfig::default(), FuzzConfig::default()],
        };

        ret.fuzz[0].id = Some("0".to_string());
        ret.fuzz[1].right_of = Some("0".to_string());

        ret.into()
    }

    pub fn remove_single(test_case: &TestCase) -> Self {
        let mut ret = PayloadConfig {
            name: "remove_single".to_string(),
            json: test_case.json.clone(),
            // datatype: Datatype::Int,
            // key: test_case.key.clone(),
            fuzz: vec![FuzzConfig::default()],
        };

        ret.fuzz[0].remove_after = 1;

        ret.into()
    }

    pub fn remove_two(test_case: &TestCase) -> Self {
        let mut ret = PayloadConfig {
            name: "remove_two".to_string(),
            json: test_case.json.clone(),
            fuzz: vec![FuzzConfig::default(), FuzzConfig::default()],
        };

        ret.fuzz[0].id = Some("0".to_string());
        ret.fuzz[1].right_of = Some("0".to_string());
        ret.fuzz[0].remove_after = 1;
        ret.fuzz[1].remove_after = 1;

        ret.into()
    }

    pub fn insert_unicode(test_case: &TestCase) -> Self {
        let mut ret = PayloadConfig {
            name: "insert_unicode".to_string(),
            json: test_case.json.clone(),
            fuzz: vec![FuzzConfig::default()],
        };

        ret.fuzz[0].map = Some(FuzzMapConfig::Characters {
            chars: "0123456789abcdef".to_string(),
            byte_count: 4,
        });
        ret.fuzz[0].prefix = "\\u".to_string();

        ret.into()
    }

    pub fn replace_unicode(test_case: &TestCase) -> Self {
        let mut ret = PayloadConfig {
            name: "replace_unicode".to_string(),
            json: test_case.json.clone(),
            fuzz: vec![FuzzConfig::default()],
        };

        ret.fuzz[0].map = Some(FuzzMapConfig::Characters {
            chars: "0123456789abcdef".to_string(),
            byte_count: 4,
        });
        ret.fuzz[0].prefix = "\\u".to_string();
        ret.fuzz[0].remove_after = 1;

        ret.into()
    }

    pub fn insert_single_word(test_case: &TestCase) -> Self {
        let mut ret = PayloadConfig {
            name: "insert_single_word".to_string(),
            json: test_case.json.clone(),
            fuzz: vec![FuzzConfig::default()],
        };

        ret.fuzz[0].map = Some(FuzzMapConfig::Word { byte_count: 2 });

        ret.into()
    }

    pub fn replace_single_word(test_case: &TestCase) -> Self {
        let mut ret = PayloadConfig {
            name: "replace_single_word".to_string(),
            json: test_case.json.clone(),
            fuzz: vec![FuzzConfig::default()],
        };

        ret.fuzz[0].map = Some(FuzzMapConfig::Word { byte_count: 2 });
        ret.fuzz[0].remove_after = 1;

        ret.into()
    }

    pub fn insert_grammar(test_case: &TestCase) -> Self {
        let mut ret = PayloadConfig {
            name: "insert_grammar".to_string(),
            json: test_case.json.clone(),
            fuzz: vec![FuzzConfig::default(), FuzzConfig::default()],
        };

        ret.fuzz[0].map = Some(FuzzMapConfig::Characters {
            chars: "09e.-[]{}\",".to_string(),
            byte_count: 1,
        });
        ret.fuzz[1].map = Some(FuzzMapConfig::Characters {
            chars: "09e.-[]{}\",".to_string(),
            byte_count: 1,
        });
        ret.fuzz[0].id = Some("0".into());
        ret.fuzz[1].right_of = Some("0".into());

        ret.into()
    }

    // Goes through all fuzz byte placements until it finds a valid one
    pub fn init(&mut self) -> Result<(), ()> {
        if self.fuzz.len() == 0 {
            return Ok(());
        }

        for fuzz in &mut self.fuzz {
            fuzz.index = fuzz.indices.start;
        }

        self.sorted_fuzz_lookup = (0..self.fuzz.len()).collect::<Vec<usize>>();
        self.inverse_sorted_fuzz_lookup = (0..self.fuzz.len()).collect::<Vec<usize>>();

        fn rec(payload: &mut OKFuzz, fuzz_i: usize) -> (bool, usize) {
            payload.fuzz[fuzz_i].index = payload.fuzz[fuzz_i].indices.start;
            payload.sort();

            let mut total_permutations = 1;

            while payload.fuzz[fuzz_i].index <= payload.fuzz[fuzz_i].indices.end {
                if fuzz_i > 0 {
                    let (res, perm) = rec(payload, fuzz_i - 1);
                    if res {
                        return (true, total_permutations);
                    }
                    total_permutations += perm;
                }

                if total_permutations > 100000 {
                    panic!("Possible infinite loop in init");
                }

                if payload.fully_valid() {
                    return (true, total_permutations);
                }

                let _ = payload.shift_fuzz(fuzz_i, true);
                payload.sort();

                if payload.fuzz[fuzz_i].index == payload.fuzz[fuzz_i].indices.end {
                    break;
                }
            }

            (false, total_permutations)
        }

        let l = self.fuzz.len() - 1;

        if rec(self, l).0 {
            Ok(())
        } else {
            Err(())
        }
    }

    #[inline]
    pub fn sort(&mut self) {
        self.sorted_fuzz_lookup
            .sort_by_key(|i| (self.byte_count as isize) - (self.fuzz[*i].index as isize));

        for (sorted_i, fuzz_i) in self.sorted_fuzz_lookup.iter().enumerate() {
            self.inverse_sorted_fuzz_lookup[*fuzz_i] = sorted_i;
        }
    }

    pub fn fully_valid(&self) -> bool {
        for i in 0..self.fuzz.len() {
            if !self.valid(i) {
                return false;
            }
        }

        true
    }

    #[inline]
    pub fn valid(&self, fuzz_i: usize) -> bool {
        let fuzz = &self.fuzz[fuzz_i];

        for fuzz_j in 0..self.fuzz.len() {
            if fuzz_i == fuzz_j {
                continue;
            }

            let fuzz2 = &self.fuzz[fuzz_j];

            let sorted_i0 = self.inverse_sorted_fuzz_lookup[fuzz_i];
            let sorted_i1 = self.inverse_sorted_fuzz_lookup[fuzz_j];

            if sorted_i0 > sorted_i1 && fuzz.remove_after == 0 && fuzz.index == fuzz2.index {
                continue;
            }

            if sorted_i0 < sorted_i1 && fuzz2.remove_after == 0 && fuzz.index == fuzz2.index {
                continue;
            }

            // Before
            if fuzz2.remove_after > 0
                && fuzz2.index <= fuzz.index
                && fuzz2.index + fuzz2.remove_after > fuzz.index
            {
                return false;
            }

            // After
            if fuzz.remove_after > 0
                && fuzz.index <= fuzz2.index
                && fuzz.index + fuzz.remove_after > fuzz2.index
            {
                return false;
            }
        }

        true
    }

    #[inline]
    pub fn shift_fuzz(&mut self, fuzz_i: usize, once: bool) -> Result<(), ()> {
        let mut sorted_i = self.inverse_sorted_fuzz_lookup[fuzz_i];

        while self.fuzz[fuzz_i].index <= self.fuzz[fuzz_i].indices.end {
            if sorted_i > 0 {
                let next_i = self.sorted_fuzz_lookup[sorted_i - 1];

                if self.fuzz[next_i].index == self.fuzz[fuzz_i].index {
                    // if let Some(right_i) = self.fuzz[next_i].right_of {
                    //     if right_i == fuzz_i {
                    //         return Err(());
                    //     }
                    // }

                    self.sorted_fuzz_lookup.swap(sorted_i, sorted_i - 1);
                    self.inverse_sorted_fuzz_lookup.swap(fuzz_i, next_i);

                    if self.valid(fuzz_i) {
                        return Ok(());
                    }

                    sorted_i -= 1;
                }
            }

            if self.fuzz[fuzz_i].index == self.fuzz[fuzz_i].indices.end {
                break;
            }

            self.fuzz[fuzz_i].index += 1;

            if self.valid(fuzz_i) {
                return Ok(());
            }

            if once {
                break;
            }
        }

        Err(())
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut buf = vec![0u8; self.byte_count];

        match self.copy_to_slice(&mut buf) {
            Ok(n) => {
                buf.truncate(n);
                buf
            }
            Err(_) => Vec::new(),
        }
    }
}

impl Fuzzer for OKFuzz {
    fn advance(&mut self) -> Result<(), ()> {
        if self.finished {
            return Err(());
        }

        // Increase fuzz values
        for fuzz in &mut self.fuzz {
            if fuzz.inherit_value_from.is_some() {
                continue;
            }

            fuzz.value += 1;

            if fuzz.value < fuzz.range.end {
                return Ok(());
            }

            fuzz.value = fuzz.range.start;
        }

        // Overflow - each fuzz byte is at their minimum value

        // All finished
        for fuzz_i in 0..self.fuzz.len() {
            let fuzz = &self.fuzz[fuzz_i];

            if fuzz.index != fuzz.indices.end {
                break;
            }

            if fuzz_i == self.fuzz.len() - 1 {
                self.finished = true;
                return Err(());
            }
        }

        // Move fuzz values to right
        for fuzz_i in 0..self.fuzz.len() {
            if self.shift_fuzz(fuzz_i, false).is_ok() {
                for fuzz_j in (0..fuzz_i).rev() {
                    self.fuzz[fuzz_j].index = if let Some(right_i) = self.fuzz[fuzz_j].right_of {
                        max(self.fuzz[right_i].index, self.fuzz[fuzz_j].indices.start)
                    } else {
                        self.fuzz[fuzz_j].indices.start
                    };

                    if !self.valid(fuzz_j) && self.shift_fuzz(fuzz_j, false).is_err() {
                        self.finished = true;
                        return Err(());
                    }
                }

                self.sort();
                return Ok(());
            }

            self.fuzz[fuzz_i].index = self.fuzz[fuzz_i].indices.end;
            self.sort();
        }

        self.finished = true;
        Err(())
    }

    #[inline(always)]
    fn copy_to_slice(&self, buf: &mut [u8]) -> Result<usize, ()> {
        if self.byte_count > buf.len() {
            return Err(());
        }

        let mut byte_i = 0;
        let mut i = 0;

        for fuzz_i in (0..self.fuzz.len()).rev() {
            let fuzz_i = self.sorted_fuzz_lookup[fuzz_i];
            let fuzz = &self.fuzz[fuzz_i];

            if fuzz.index > byte_i {
                let n = fuzz.index - byte_i;
                buf[i..i + n].copy_from_slice(&self.bytes[byte_i..byte_i + n]);
                // std::ptr::copy_nonoverlapping(src_ptr, buf_ptr, n);
                // buf_ptr = buf_ptr.add(n);
                // src_ptr = src_ptr.add(n);
                i += n;
                byte_i += n;
            }

            let fuzz = if let Some(i) = fuzz.inherit_value_from {
                &self.fuzz[i]
            } else {
                fuzz
            };

            let fuzz_prefix_len = fuzz.prefix.len();
            // let fuzz_suffix_len = fuzz.suffix.len();

            if fuzz_prefix_len > 0 {
                buf[i..i + fuzz_prefix_len].copy_from_slice(&fuzz.prefix);
                i += fuzz_prefix_len;
                // std::ptr::copy_nonoverlapping(fuzz.prefix.as_ptr(), buf_ptr, fuzz_prefix_len);
                // buf_ptr = buf_ptr.add(fuzz_prefix_len);
            }

            for j in 0..fuzz.mapped_length {
                buf[i] = fuzz.map.map(fuzz.value, j);
                i += 1;
                // *buf_ptr = fuzz.map.map(fuzz.value, j);
                // buf_ptr = buf_ptr.add(1);
            }

            // if fuzz_suffix_len > 0 {
            //     std::ptr::copy_nonoverlapping(fuzz.suffix.as_ptr(), buf_ptr, fuzz_suffix_len);
            //     buf_ptr = buf_ptr.add(fuzz_suffix_len);
            // }

            byte_i += fuzz.remove_after;
        }

        if self.bytes.len() > byte_i {
            let n = self.bytes.len() - byte_i;
            buf[i..i + n].copy_from_slice(&self.bytes[byte_i..byte_i + n]);
            // std::ptr::copy_nonoverlapping(src_ptr, buf_ptr, n);
        }
        // }

        Ok(self.byte_count)
    }

    fn id(&self) -> String {
        self.id.clone()
    }
}

impl From<PayloadConfig> for OKFuzz {
    fn from(config: PayloadConfig) -> Self {
        let fuzzed_payload: Vec<u8> = decode_str(&config.json);
        let mut fuzz_configs: Vec<FuzzConfig> = Vec::new();

        // Convert FuzzMapConfig::ReplaceCharacters
        for c in &config.fuzz {
            match &c.map {
                Some(FuzzMapConfig::ReplaceCharacters {
                    chars,
                    replace_with,
                }) => {
                    match **replace_with {
                        FuzzMapConfig::ReplaceCharacters {
                            chars: _,
                            replace_with: _,
                        } => panic!("Can not replace with 'ReplaceCharacters'"),
                        _ => {}
                    }

                    for c0 in chars.chars() {
                        let mut parent_id = String::new();

                        for (i, c1) in config.json.chars().enumerate() {
                            if c0 == c1 {
                                let mut new_config = c.clone();
                                new_config.map = Some((**replace_with).clone());
                                new_config.indices = Some(i..i);

                                if parent_id.is_empty() {
                                    parent_id = format!("_{}_id", &chars);
                                    new_config.id = Some(parent_id.clone());
                                } else {
                                    new_config.inherit_value_from = Some(parent_id.clone());
                                }

                                fuzz_configs.push(new_config);
                            }
                        }
                    }
                }
                _ => fuzz_configs.push(c.clone()),
            };
        }

        // Sort configs. This determines the update order of the fuzzed values
        fuzz_configs.sort_by(|a, b| {
            if let Some(id0) = &a.right_of {
                if let Some(id1) = &b.id {
                    if id0 == id1 {
                        return Ordering::Less;
                    }
                }
            }

            if let Some(id0) = &b.right_of {
                if let Some(id1) = &a.id {
                    if id0 == id1 {
                        return Ordering::Greater;
                    }
                }
            }

            if let Some(id0) = &a.inherit_value_from {
                if let Some(id1) = &b.id {
                    if id0 == id1 {
                        return Ordering::Greater;
                    }
                }
            }

            if let Some(in0) = &a.indices {
                if let Some(in1) = &b.indices {
                    return in1.start.cmp(&in0.start);
                }
            }

            if let Some(_in0) = &a.indices {
                if b.indices.is_none() {
                    return Ordering::Less;
                }
            }

            if let Some(_in0) = &b.indices {
                if a.indices.is_none() {
                    return Ordering::Greater;
                }
            }

            b.remove_after.cmp(&a.remove_after)
        });

        // Convert fuzz configs to fuzz
        let mut fuzzes: Vec<Fuzz> = Vec::new();

        for c in &fuzz_configs {
            let mut indices = c.indices.clone().unwrap_or(0..fuzzed_payload.len());
            indices.end = min(indices.end, fuzzed_payload.len() - c.remove_after);

            let map: FuzzMap = match c.map.clone() {
                Some(FuzzMapConfig::Word { byte_count }) => FuzzMap::Word { byte_count },
                Some(FuzzMapConfig::Characters { chars, byte_count }) => FuzzMap::Bytes {
                    bytes: decode_str(&chars),
                    byte_count,
                },
                _ => FuzzMap::Word { byte_count: 1 },
            };

            let (mapped_length, max_val) = match &map {
                FuzzMap::Word { byte_count } => (byte_count, 1u32 << (byte_count * 8)),
                FuzzMap::Bytes { bytes, byte_count } => {
                    (byte_count, bytes.len().pow(*byte_count as u32) as u32)
                }
            };

            let range = match c.range.clone() {
                Some(r) => r,
                None => 0..(max_val),
            };

            if range.end > max_val {
                panic!(
                    "Fuzzing range exceeds maximum fuzz value. {} does not fit into {} bytes",
                    range.end - 1,
                    mapped_length
                );
            }

            fuzzes.push(Fuzz {
                mapped_length: *mapped_length,
                map,
                value: range.start,
                index: usize::MAX,
                prefix: decode_str(&c.prefix),
                // suffix: decode_str(&c.suffix),
                range: range,
                indices: indices,
                remove_after: c.remove_after,
                inherit_value_from: None,
                right_of: None,
                left_of: None,
            });
        }

        // Value inheritance and right_of
        for (i, c0) in fuzz_configs.iter().enumerate() {
            if let Some(id0) = &c0.inherit_value_from {
                for (j, c1) in fuzz_configs.iter().enumerate() {
                    if let Some(id1) = &c1.id {
                        if id0 == id1 {
                            fuzzes[i].inherit_value_from = Some(j);
                            break;
                        }
                    }
                }
            }

            if let Some(id0) = &c0.right_of {
                for (j, c1) in fuzz_configs.iter().enumerate() {
                    if let Some(id1) = &c1.id {
                        if id0 == id1 {
                            fuzzes[i].right_of = Some(j);
                            fuzzes[j].left_of = Some(i);
                            break;
                        }
                    }
                }
            }
        }

        let byte_count: isize = (fuzzed_payload.len() as isize)
            + fuzzes
                .iter()
                .map(|f| {
                    (f.prefix.len() as isize)/* + (f.suffix.len() as isize) */
                        - (f.remove_after as isize)
                        + (f.mapped_length as isize)
                })
                .sum::<isize>();

        let sorted_fuzz = (0..fuzzes.len()).collect::<Vec<usize>>();

        let mut ret = OKFuzz {
            byte_count: byte_count
                .try_into()
                .expect("Invalid total byte count in payload"),
            fuzz: fuzzes,
            sorted_fuzz_lookup: sorted_fuzz.clone(),
            inverse_sorted_fuzz_lookup: sorted_fuzz,
            bytes: fuzzed_payload,
            finished: false,
            id: config.name.clone(),
            config,
        };

        if ret.init().is_err() {
            panic!("Could not init Payload: {:#?}", ret);
        }

        ret
    }
}

// pub struct PayloadIter<'a> {
//     pub payload: &'a ComprehensiveFuzzer,
//     pub byte_i: usize,
//     pub fuzz_i: usize,
//     pub fuzz_byte_offset: usize,
//     // pub skip: usize,
//     pub truncated: usize,
// }
//
// // Iterates payload bytes.
// // Inserts fuzzed values into the payload
// impl<'a> Iterator for PayloadIter<'a> {
//     type Item = u8;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.fuzz_i > 0 {
//             let fuzz_i = self.payload.sorted_fuzz_lookup[self.fuzz_i - 1];
//             let fuzz = &self.payload.fuzz[fuzz_i];
//
//             if fuzz.index <= self.byte_i {
//                 let fuzz = if let Some(i) = fuzz.inherit_value_from {
//                     &self.payload.fuzz[i]
//                 } else {
//                     fuzz
//                 };
//
//                 let o0 = self.fuzz_byte_offset as i32;
//                 let o1 = o0 - fuzz.prefix.len() as i32;
//                 let o2 = o1 - fuzz.mapped_length as i32;
//
//                 let byte = if o0 < fuzz.prefix.len() as i32 {
//                     fuzz.prefix[self.fuzz_byte_offset]
//                 } else if o1 < fuzz.mapped_length as i32 {
//                     fuzz.map.map(fuzz.value, o1 as usize)
//                 // } else if o2 < fuzz.suffix.len() as i32 {
//                 //     fuzz.suffix[o2 as usize]
//                 } else {
//                     0
//                 };
//
//                 self.fuzz_byte_offset += 1;
//
//                 // if o2 + 1 >= fuzz.suffix.len() as i32 {
//                 if o2 + 1 >= 0 {
//                     self.fuzz_i -= 1;
//                     self.fuzz_byte_offset = 0;
//                     self.byte_i += fuzz.remove_after;
//                     self.truncated += fuzz.remove_after;
//                 }
//
//                 return Some(byte);
//             }
//         }
//
//         if self.byte_i >= self.payload.bytes.len() {
//             return None;
//         }
//
//         self.byte_i += 1;
//         Some(self.payload.bytes[self.byte_i - 1])
//     }
// }
//
// impl<'a> IntoIterator for &'a ComprehensiveFuzzer {
//     type Item = u8;
//     type IntoIter = PayloadIter<'a>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         PayloadIter {
//             payload: self,
//             byte_i: 0,
//             fuzz_i: self.fuzz.len(),
//             fuzz_byte_offset: 0,
//             // skip: 0,
//             truncated: 0,
//         }
//     }
// }

// 3.3s in total
#[cfg(test)]
mod tests {
    use crate::util::{byte_to_string, decode_str};

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

    #[allow(unused)]
    fn print_buffer(name: &str, buffer: &[u8]) {
        println!("{}", buffer_as_string(name, buffer));
    }

    #[test]
    fn creation() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "empty"
                json = 'null'
                datatype = "Null"
                key = "q"

                [[payloads]]
                name = "duplicate_keys_single_byte"
                json = '{"q":2,"q":3}'
                datatype = "Int"
                key = "q"
                [[payloads.fuzz]]

                [[payloads]]
                name = "duplicate_keys_two_bytes"
                json = '{"q":2,"q":3}'
                datatype = "Int"
                key = "q"
                [[payloads.fuzz]]
                [[payloads.fuzz]]

                [[payloads]]
                name = "all_fuzz_modes_single_byte"
                json = '{"q":"a"}'
                datatype = "String"
                key = "q"
                [[payloads.fuzz]]
                map = { Word = { byte_count = 1 }}
                [[payloads.fuzz]]
                map = { Characters = { chars = '\x00\x01A', byte_count = 1 }}
                prefix = 'asdf'
                remove_after = 2
        "#,
            // suffix = 'qwer'
        )
        .unwrap();

        assert_eq!(config.payloads.len(), 4);
        let p = &config.payloads[3];
        assert!(matches!(
            p.fuzz[0].map,
            Some(FuzzMapConfig::Word { byte_count: 1 })
        ));

        match &p.fuzz[1].map {
            Some(FuzzMapConfig::Characters {
                chars,
                byte_count: _,
            }) => {
                assert_eq!(decode_str(&chars)[..], [0x0, 0x1, 0x41])
            }
            _ => panic!(),
        }
        assert_eq!(p.fuzz[1].prefix, "asdf".to_string());
        // assert_eq!(p.fuzz[1].suffix, "qwer".to_string());
        assert_eq!(p.fuzz[1].remove_after, 2);
    }

    // 0.29s
    #[test]
    fn performance1() {
        // Chained right_of don't work properly
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                json = ''
                [[payloads.fuzz]]
                id = '0'
                map = { Word = { byte_count = 2 }}
                [[payloads.fuzz]]
                right_of = '0'
                map = { Word = { byte_count = 2 }}
        "#,
        )
        .unwrap();

        let mut payload: OKFuzz = config.payloads[0].clone().into();
        println!("{:?}", payload.config);
        let mut result = [0u8; 4];

        for i in 0..(u32::MAX & 0xffffffff) {
            // let mut it = payload.into_iter();
            assert_eq!(payload.copy_to_slice(&mut result), Ok(4));
            // let d = it.next().unwrap();
            // let c = it.next().unwrap();
            // let b = it.next().unwrap();
            // let a = it.next().unwrap();
            // println!("{} {} {} {}", a, b, c, d);
            // println!("{} {} {}", a, b, c);

            // assert_eq!(((i >> 24) & 0xff) as u8, d);
            // assert_eq!(((i >> 16) & 0xff) as u8, c);
            // assert_eq!(((i >> 8) & 0xff) as u8, b);
            // assert_eq!(((i >> 0) & 0xff) as u8, a);
            assert_eq!(((i >> 24) & 0xff) as u8, result[0]);
            assert_eq!(((i >> 16) & 0xff) as u8, result[1]);
            assert_eq!(((i >> 8) & 0xff) as u8, result[2]);
            assert_eq!(((i >> 0) & 0xff) as u8, result[3]);
            // assert!(it.next().is_none());
            let _ = payload.advance();
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}\n{:#?}",
            // buffer_as_string("Payload", &payload.into_iter().collect::<Vec<u8>>()),
            buffer_as_string("Payload", &payload.as_bytes()),
            payload.fuzz
        );
    }

    #[test]
    fn performance2() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                json = '{"":2,"q":3}'
                [[payloads.fuzz]]
                map = { Word = { byte_count = 2 }}
                indices = { start = 2, end = 2 }
                id = '0'
                [[payloads.fuzz]]
                right_of = '0'
                map = { Word = { byte_count = 2 }}
                indices = { start = 2, end = 2 }
        "#,
        )
        .unwrap();

        let mut payload: OKFuzz = config.payloads[0].clone().into();
        println!("{:?}", payload.config);
        let mut result = [0u8; 16];

        for i in 0..(u32::MAX & 0xffffffff) {
            // let mut it = payload.into_iter();
            assert_eq!(payload.copy_to_slice(&mut result), Ok(16));
            // println!("{:02x?}", result);

            // let d = it.next().unwrap();
            // let c = it.next().unwrap();
            // let b = it.next().unwrap();
            // let a = it.next().unwrap();
            // println!("{} {} {} {}", a, b, c, d);
            // println!("{} {} {}", a, b, c);

            // assert_eq!(((i >> 24) & 0xff) as u8, d);
            // assert_eq!(((i >> 16) & 0xff) as u8, c);
            // assert_eq!(((i >> 8) & 0xff) as u8, b);
            // assert_eq!(((i >> 0) & 0xff) as u8, a);
            assert_eq!(((i >> 24) & 0xff) as u8, result[2]);
            assert_eq!(((i >> 16) & 0xff) as u8, result[3]);
            assert_eq!(((i >> 8) & 0xff) as u8, result[4]);
            assert_eq!(((i >> 0) & 0xff) as u8, result[5]);
            // assert!(it.next().is_none());
            let _ = payload.advance();
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}\n{:#?}",
            buffer_as_string("Payload", &payload.as_bytes()),
            payload.fuzz
        );
    }

    #[test]
    fn performance3() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                json = '{"":2,"q":3}'
                [[payloads.fuzz]]
                map = { Word = { byte_count = 1 }}
                indices = { start = 2, end = 2 }
                id = '0'
                [[payloads.fuzz]]
                right_of = '0'
                map = { Word = { byte_count = 2 }}
                indices = { start = 2, end = 2 }
        "#,
        )
        .unwrap();

        let mut payload: OKFuzz = config.payloads[0].clone().into();
        println!("{:?}", payload.config);
        let mut result = [0u8; 15];

        for i in 0..(u32::MAX & 0x00ffffff) {
            assert_eq!(payload.copy_to_slice(&mut result), Ok(15));
            // println!("{:02x?}", result);

            assert_eq!(((i >> 16) & 0xff) as u8, result[2]);
            assert_eq!(((i >> 8) & 0xff) as u8, result[3]);
            assert_eq!(((i >> 0) & 0xff) as u8, result[4]);
            let _ = payload.advance();
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}\n{:#?}",
            buffer_as_string("Payload", &payload.as_bytes()),
            payload.fuzz
        );
    }

    #[test]
    fn advance_single_locked() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                json = '{"@":2,"q":3}'
                [[payloads.fuzz]]
                indices = { start = 2, end = 2 }
                remove_after = 1
        "#,
        )
        .unwrap();

        let mut payload: OKFuzz = config.payloads[0].clone().into();
        let mut bytes: Vec<u8> = payload.config.json.bytes().collect();
        let mut byte = 0u8;

        loop {
            bytes[2] = byte;

            // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
            // print_buffer("Expected", &bytes);

            for (i, byte) in payload.as_bytes().iter().enumerate() {
                assert_eq!(*byte, bytes[i]);
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
                json = '{"q":2,"q":3}'
                [[payloads.fuzz]]
        "#,
        )
        .unwrap();
        let mut buffer: Vec<u8> = [vec![0], config.payloads[0].json.as_bytes().to_vec()].concat();
        let mut payload: OKFuzz = config.payloads[0].clone().into();

        println!("{:#?}", config);
        println!("{:#?}", payload);

        for i in 0..buffer.len() {
            buffer[i] = 0;

            for _ in 0..=255 {
                // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
                // print_buffer("Expected", &buffer);

                assert_eq!(buffer, payload.as_bytes());

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
            buffer_as_string("Payload", &payload.as_bytes()),
        );
    }

    #[test]
    fn insert_two_bytes() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                json = '{"q":"a"}'
                datatype = "String"
                key = "q"
                [[payloads.fuzz]]
                id = "0"
                [[payloads.fuzz]]
                right_of = "0"
        "#,
        )
        .unwrap();

        let original_buffer: Vec<u8> = config.payloads[0].json.as_bytes().to_vec();
        let mut payload: OKFuzz = config.payloads[0].clone().into();

        println!("{:#?}", payload);

        let mut byte0_i = 0;
        let mut byte1_i = 0;

        loop {
            for b0 in 0..=255 {
                for b1 in 0..=255 {
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
                        payload.as_bytes(),
                        "{}\n{}\n{:#?}",
                        buffer_as_string("Payload", &payload.as_bytes()),
                        buffer_as_string("Expected", &expected),
                        payload.fuzz,
                    );

                    let _ = payload.advance();
                }
            }

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
            buffer_as_string("Payload", &payload.as_bytes()),
        );
    }

    #[test]
    fn insert_two_bytes_slow() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_insert_two_bytes"
                json = '{"q":2,"q":3}'
                datatype = "Int"
                key = "q"
                [[payloads.fuzz]]
                [[payloads.fuzz]]
        "#,
        )
        .unwrap();

        let original_buffer: Vec<u8> = config.payloads[0].json.as_bytes().to_vec();
        let mut payload: OKFuzz = config.payloads[0].clone().into();

        println!("{:#?}", payload);

        let mut byte0_i = 0;
        let mut byte1_i = 0;
        let mut byte0_first = true;

        loop {
            for b0 in 0..=255 {
                for b1 in 0..=255 {
                    let expected = if byte0_first {
                        [
                            original_buffer[0..byte0_i].to_vec(),
                            vec![b0],
                            original_buffer[byte0_i..byte1_i].to_vec(),
                            vec![b1],
                            original_buffer[byte1_i..].to_vec(),
                        ]
                    } else {
                        [
                            original_buffer[0..byte1_i].to_vec(),
                            vec![b1],
                            original_buffer[byte1_i..byte0_i].to_vec(),
                            vec![b0],
                            original_buffer[byte0_i..].to_vec(),
                        ]
                    }
                    .concat();

                    // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
                    // print_buffer("Expected", &expected);

                    assert_eq!(
                        expected,
                        payload.as_bytes(),
                        "\n{}\n{}\n{:#?}",
                        buffer_as_string("Payload", &payload.as_bytes()),
                        buffer_as_string("Expected", &expected),
                        payload.fuzz,
                    );

                    let _ = payload.advance();
                }
            }

            if byte1_i >= original_buffer.len() {
                if byte0_i >= original_buffer.len() {
                    break;
                }
                byte0_i += 1;
                byte1_i = 0;
                byte0_first = false;
            } else {
                if byte0_i == byte1_i && !byte0_first {
                    byte0_first = true;
                } else {
                    byte1_i += 1;

                    if byte0_i == byte1_i {
                        byte0_first = false;
                    }
                }
            }
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.as_bytes()),
        );
    }

    #[test]
    fn replace_two_bytes() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                json = '{"q":"a"}'
                datatype = "String"
                key = "q"
                [[payloads.fuzz]]
                remove_after = 1
                id = "0"
                [[payloads.fuzz]]
                remove_after = 1
                right_of = "0"
        "#,
        )
        .unwrap();

        let original_buffer: Vec<u8> = config.payloads[0].json.as_bytes().to_vec();
        let mut payload: OKFuzz = config.payloads[0].clone().into();

        println!("{:#?}", payload);

        let mut byte0_i = 0;
        let mut byte1_i = 1;

        loop {
            for b0 in 0..=255 {
                for b1 in 0..=255 {
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
                        payload.as_bytes(),
                        "\n{}\n{}\n{:#?}",
                        buffer_as_string("Payload", &payload.as_bytes()),
                        buffer_as_string("Expected", &expected),
                        payload.fuzz
                    );

                    let _ = payload.advance();
                }
            }

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

        println!("{:#?}", payload.fuzz);
        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.as_bytes()),
        );
    }

    #[test]
    fn replace_two_bytes_slow() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_insert_two_bytes"
                json = '{"q":2,"q":3}'
                [[payloads.fuzz]]
                remove_after = 1
                [[payloads.fuzz]]
                remove_after = 1
        "#,
        )
        .unwrap();

        let original_buffer: Vec<u8> = config.payloads[0].json.as_bytes().to_vec();
        let mut payload: OKFuzz = config.payloads[0].clone().into();

        println!("{:#?}", payload);

        let mut byte0_i = 0;
        let mut byte1_i = 1;

        loop {
            for b0 in 0..=255 {
                for b1 in 0..=255 {
                    let expected = if byte0_i <= byte1_i {
                        [
                            original_buffer[0..byte0_i].to_vec(),
                            vec![b0],
                            original_buffer[byte0_i + 1..byte1_i].to_vec(),
                            vec![b1],
                            original_buffer[byte1_i + 1..].to_vec(),
                        ]
                    } else {
                        [
                            original_buffer[0..byte1_i].to_vec(),
                            vec![b1],
                            original_buffer[byte1_i + 1..byte0_i].to_vec(),
                            vec![b0],
                            original_buffer[byte0_i + 1..].to_vec(),
                        ]
                    }
                    .concat();

                    // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
                    // print_buffer("Expected", &expected);

                    assert_eq!(
                        expected,
                        payload.as_bytes(),
                        "\n{}\n{}\n{:#?}",
                        buffer_as_string("Payload", &payload.as_bytes()),
                        buffer_as_string("Expected", &expected),
                        payload.fuzz,
                    );

                    let _ = payload.advance();
                }
            }

            if byte1_i >= original_buffer.len() - 1 {
                if byte0_i >= original_buffer.len() - 2 {
                    break;
                }
                byte0_i += 1;
                byte1_i = 0;
            } else {
                byte1_i += 1;

                if byte0_i == byte1_i {
                    byte1_i += 1;
                }
            }
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.as_bytes()),
        );
    }

    #[test]
    fn prefix() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_replace_unicode"
                json = '{"q":2,"q":3}'
                [[payloads.fuzz]]
                map = { Characters = { chars = '0123456789abcdef', byte_count = 4 }}
                prefix = '\u'
                remove_after = 1
        "#,
        )
        .unwrap();

        let buffer: Vec<u8> = config.payloads[0].json.as_bytes().to_vec();
        let mut payload: OKFuzz = config.payloads[0].clone().into();

        println!("{:#?}", payload);

        let chars = "0123456789abcdef".bytes().into_iter().collect::<Vec<u8>>();
        let mut byte0_i = 0;

        loop {
            for b0 in 0..16 {
                for b1 in 0..16 {
                    for b2 in 0..16 {
                        for b3 in 0..16 {
                            let expected = [
                                buffer[0..byte0_i].to_vec(),
                                "\\u".bytes().into_iter().collect(),
                                vec![chars[b0], chars[b1], chars[b2], chars[b3]],
                                buffer[byte0_i + 1..].to_vec(),
                            ]
                            .concat();

                            // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
                            // print_buffer("Expected", &expected);

                            assert_eq!(
                                expected,
                                payload.as_bytes(),
                                "\n{}\n{}\n{:#?}",
                                buffer_as_string("Payload", &payload.as_bytes()),
                                buffer_as_string("Expected", &expected),
                                payload.fuzz,
                            );

                            let _ = payload.advance();
                        }
                    }
                }
            }

            if byte0_i >= buffer.len() - 1 {
                break;
            }

            byte0_i += 1;
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.as_bytes()),
        );
    }
    #[test]
    fn insert_one_locked_word() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                json = '{"q":2@"q":3}'
                datatype = "Int"
                key = "q"
                [[payloads.fuzz]]
                indices = { start = 6, end = 6 }
                remove_after = 1
                map = { Word = { byte_count = 2 }}
        "#,
        )
        .unwrap();
        let buffer: Vec<u8> = config.payloads[0].json.as_bytes().to_vec();
        let mut payload: OKFuzz = config.payloads[0].clone().into();
        let i = 6;

        println!("{:#?}", payload);

        for byte in 0..=(u16::MAX as u32) {
            let expected = [
                buffer[0..i].to_vec(),
                byte.to_be_bytes()[2..4].to_vec(),
                buffer[i + 1..].to_vec(),
            ]
            .concat();

            // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
            // print_buffer("Expected", &expected);

            assert_eq!(
                expected,
                payload.as_bytes(),
                "\n{}\n{}",
                buffer_as_string("Payload", &payload.as_bytes()),
                buffer_as_string("Expected", &expected),
            );

            let _ = payload.advance();
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.as_bytes()),
        );
    }

    #[test]
    fn insert_one_word() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                json = '{"q":2,"q":3}'
                datatype = "Int"
                key = "q"
                [[payloads.fuzz]]
                map = { Word = { byte_count = 2 }}
        "#,
        )
        .unwrap();
        let buffer: Vec<u8> = config.payloads[0].json.as_bytes().to_vec();
        let mut payload: OKFuzz = config.payloads[0].clone().into();

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
                    payload.as_bytes(),
                    "{}\n{}",
                    buffer_as_string("Payload", &payload.as_bytes()),
                    buffer_as_string("Expected", &expected),
                );

                let _ = payload.advance();
            }
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.as_bytes()),
        );
    }

    #[test]
    fn replace_quotes_and_insert_one() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                json = '{"q":"a"}'
                datatype = "String"
                key = "q"
                [[payloads.fuzz]]
                [[payloads.fuzz]]
                map = { ReplaceCharacters = { chars = '"', replace_with = { Word = { byte_count = 1 }}}}
                remove_after = 1
        "#,
        )
        .unwrap();

        let mut buffer: Vec<u8> = config.payloads[0].json.as_bytes().to_vec();
        let mut payload: OKFuzz = config.payloads[0].clone().into();

        let mut byte0_i = 0;

        loop {
            for insert_byte in 0..=255 {
                for quote_byte in 0..=255 {
                    buffer[1] = quote_byte;
                    buffer[3] = quote_byte;
                    buffer[5] = quote_byte;
                    buffer[7] = quote_byte;

                    let expected = [
                        buffer[0..byte0_i].to_vec(),
                        vec![insert_byte],
                        buffer[byte0_i..].to_vec(),
                    ]
                    .concat();

                    // print_buffer("Payload", &payload.into_iter().collect::<Vec<u8>>());
                    // print_buffer("Expected", &expected);

                    assert_eq!(
                        expected,
                        payload.as_bytes(),
                        "{}\n{}\n{:#?}\n{:#?}",
                        buffer_as_string("Payload", &payload.as_bytes()),
                        buffer_as_string("Expected", &expected),
                        payload.fuzz,
                        payload.sorted_fuzz_lookup,
                    );

                    let _ = payload.advance();
                }
            }
            if byte0_i >= buffer.len() {
                break;
            }
            byte0_i += 1;
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.as_bytes()),
        );
    }

    #[test]
    fn insert_one_and_replace_word() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                json = '42'
                datatype = "String"
                key = "q"
                [[payloads.fuzz]]
                [[payloads.fuzz]]
                map = { Word = { byte_count = 2 } }
                remove_after = 2
                fuzz_mode = "ReplaceAllPossible"
                bytes = 2
        "#,
        )
        .unwrap();

        let mut buffer: Vec<u8> = config.payloads[0].json.as_bytes().to_vec();
        let mut payload: OKFuzz = config.payloads[0].clone().into();

        println!("{:#?}", payload);

        let mut insert_i = 0;

        loop {
            for insert_byte in 0..=255 {
                for replace_byte in 0..=u16::MAX {
                    buffer[0] = ((replace_byte >> 8) & 0xff) as u8;
                    buffer[1] = (replace_byte & 0xff) as u8;

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
                        payload.as_bytes(),
                        "\n{}\n{}\n{:#?}\n{:?}",
                        buffer_as_string("Payload", &payload.as_bytes()),
                        buffer_as_string("Expected", &expected),
                        payload.fuzz,
                        payload.sorted_fuzz_lookup
                    );

                    let _ = payload.advance();
                }
            }

            if insert_i >= buffer.len() {
                break;
            }
            insert_i += 2;
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.as_bytes()),
        );
    }

    #[test]
    fn combine_all_modes() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                json = '[2]'
                datatype = "String"
                key = "q"
                [[payloads.fuzz]]
                range = { start = 0, end = 16 }
                [[payloads.fuzz]]
                range = { start = 16, end = 32 }
                remove_after = 1
                [[payloads.fuzz]]
                map = { ReplaceCharacters = { chars = '[]', replace_with = { Word = { byte_count = 1 }}}}
                range = { start = 32, end = 48 }
                remove_after = 1
        "#,
        )
        .unwrap();

        let mut buffer: Vec<u8> = config.payloads[0].json.as_bytes().to_vec();
        let mut payload: OKFuzz = config.payloads[0].clone().into();

        println!("{:#?}", payload.fuzz);

        let mut insert_i = 0;

        loop {
            for insert_byte in 0..16 {
                for replace_byte in 16..32 {
                    buffer[1] = replace_byte;

                    for replace_chars_byte1 in 32..48 {
                        buffer[0] = replace_chars_byte1;
                        for replace_chars_byte0 in 32..48 {
                            buffer[2] = replace_chars_byte0;
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
                                payload.as_bytes(),
                                "\n{}\n{}\n{:#?}\n{:?}",
                                buffer_as_string("Payload", &payload.as_bytes(),),
                                buffer_as_string("Expected", &expected),
                                payload.fuzz,
                                payload.sorted_fuzz_lookup,
                            );

                            let _ = payload.advance();
                        }
                    }
                }
            }

            if insert_i < buffer.len() {
                insert_i += 1;
            } else {
                break;
            }
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.as_bytes()),
        );
    }

    #[test]
    fn min_max() {
        let config: Config = toml::from_str(
            r#"
                [[payloads]]
                name = "duplicate_keys_single_byte"
                json = '[0]'
                datatype = "String"
                key = "q"
                [[payloads.fuzz]]
                range = { start = 7, end = 14 }
                fuzz_range = {MinMax = { min = 7, max = 13 }}
                [[payloads.fuzz]]
                range = { start = 48, end = 58 }
                remove_after = 1
                fuzz_mode = "ReplaceAllPossible"
                fuzz_range = {MinMax = { min = 48, max = 57 }}
        "#,
        )
        .unwrap();

        let mut buffer: Vec<u8> = config.payloads[0].json.as_bytes().to_vec();
        let original_buffer: Vec<u8> = config.payloads[0].json.as_bytes().to_vec();
        let mut payload: OKFuzz = config.payloads[0].clone().into();

        println!("{:#?}", payload.fuzz);

        let mut insert_i = 0;
        let mut replace_i = 0;

        loop {
            for insert_byte in 7..=13 {
                for replace_byte in 48..=57 {
                    buffer[replace_i] = replace_byte;

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
                        payload.as_bytes(),
                        "{} {}\n{}\n{}\n{:#?}\n{:?}",
                        replace_i,
                        insert_i,
                        buffer_as_string("Payload", &payload.as_bytes()),
                        buffer_as_string("Expected", &expected),
                        payload.fuzz,
                        payload.sorted_fuzz_lookup
                    );

                    let _ = payload.advance();
                }
                buffer[replace_i] = original_buffer[replace_i];
            }

            if replace_i < buffer.len() - 1 {
                replace_i += 1;
            } else {
                replace_i = 0;

                if insert_i >= buffer.len() {
                    break;
                }

                insert_i += 1;
            }
        }

        assert!(
            payload.advance().is_err(),
            "Should not have next\n{}",
            buffer_as_string("Payload", &payload.as_bytes()),
        );
    }
}
