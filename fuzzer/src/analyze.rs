use crate::compression::*;
use crate::payload::*;
// use regex::Regex;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::ops::Range;

struct FuzzingResult {
    parser_name: String,
    decoder: Decoder,
}

#[derive(Clone)]
struct Vulnerability {
    payload: String,
    parser_0_name: String,
    parser_1_name: String,
    parser_0_output: String,
    parser_1_output: String,
}

impl Hash for Vulnerability {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.parser_0_name.hash(state);
        self.parser_1_name.hash(state);
        self.parser_0_output.hash(state);
        self.parser_1_output.hash(state);

        // let re = Regex::new(r"\d+").unwrap();
        // re.replace_all(&self.payload, "").hash(state);
    }
}

impl PartialEq for Vulnerability {
    fn eq(&self, other: &Self) -> bool {
        // lt re = Regex::new(r"\d+").unwrap();
        self.parser_0_name == other.parser_0_name && self.parser_1_name == other.parser_1_name
        // && re.replace_all(&self.payload, "") == re.replace_all(&other.payload, "")
        && self.parser_0_output == other.parser_0_output
        && self.parser_1_output == other.parser_1_output
        // && self.payload == other.payload
    }
}

impl Eq for Vulnerability {}

fn analyze_results(
    fuzzing_type: &FuzzingType,
    results: &mut Vec<FuzzingResult>,
) -> HashSet<Vulnerability> {
    if results.len() == 0 {
        return HashSet::new();
    }

    // print!("{:20}\t", "payload");
    //
    let mut base_payload: &str = "";
    //
    for result in &mut *results {
        // print!("{:20}\t", result.parser_name);
        base_payload = result.decoder.next_message().unwrap();
    }
    //
    // println!("");

    let mut payload: Box<dyn Payload> = match fuzzing_type {
        FuzzingType::ReplaceOneByte => Box::new(ReplaceBytes::new(base_payload.as_bytes(), 1)),
        FuzzingType::ReplaceTwoBytes => Box::new(ReplaceBytes::new(base_payload.as_bytes(), 2)),
        FuzzingType::ReplaceThreeBytes => Box::new(ReplaceBytes::new(base_payload.as_bytes(), 3)),
        FuzzingType::InsertOneByte => Box::new(InsertBytes::new(base_payload.as_bytes(), 1)),
        FuzzingType::InsertTwoBytes => Box::new(InsertBytes::new(base_payload.as_bytes(), 2)),
        FuzzingType::InsertThreeBytes => Box::new(InsertBytes::new(base_payload.as_bytes(), 3)),
        FuzzingType::ReplaceOneUnicodeByte => Box::new(ReplaceFormatted::new(
            base_payload.as_bytes(),
            1,
            Range::<usize> {
                start: 0,
                end: 0x10000,
            },
            |b| format!("\\u{:04x}", b),
        )),
        FuzzingType::ReplaceTwoUnicodeBytes => Box::new(ReplaceFormatted::new(
            base_payload.as_bytes(),
            1,
            Range::<usize> {
                start: 0,
                end: 0x10000,
            },
            |b| format!("\\u{:04x}", b),
        )),

        FuzzingType::ReplaceThreeUnicodeBytes => Box::new(ReplaceFormatted::new(
            base_payload.as_bytes(),
            1,
            Range::<usize> {
                start: 0,
                end: 0x10000,
            },
            |b| format!("\\u{:04x}", b),
        )),

        FuzzingType::InsertOneUnicodeByte => Box::new(ReplaceFormatted::new(
            base_payload.as_bytes(),
            1,
            Range::<usize> {
                start: 0,
                end: 0x10000,
            },
            |b| format!("\\u{:04x}", b),
        )),

        FuzzingType::InsertTwoUnicodeBytes => Box::new(ReplaceFormatted::new(
            base_payload.as_bytes(),
            1,
            Range::<usize> {
                start: 0,
                end: 0x10000,
            },
            |b| format!("\\u{:04x}", b),
        )),

        FuzzingType::InsertThreeUnicodeBytes => Box::new(ReplaceFormatted::new(
            base_payload.as_bytes(),
            1,
            Range::<usize> {
                start: 0,
                end: 0x10000,
            },
            |b| format!("\\u{:04x}", b),
        )),
    };

    // let mut analyzed: HashSet<String> = HashSet::new();
    let mut vulnerabilities: HashSet<Vulnerability> = HashSet::new();

    // let mut messages: usize = 0;

    loop {
        payload.next();
        let mut parser_output: Vec<String> = Vec::new();
        // let mut parser_names: Vec<String> = Vec::new();
        // messages += 1;

        for result in &mut *results {
            match result.decoder.next_message() {
                Some(r) => parser_output.push(r.to_string()),
                None => {
                    // println!("Parsed: {} {}", messages, result.decoder.messages_parsed);

                    for test_result in &mut *results {
                        if test_result.decoder.next_message().is_some() {
                            eprintln!("Result decoder size mismatch");
                        }
                    }
                    break;
                }
            }
        }

        if parser_output.len() != results.len() {
            break;
        }

        let first_value = match parser_output
            .iter()
            .find(|e| *e != "PARSE_ERROR" && *e != "KEY_NOT_FOUND")
        {
            Some(e) => e,
            None => continue,
        };

        let mut equal = true;

        // Equal output
        for output in &parser_output {
            if *output != *first_value && *output != "PARSE_ERROR" && *output != "KEY_NOT_FOUND" {
                equal = false;
                break;
            }
        }

        if equal {
            continue;
        }

        let buffer = payload.get_payload();
        let mut payload_str = String::with_capacity(buffer.len());

        for i in 0..buffer.len() {
            match buffer[i] {
                0u8..0x20 => payload_str.push_str(format!("\\x{:02x}", buffer[i]).as_str()),
                0x7fu8..=0xff => payload_str.push_str(format!("\\x{:02x}", buffer[i]).as_str()),
                b => payload_str.push(b as char),
            }
        }

        // if !analyzed.contains(&payload_str) {
        //     print!("{:20}\t", payload_str);
        //
        //     for output in &parser_output {
        //         print!("{:13}\t", output);
        //     }
        //     println!("");
        //     analyzed.insert(payload_str.clone());
        // }

        for i in 0..results.len() {
            let result0 = &results[i];
            let output0 = &parser_output[i];

            if output0 == "KEY_NOT_FOUND" || output0 == "PARSE_ERROR" {
                continue;
            }

            for j in 0..results.len() {
                if i == j {
                    continue;
                }

                let result1 = &results[j];
                let output1 = &parser_output[j];

                if output1 == "KEY_NOT_FOUND" || output1 == "PARSE_ERROR" {
                    continue;
                }

                if output0 == output1 {
                    continue;
                }

                if !((output0 == "1" && output1 == "2") || (output0 == "2" && output1 == "1")) {
                    continue;
                }

                let vuln = Vulnerability {
                    payload: payload_str.clone(),
                    parser_0_name: result0.parser_name.clone(),
                    parser_1_name: result1.parser_name.clone(),
                    parser_0_output: output0.clone(),
                    parser_1_output: output1.clone(),
                };

                vulnerabilities.insert(vuln);
            }
        }
    }

    vulnerabilities
    // .iter()
    // .map(|v| v.clone())
    // .collect::<Vec<Vulnerability>>()
}

