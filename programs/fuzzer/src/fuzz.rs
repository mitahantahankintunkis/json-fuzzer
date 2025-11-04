use std::fmt::Display;
#[allow(unused)]
use std::{
    cmp::{max, min, Ordering},
    fs,
    ops::Range,
};

use crate::{comprehensive_fuzzer::ComprehensiveFuzzer, radamsa_wrapper::Radamsa};

pub trait Fuzzer {
    fn advance(&mut self) -> Result<(), ()>;
    fn copy_to_slice(&mut self, buf: &mut [u8]) -> Result<usize, ()>;
    fn id(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub weight: f64,
    pub parent_json: Option<String>,
    pub depth: usize,
    pub json: String,
    pub key: String,
}

impl TestCase {
    pub fn new(json: String, key: String, parent: Option<TestCase>) -> Self {
        let mut parent_json = None;
        let mut depth = 0;

        if let Some(parent) = parent {
            parent_json = Some(parent.json.clone());
            depth = parent.depth + 1;
        }

        TestCase {
            weight: 0.0,
            parent_json,
            depth,
            json,
            key,
        }
    }
}

impl Display for TestCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({} {})  p: {}  w: {}",
            self.json,
            self.key,
            self.parent_json.clone().unwrap_or("None".to_string()),
            self.weight
        )
    }
}

#[derive(Default, Debug, Clone, clap::ValueEnum)]
pub enum Fuzzers {
    #[default]
    Comprehensive,
    Radamsa,
}

pub fn create_fuzzers(testcase: &TestCase) -> Vec<Box<dyn Fuzzer>> {
    let mut ret: Vec<Box<dyn Fuzzer>> = vec![
        Box::new(ComprehensiveFuzzer::insert_single(testcase)),
        Box::new(ComprehensiveFuzzer::remove_single(testcase)),
        Box::new(ComprehensiveFuzzer::insert_unicode(testcase)),
        Box::new(ComprehensiveFuzzer::replace_unicode(testcase)),
    ];

    if testcase.json.len() < 30 {
        ret.push(Box::new(ComprehensiveFuzzer::insert_two(testcase)));
        ret.push(Box::new(ComprehensiveFuzzer::remove_two(testcase)));
    }

    ret.push(Box::new(Radamsa::new(testcase, None)));
    ret
}

pub fn load_testcases() -> Vec<TestCase> {
    let csv = fs::read_to_string("payloads.csv").expect("Could not read 'payloads.csv'");
    let mut ret: Vec<TestCase> = Vec::new();

    for (_i, line) in csv.lines().enumerate() {
        let mut spl = line.splitn(2, "\t");
        let json = spl.next();
        let key = spl.next();

        if json.is_none() || key.is_none() {
            continue;
        }

        let json = json.unwrap();
        let key = key.unwrap();
        // let digest = format!("{:x}", md5::compute(&json))[0..16].to_string();

        ret.push(TestCase {
            weight: 0.0,
            parent_json: None,
            depth: 0,
            json: json.to_string(),
            key: key.to_string(),
        });
    }

    ret
}
