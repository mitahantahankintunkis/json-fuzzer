use crate::compression::*;
use crate::payload::*;
use crate::util::byte_to_string;
use std::collections::HashMap;
// use regex::Regex;
// use std::hash::DefaultHasher;
use std::hash::{Hash, Hasher};
// use std::rc::Rc;
// use std::path::Path;
// use std::ops::Range;

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
    hash: u32,
}

fn vuln_hash(name0: &str, name1: &str, out0: &str, out1: &str) -> u32 {
    // let mut hasher = DefaultHasher::new();
    // name0.hash(&mut hasher);
    // name1.hash(&mut hasher);
    // out0.hash(&mut hasher);
    // out1.hash(&mut hasher);
    // hasher.finish()

    // djb2 in C++
    // unsigned long
    // hash(unsigned char *str)
    // {
    //     unsigned long hash = 5381;
    //     int c;
    //
    //     while (c = *str++)
    //         hash = ((hash << 5) + hash) + c; /* hash * 33 + c */
    //
    //     return hash;
    // }

    let mut hash: u32 = 5381;

    for byte in name0.bytes() {
        hash = ((hash << 5) + hash) + byte as u32;
    }

    for byte in name1.bytes() {
        hash = ((hash << 5) + hash) + byte as u32;
    }

    for byte in out0.bytes() {
        hash = ((hash << 5) + hash) + byte as u32;
    }

    for byte in out1.bytes() {
        hash = ((hash << 5) + hash) + byte as u32;
    }

    hash
}

impl Vulnerability {
    fn new(payload: &str, name0: &str, name1: &str, out0: &str, out1: &str) -> Self {
        Vulnerability {
            payload: payload.to_string(),
            parser_0_name: name0.to_string(),
            parser_1_name: name1.to_string(),
            parser_0_output: out0.to_string(),
            parser_1_output: out1.to_string(),
            hash: vuln_hash(name0, name1, out0, out1),
        }
    }
}

