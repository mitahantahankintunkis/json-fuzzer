use crate::compression::*;
use crate::payload::*;
use std::collections::HashSet;
use std::ops::Range;
// use std::hash::{Hash, Hasher};

struct FuzzingResult {
    parser_name: String,
    decoder: Decoder,
}

// struct Vulnerability {
//     payload: String,
//     parser_0_name: String,
//     parser_1_name: String,
//     parser_0_output: String,
//     parser_1_output: String,
// }

// impl Hash for Vulnerability {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.parser_0_name.hash(state);
//         self.parser_1_name.hash(state);
//         self.parser_0_output.hash(state);
//         self.parser_1_output.hash(state);
//     }
// }

fn analyze_results(fuzzing_type: &FuzzingType, results: &mut Vec<FuzzingResult>) {
    if results.len() == 0 {
        return;
    }

    print!("{:20}\t", "payload");

    let mut base_payload: &str = "";

    for result in &mut *results {
        print!("{:20}\t", result.parser_name);
        base_payload = result.decoder.next_message().unwrap();
    }

    println!("");

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

    let mut analyzed: HashSet<String> = HashSet::new();
    // let mut vulnerabilities: HashSet<Vulnerability> = HashSet::new();

    // let mut messages: usize = 0;

    loop {
        payload.next();
        let mut parser_output: Vec<String> = Vec::new();
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

        if !equal {
            let buffer = payload.get_payload();
            let mut payload_str = String::with_capacity(buffer.len());

            for i in 0..buffer.len() {
                match buffer[i] {
                    0u8..0x20 => payload_str.push_str(format!("\\x{:02x}", buffer[i]).as_str()),
                    0x7fu8..=0xff => payload_str.push_str(format!("\\x{:02x}", buffer[i]).as_str()),
                    b => payload_str.push(b as char),
                }
            }

            if !analyzed.contains(&payload_str) {
                print!("{:20}\t", payload_str);

                for output in &parser_output {
                    print!("{:13}\t", output);
                }
                println!("");
                analyzed.insert(payload_str);
            }
        }
    }
}

pub fn analyze(args: &crate::Args) {
    let fuzzing_types = &vec![
        FuzzingType::ReplaceOneByte,
        FuzzingType::ReplaceTwoBytes,
        FuzzingType::ReplaceThreeBytes,
        FuzzingType::InsertOneByte,
        FuzzingType::InsertTwoBytes,
        FuzzingType::InsertThreeBytes,
        FuzzingType::ReplaceOneUnicodeByte,
        FuzzingType::ReplaceTwoUnicodeBytes,
        FuzzingType::ReplaceThreeUnicodeBytes,
        FuzzingType::InsertOneUnicodeByte,
        FuzzingType::InsertTwoUnicodeBytes,
        FuzzingType::InsertThreeUnicodeBytes,
    ];

    let digest = md5::compute(args.payload.as_bytes());
    let hash: String = format!("{:x}", digest);

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

                // println!("{}", result.parser_name);

                results.push(result);
            }
        }

        results.sort_by(|a, b| a.parser_name.cmp(&b.parser_name));
        analyze_results(fuzzing_type, &mut results);

        // if !fs::exists("../data/").expect("Could not check if directory exists") {
        //     fs::create_dir("../data").expect("Could not create directory");
        // }
        //
        // let file_name: String = format!("../data/{}_{}.bin", client_name, fuzzing_type);
        //
        // if fs::exists(&file_name).expect("Could not check if file exists") {
        //     println!("'{}' already exists. Skipping.", file_name);
        //     continue;
        // }
        //
        // let mut payload: Box<dyn Payload> = match fuzzing_type {
        //     FuzzingType::ReplaceOneByte => Box::new(ReplaceBytes::new(&base_payload, 1)),
        //     FuzzingType::ReplaceTwoBytes => Box::new(ReplaceBytes::new(&base_payload, 2)),
        //     FuzzingType::ReplaceThreeBytes => Box::new(ReplaceBytes::new(&base_payload, 3)),
        // };

        // let mut finished: bool = false;
        //
        // loop {
        //     if finished {
        //         break;
        //     }
        //
        //     if !payload.next() {
        //         finished = true;
        //     }
        // }

        // let start = std::time::Instant::now();
        // let compressed = fuzz(stream, &args, &mut payload);
        //
        // println!(
        //     "{:20} {:25}  n: {:12}k, {:7.1}k/s  time: {}s  compression: {:.1}kb, {:.1}%",
        //     client_name,
        //     fuzzing_type.to_string(),
        //     compressed.message_count / 1000,
        //     (compressed.message_count as f64) / (start.elapsed().as_millis() as f64),
        //     start.elapsed().as_secs(),
        //     compressed.bytes.len() as f64 / 1000.0,
        //     compressed.bytes.len() as f64 / compressed.uncompressed_bytes as f64 * 100.0,
        // );
        //
        // let mut file = fs::File::create(&file_name.as_str()).expect("Could not create file");
        // file.write_all(&compressed.bytes)
        //     .expect("Could not write to file");
    }
}
