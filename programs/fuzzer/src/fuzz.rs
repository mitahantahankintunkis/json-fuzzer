use std::fmt::Display;

use crate::{okfuzz::OKFuzz, radamsa_wrapper::Radamsa};

pub trait Fuzzer {
    fn advance(&mut self) -> Result<(), ()>;
    fn copy_to_slice(&self, buf: &mut [u8]) -> Result<usize, ()>;
    fn id(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub id: isize,
    pub json: String,
    pub key: String,
    pub weight: f64,
    pub depth: usize,
    pub parent_id: Option<isize>,
    pub parser: Option<String>,
}

impl PartialEq for TestCase {
    fn eq(&self, other: &Self) -> bool {
        return self.weight == other.weight
            && self.depth == other.depth
            && self.json == other.json
            && self.key == other.key
            && self.parser.eq(&other.parser);
    }
}

impl Eq for TestCase {}

impl Ord for TestCase {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.weight
            .total_cmp(&other.weight)
            .then_with(|| self.depth.cmp(&other.depth))
            .then_with(|| self.json.cmp(&other.json))
            .then_with(|| self.key.cmp(&other.key))
    }
}

impl PartialOrd for TestCase {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl TestCase {
    pub fn new(mut json: String, key: String, parent: Option<TestCase>) -> Self {
        let mut parent_id = None;
        let mut depth = 0;

        if let Some(parent) = parent {
            parent_id = Some(parent.id);
            depth = parent.depth + 1;
        }

        json.truncate(u16::MAX as usize - 256);

        TestCase {
            id: -1,
            weight: 0.0,
            parent_id,
            depth,
            json,
            key,
            parser: None,
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
            self.parent_id.unwrap_or(-1).to_string(),
            self.weight
        )
    }
}

pub fn create_fuzzers(testcase: &TestCase) -> Vec<Box<dyn Fuzzer>> {
    let mut ret: Vec<Box<dyn Fuzzer>> = Vec::new();

    if testcase.json.len() < 30 {
        ret.push(Box::new(OKFuzz::insert_grammar(testcase)));
        ret.push(Box::new(OKFuzz::insert_single(testcase)));
        ret.push(Box::new(OKFuzz::remove_single(testcase)));
    }

    if testcase.depth == 0 {
        ret.push(Box::new(OKFuzz::insert_unicode(testcase)));
        ret.push(Box::new(OKFuzz::replace_unicode(testcase)));
        ret.push(Box::new(OKFuzz::insert_single_word(testcase)));
        ret.push(Box::new(OKFuzz::replace_single_word(testcase)));
    }

    if let Ok(r) = Radamsa::new(testcase) {
        ret.push(Box::new(r));
    }

    ret
}
