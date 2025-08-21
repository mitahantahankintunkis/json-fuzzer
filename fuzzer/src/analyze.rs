use crate::compression::*;
use crate::payload::*;
use crate::util::byte_to_string;
// use regex::Regex;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
// use std::path::Path;
// use std::ops::Range;

#[derive(Clone)]
struct FuzzingResult {
    parser_name: String,
    // test_name: String,
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
    payload_config: PayloadConfig,
    results: &mut Vec<FuzzingResult>,
) -> HashSet<Vulnerability> {
    if results.len() == 0 {
        return HashSet::new();
    }

    // for result in &mut *results {
    //     result.decoder.next_message().unwrap();
    // }

    let mut vulnerabilities: HashSet<Vulnerability> = HashSet::new();

    let mut payload: Payload = payload_config.clone().into();

    if results.len() == 0 {
        panic!("No results for {}", payload_config.name);
    }

    loop {
        let mut parser_output: Vec<String> = Vec::new();

        for result in &mut *results {
            match result.decoder.next_message() {
                Some(r) => {
                    // println!(
                    //     "{} {} {}",
                    //     payload_config.name,
                    //     payload
                    //         .into_iter()
                    //         .map(|c| byte_to_string(c))
                    //         .collect::<Vec<String>>()
                    //         .join(""),
                    //     r.to_string()
                    // );
                    parser_output.push(r.to_string());
                }
                None => {
                    // for test_result in &mut *results {
                    //     if test_result.decoder.next_message().is_some() {
                    //         eprintln!("Result decoder size mismatch");
                    //     }
                    // }
                    break;
                }
            }
        }

        let mut payload_str = String::with_capacity(64);

        for byte in payload.into_iter() {
            payload_str.push_str(&byte_to_string(byte));
        }

        if payload.advance().is_err() {
            break;
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

                if !((output0 == "2" && output1 == "3") || (output0 == "3" && output1 == "2")) {
                    continue;
                }

                let vuln = Vulnerability {
                    payload: payload_str.clone(),
                    parser_0_name: result0.parser_name.clone(),
                    parser_1_name: result1.parser_name.clone(),
                    parser_0_output: output0.clone(),
                    parser_1_output: output1.clone(),
                };

                match vulnerabilities.get(&vuln) {
                    Some(best_vuln) => {
                        if vuln.payload.len() < best_vuln.payload.len() {
                            vulnerabilities.insert(vuln);
                        }
                    }
                    None => {
                        vulnerabilities.insert(vuln);
                    }
                }
            }
        }
    }

    vulnerabilities
}

pub fn analyze(_args: &crate::Args) {
    let mut vulns: HashSet<Vulnerability> = HashSet::new();

    let config = load_payloads();

    for payload_config in &config.payloads {
        let files = std::fs::read_dir("../data/").expect("Could not open '../data/'");
        let mut results: Vec<FuzzingResult> = Vec::new();

        for file in files {
            if let Ok(f) = file {
                let file_name: String = f.file_name().to_str().unwrap().to_string();
                if !file_name.contains(&payload_config.name) {
                    continue;
                }

                let split: Vec<&str> = file_name.splitn(2, '-').collect();

                if split.len() != 2 {
                    eprintln!("Malformed filename '{}'", file_name);
                    continue;
                }

                let bytes = std::fs::read(f.path()).expect("Could not read file");

                // TODO - remove
                if bytes.len() >= 10_000_000 {
                    println!("Skipping {} due to size", file_name);
                    continue;
                }

                let result = FuzzingResult {
                    parser_name: split[0].to_string(),
                    decoder: Decoder::new(Box::new(bytes)),
                };

                results.push(result);
            }
        }

        results.sort_by(|a, b| a.parser_name.cmp(&b.parser_name));
        for vuln in analyze_results(payload_config.clone(), &mut results) {
            vulns.insert(vuln);
        }
    }

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