impl Hash for Vulnerability {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for Vulnerability {
    fn eq(&self, other: &Self) -> bool {
        self.parser_0_name == other.parser_0_name
            && self.parser_1_name == other.parser_1_name
            && self.parser_0_output == other.parser_0_output
            && self.parser_1_output == other.parser_1_output
    }
}

impl Eq for Vulnerability {}

fn analyze_results(
    payload_config: PayloadConfig,
    results: &Vec<FuzzingResult>,
) -> Vec<(u32, Vulnerability)> {
    if results.len() == 0 {
        return Vec::new();
        // return HashMap::new();
    }

    let mut vulnerabilities: Vec<(u32, Vulnerability)> = Vec::new();
    // let mut vulnerabilities: HashMap<u32, Vulnerability> = HashMap::new();

    let mut payload: Payload = payload_config.clone().into();

    if results.len() == 0 {
        panic!("No results for {}", payload_config.name);
    }

    let mut payload_str = String::with_capacity(64);
    let mut parser_outputs: Vec<(&str, &str)> = Vec::with_capacity(results.len());

    // Storing decoder states here gets around some issues with the borrow checker
    let mut decoder_states: Vec<DecoderState> = Vec::new();

    for _ in 0..results.len() {
        decoder_states.push(DecoderState::default());
    }

    // Pop parser names
    for (i, result) in results.iter().enumerate() {
        let _ = result
            .decoder
            .next_message_with_state(&mut decoder_states[i]);
    }

    loop {
        parser_outputs.clear();
        payload_str.clear();

        for (i, result) in results.iter().enumerate() {
            let res = result
                .decoder
                .next_message_with_state(&mut decoder_states[i]);

            match res {
                Some(r) => {
                    if r != "PARSE_ERROR" && r != "KEY_NOT_FOUND" {
                        parser_outputs.push((&result.parser_name, r));
                    }
                }
                None => {
                    panic!(
                        "No next message {} {} {}",
                        result.parser_name,
                        payload_config.name,
                        payload
                            .into_iter()
                            .map(|c| byte_to_string(c))
                            .collect::<Vec<String>>()
                            .join(""),
                    );
                }
            }
        }

        if parser_outputs.len() == 0 {
            if payload.advance().is_err() {
                break;
            }

            continue;
        }

        // if parser_output.len() != results.len() {
        //     break;
        // }

        let first_value = &parser_outputs[0];
        let mut equal = true;

        // Equal output
        for output in &parser_outputs {
            if *output != *first_value {
                equal = false;
                break;
            }
        }

        if equal {
            if payload.advance().is_err() {
                break;
            }
            continue;
        }

        for i in 0..parser_outputs.len() - 1 {
            let (name0, output0) = &parser_outputs[i];

            for j in i + 1..parser_outputs.len() {
                let (name1, output1) = &parser_outputs[j];

                if output0 == output1 {
                    continue;
                }

                if !((*output0 == "2" && *output1 == "3") || (*output0 == "3" && *output1 == "2")) {
                    continue;
                }

                if payload_str.len() == 0 {
                    for byte in payload.into_iter() {
                        payload_str.push_str(&byte_to_string(byte));
                    }
                }

                let hash = vuln_hash(&name0, &name1, output0, output1);
                let mut found = false;

                for i in 0..vulnerabilities.len() {
                    let best_vuln = &vulnerabilities[i];

                    if hash == best_vuln.0 {
                        if payload_str.len() < best_vuln.1.payload.len() {
                            let vuln =
                                Vulnerability::new(&payload_str, &name0, &name1, output0, output1);
                            vulnerabilities[i] = (hash, vuln);
                        }

                        found = true;
                        break;
                    }
                }

                if !found {
                    let vuln = Vulnerability::new(&payload_str, &name0, &name1, output0, output1);
                    vulnerabilities.push((hash, vuln));
                }

                // match vulnerabilities.get(&hash) {
                //     Some(best_vuln) => {
                //         if payload_str.len() < best_vuln.payload.len() {
                //             let vuln =
                //                 Vulnerability::new(&payload_str, &name0, &name1, output0, output1);
                //             vulnerabilities.insert(hash, vuln);
                //         }
                //     }
                //     None => {
                //         let vuln =
                //             Vulnerability::new(&payload_str, &name0, &name1, output0, output1);
                //         vulnerabilities.insert(hash, vuln);
                //     }
                // }
            }
        }

        if payload.advance().is_err() {
            break;
        }
    }

    vulnerabilities
}

pub fn analyze(_args: &crate::Args) {
    let mut vulns: HashMap<u32, Vulnerability> = HashMap::new();

    let config = load_payloads();

    for payload_config in &config.payloads {
        let files = std::fs::read_dir("data/").expect("Could not open 'data/'");
        let mut results: Vec<FuzzingResult> = Vec::new();

        for file in files {
            if let Ok(f) = file {
                let file_name: String = f.file_name().to_str().unwrap().to_string();
                let split: Vec<&str> = file_name.splitn(2, '-').collect();

                if split.len() != 2 {
                    eprintln!("Malformed filename '{}'", file_name);
                    continue;
                }

                let file_parser_name = &split[0];
                let file_config_name = &split[1][0..split[1].len() - 4];

                if file_config_name != payload_config.name {
                    continue;
                }

                let bytes = std::fs::read(f.path()).expect("Could not read file");

                // TODO - remove
                // if bytes.len() >= 10_000_000 {
                //     // println!("Skipping {} due to size", file_name);
                //     continue;
                // }

                let result = FuzzingResult {
                    parser_name: file_parser_name.to_string(),
                    decoder: Decoder::new(Box::new(bytes)),
                };

                results.push(result);
            }
        }

        results.sort_by(|a, b| a.parser_name.cmp(&b.parser_name));

        for (hash, vuln) in analyze_results(payload_config.clone(), &mut results) {
            match vulns.get(&hash) {
                Some(best_vuln) => {
                    if vuln.payload.len() < best_vuln.payload.len() {
                        vulns.insert(hash, vuln);
                    }
                }
                None => {
                    vulns.insert(hash, vuln);
                }
            }
        }
    }

    let mut vulns_vec: Vec<&Vulnerability> = vulns.iter().map(|(_, v)| v).collect();

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
