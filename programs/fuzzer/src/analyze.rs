#![allow(unused)]
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use rusqlite::Connection;

use crate::compression::*;
use crate::fuzz::*;
use crate::server::Job;
use crate::server::ParsingResult;
use crate::util::byte_to_string;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::thread::spawn;
use std::thread::JoinHandle;

struct FuzzingResult {
    parser_name: String,
    decoder: Decoder,
}

#[derive(Clone, Debug)]
struct Vulnerability {
    testcase: TestCase,
    parser_0_name: String,
    parser_1_name: String,
    parser_0_output: String,
    parser_1_output: String,
    hash: u32,
}

fn vuln_hash(
    name0: &str,
    name1: &str,
    out0: &str,
    out1: &str,
    testcase: &TestCase,
    fuzzer_name: &str,
) -> u32 {
    // let mut hasher = DefaultHasher::new();
    // name0.hash(&mut hasher);
    // name1.hash(&mut hasher);
    // out0.hash(&mut hasher);
    // out1.hash(&mut hasher);
    // hasher.finish()

    // djb2 hashing algorithm
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

    for byte in testcase.json.bytes() {
        hash = ((hash << 5) + hash) + byte as u32;
    }

    for byte in fuzzer_name.bytes() {
        hash = ((hash << 5) + hash) + byte as u32;
    }

    hash
}