pub fn analyze(args: &crate::Args) {
    let fuzzing_types = &vec![
        FuzzingType::InsertOneByte,
        FuzzingType::InsertOneUnicodeByte,
        FuzzingType::InsertTwoBytes,
        FuzzingType::InsertTwoUnicodeBytes,
        FuzzingType::ReplaceOneByte,
        FuzzingType::ReplaceOneUnicodeByte,
        FuzzingType::ReplaceTwoBytes,
        FuzzingType::ReplaceTwoUnicodeBytes,
        FuzzingType::ReplaceThreeBytes,
        FuzzingType::InsertThreeBytes,
        FuzzingType::ReplaceThreeUnicodeBytes,
        FuzzingType::InsertThreeUnicodeBytes,
    ];

    let digest = md5::compute(args.payload.as_bytes());
    let hash: String = format!("{:x}", digest);
    let mut vulns: HashSet<Vulnerability> = HashSet::new();

    for fuzzing_type in fuzzing_types {
        let files = std::fs::read_dir("../data/").expect("Could not open '../data/'");
        let mut results: Vec<FuzzingResult> = Vec::new();

        for file in files {
            if let Ok(f) = file {
                let file_name: String = f.file_name().to_str().unwrap().to_string();
                if !file_name.contains(&hash) || !file_name.contains(&fuzzing_type.to_string()) {
                    continue;
                }

                let split: Vec<&str> = file_name.splitn(3, '_').collect();

                if split.len() != 3 {
                    eprintln!("Malformed filename '{}'", file_name);
                    continue;
                }

                let bytes = std::fs::read(f.path()).expect("Could not read file");

                let result = FuzzingResult {
                    parser_name: split[0].to_string(),
                    decoder: Decoder::new(Box::new(bytes)),
                };

                results.push(result);
            }
        }

        results.sort_by(|a, b| a.parser_name.cmp(&b.parser_name));
        for vuln in analyze_results(fuzzing_type, &mut results) {
            vulns.insert(vuln);
        }
    }

    println!();

    //     {
    //     a.parser_0_name
    //         .cmp(&b.parser_0_name)
    //         .cmp(&a.parser_1_name.cmp(&b.parser_1_name))
    // });

    let mut vulns_vec: Vec<&Vulnerability> = vulns.iter().collect();

    vulns_vec.sort_by_key(|v| {
        (
            v.parser_0_name.clone(),
            v.parser_1_name.clone(),
            v.parser_0_output.clone(),
            v.parser_1_output.clone(),
        )
    });

    for vuln in vulns_vec {
        println!(
            "{}\t{}\t{}\t{}\t{}",
            vuln.parser_0_name,
            vuln.parser_1_name,
            vuln.payload,
            vuln.parser_0_output,
            vuln.parser_1_output
        );
    }
}