impl Vulnerability {
    fn new(
        name0: &str,
        name1: &str,
        out0: &str,
        out1: &str,
        testcase: TestCase,
        fuzzer_name: &str,
    ) -> Self {
        Vulnerability {
            parser_0_name: name0.to_string(),
            parser_1_name: name1.to_string(),
            parser_0_output: out0.to_string(),
            parser_1_output: out1.to_string(),
            hash: vuln_hash(name0, name1, out0, out1, &testcase, fuzzer_name),
            testcase,
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
    testcase: &TestCase,
    fuzzer: &mut Box<dyn Fuzzer>,
    results: &Vec<FuzzingResult>,
) -> (Vec<(u32, Vulnerability)>, usize) {
    if results.len() == 0 {
        return (Vec::new(), 0);
    }

    let mut vulnerabilities: Vec<(u32, Vulnerability)> = Vec::new();

    if results.len() == 0 {
        panic!("No results for {}", fuzzer.id());
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

    let mut total_bytes = 0;
    let mut errs: HashSet<String> = HashSet::new();
    let mut fuzzed = vec![0u8; 1 << 16];

    loop {
        parser_outputs.clear();
        payload_str.clear();

        for (i, result) in results.iter().enumerate() {
            let res = result
                .decoder
                .next_message_with_state(&mut decoder_states[i]);

            match res {
                Some(r) => {
                    total_bytes += r.len();
                    if r != "PARSE_ERROR" && r != "KEY_NOT_FOUND" {
                        parser_outputs.push((&result.parser_name, r));
                    }
                }
                None => {
                    let key = format!("{} {} {}", result.parser_name, fuzzer.id(), testcase.json);
                    if !errs.contains(&key) {
                        let n = fuzzer.copy_to_slice(&mut fuzzed).unwrap();
                        for byte in &fuzzed[0..n] {
                            payload_str.push_str(&byte_to_string(*byte));
                        }

                        errs.insert(key);
                        eprintln!(
                            "No next message for {} {} {} {}",
                            result.parser_name,
                            fuzzer.id(),
                            testcase.json,
                            fuzzed[0..n]
                                .iter()
                                .map(|c| byte_to_string(*c))
                                .collect::<Vec<String>>()
                                .join(""),
                        );
                    }
                }
            }
        }

        if parser_outputs.len() == 0 {
            if fuzzer.advance().is_err() {
                break;
            }

            continue;
        }

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
            if fuzzer.advance().is_err() {
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
                    let n = fuzzer.copy_to_slice(&mut fuzzed).unwrap();
                    for byte in &fuzzed[0..n] {
                        payload_str.push_str(&byte_to_string(*byte));
                    }
                }

                let testcase = TestCase::new(
                    payload_str.clone(),
                    testcase.key.clone(),
                    Some(testcase.clone()),
                );

                let hash = vuln_hash(&name0, &name1, output0, output1, &testcase, &fuzzer.id());
                let mut found = false;

                for i in 0..vulnerabilities.len() {
                    let best_vuln = &vulnerabilities[i];

                    if hash == best_vuln.0 {
                        if payload_str.len() < best_vuln.1.testcase.json.len() {
                            let vuln = Vulnerability::new(
                                &name0,
                                &name1,
                                output0,
                                output1,
                                testcase.clone(),
                                &fuzzer.id(),
                            );
                            vulnerabilities[i] = (hash, vuln);
                        }

                        found = true;
                        break;
                    }
                }

                if !found {
                    let vuln = Vulnerability::new(
                        &name0,
                        &name1,
                        output0,
                        output1,
                        testcase.clone(),
                        &fuzzer.id(),
                    );
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

        if fuzzer.advance().is_err() {
            break;
        }
    }

    (vulnerabilities, total_bytes)
}

fn analyze(res: ParsingResult, vuln_mat: &mut HashSet<(String, String)>) -> Vec<Vulnerability> {
    let mut vulns: HashMap<u32, Vulnerability> = HashMap::new();
    let mut total_bytes = 0;

    let digest = format!("{:x}", md5::compute(&res.testcase.json))[0..8].to_string();

    for mut fuzzer in &mut create_fuzzers(&res.testcase) {
        let files = std::fs::read_dir("data/").expect("Could not open 'data/'");
        let mut results: Vec<FuzzingResult> = Vec::new();

        for file in files {
            if let Ok(f) = file {
                let file_name: String = f.file_name().to_str().unwrap().to_string();
                let split: Vec<&str> = file_name.split(';').collect();

                if split.len() != 4 {
                    eprintln!("Malformed filename '{}'", file_name);
                    continue;
                }

                let file_parser_name = &split[0];
                let file_fuzzer_name = &split[1];
                let file_json_id = &split[2];

                if *file_fuzzer_name != fuzzer.id() || *file_json_id != digest {
                    continue;
                }

                let bytes = std::fs::read(f.path()).expect("Could not read file");

                let result = FuzzingResult {
                    parser_name: file_parser_name.to_string(),
                    decoder: Decoder::new(Box::new(bytes)),
                };

                results.push(result);
            }
        }

        results.sort_by(|a, b| a.parser_name.cmp(&b.parser_name));
        let (res, n) = analyze_results(&res.testcase, &mut fuzzer, &mut results);
        total_bytes += n;

        for (hash, vuln) in res {
            match vulns.get(&hash) {
                Some(best_vuln) => {
                    if vuln.testcase.json.len() < best_vuln.testcase.json.len() {
                        vulns.insert(hash, vuln);
                    }
                }
                None => {
                    vulns.insert(hash, vuln);
                }
            }
        }
    }

    // eprintln!(
    //     "Analyzed {} gigabytes of parsing results",
    //     total_bytes / 1000_000_000
    // );

    let mut vulns_vec: Vec<Vulnerability> = vulns.iter().map(|(_, v)| v.clone()).collect();

    vulns_vec.sort_by_key(|v| {
        (
            v.parser_0_name.clone(),
            v.parser_1_name.clone(),
            v.parser_0_output.clone(),
            v.parser_1_output.clone(),
        )
    });

    vulns_vec
}

pub struct Analyzer {
    pub handle: JoinHandle<()>,
}

impl Analyzer {
    pub fn new(rx: Receiver<ParsingResult>, tx: Sender<Job>) -> Self {
        let db_conn = Connection::open("analyzed/db.sqlite").unwrap();
        Analyzer::init_db(&db_conn);

        let handle = spawn(move || {
            let mut vuln_mat: HashSet<(String, String)> = HashSet::new();

            while let Ok(res) = rx.recv() {
                let micros = match res.times.last() {
                    Some(t) => t.duration,
                    None => 0,
                };

                println!(
                    "Analyzer received: {} from {} ({}us - {}us)",
                    res.testcase,
                    res.client_name,
                    micros,
                    match res.times.first() {
                        Some(t) => t.duration,
                        None => 0,
                    }
                );

                if let Some(last) = res.times.last() {
                    let mut testcase = last.testcase.clone();
                    testcase.weight = 1.0 / (last.duration as f64) + testcase.depth as f64;

                    if last.duration > 0 && testcase.depth < 10 {
                        tx.send(Job {
                            testcase: testcase,
                            clients: vec![res.client_name.clone()],
                        })
                        .unwrap();
                    }
                }

                let vulns = analyze(res, &mut vuln_mat);

                for vuln in &vulns {
                    vuln_mat.insert((vuln.parser_0_name.clone(), vuln.parser_1_name.clone()));
                    vuln_mat.insert((vuln.parser_1_name.clone(), vuln.parser_0_name.clone()));
                    db_conn.execute(
                        "INSERT INTO results VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        (
                            &vuln.parser_0_name,
                            &vuln.parser_1_name,
                            &vuln.testcase.json,
                            &vuln.testcase.key,
                            &vuln.parser_0_output,
                            &vuln.parser_1_output,
                        ),
                    );
                    db_conn.execute(
                        "INSERT INTO results VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        (
                            &vuln.parser_1_name,
                            &vuln.parser_0_name,
                            &vuln.testcase.json,
                            &vuln.testcase.key,
                            &vuln.parser_1_output,
                            &vuln.parser_0_output,
                        ),
                    );
                }

                for vuln in vulns {
                    tx.send(Job {
                        testcase: vuln.testcase.clone(),
                        clients: vec![],
                    })
                    .unwrap();
                    break;
                }
            }
        });

        Analyzer { handle }
    }

    fn init_db(conn: &Connection) {
        if !conn.table_exists(None, "results").unwrap() {
            conn.execute(
                "CREATE TABLE results (
                        client0 TEXT NOT NULL,
                        client1 TEXT NOT NULL,
                        json TEXT NOT NULL,
                        key TEXT NOT NULL,
                        output0 TEXT NOT NULL,
                        output1 TEXT NOT NULL
                    )",
                (),
            )
            .unwrap();
        }

        if !conn.table_exists(None, "testcases").unwrap() {
            conn.execute(
                "CREATE TABLE testcases (
                        json TEXT NOT NULL,
                        key TEXT NOT NULL,
                        weight REAL NOT NULL,
                        depth INTEGER NOT NULL,
                        parent INTEGER,
                        FOREIGN KEY (parent) REFERENCES testcases(rowid)
                    )",
                (),
            )
            .unwrap();
        }

        if !conn.table_exists(None, "parsing_times").unwrap() {
            conn.execute(
                "CREATE TABLE parsing_times (
                        client TEXT NOT NULL,
                        time REAL NOT NULL,
                        testcase INTEGER NOT NULL,
                        FOREIGN KEY (testcase) REFERENCES testcases(rowid)
                    )",
                (),
            )
            .unwrap();
        }
    }
}
