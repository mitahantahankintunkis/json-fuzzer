use std::fmt::Display;

use crate::{comprehensive_fuzzer::ComprehensiveFuzzer, radamsa_wrapper::Radamsa};

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

    // pub fn update_id(&mut self, db_conn: &Connection) {
    //     if self.id >= 0 {
    //         return;
    //     }
    //
    //     // Find existing
    //     let id = db_conn.query_one(
    //         "SELECT id FROM corpus
    //             WHERE json = ?1 AND key = ?2
    //             LIMIT 1",
    //         params![&self.json, &self.key,],
    //         |row| {
    //             let id: isize = row.get(0)?;
    //             Ok(id)
    //         },
    //     );
    //
    //     if let Ok(id) = id {
    //         self.id = id;
    //         return;
    //     }
    //
    //     // json TEXT NOT NULL,
    //     // key TEXT NOT NULL,
    //     // weight REAL NOT NULL,
    //     // depth INTEGER NOT NULL,
    //     // parent INTEGER,
    //     // FOREIGN KEY (parent) REFERENCES corpus(rowid)
    //     db_conn
    //         .execute(
    //             "INSERT INTO corpus (json, key, weight, depth, parent)
    //             VALUES (?1, ?2, ?3, ?4, ?5)",
    //             params![
    //                 &self.json,
    //                 &self.key,
    //                 &self.weight.to_string(),
    //                 &self.depth.to_string(),
    //                 &self.parent_id,
    //             ],
    //         )
    //         .expect(&format!("Could not add testcase {:?} to corpus", self));
    //
    //     self.id = db_conn.last_insert_rowid() as isize;
    // }
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

#[derive(Default, Debug, Clone, clap::ValueEnum)]
pub enum Fuzzers {
    #[default]
    Comprehensive,
    Radamsa,
}

pub fn create_fuzzers(testcase: &TestCase) -> Vec<Box<dyn Fuzzer>> {
    let mut ret: Vec<Box<dyn Fuzzer>> = Vec::new();

    // if testcase.weight <= 1.0 {
    if testcase.json.len() < 30 {
        ret.push(Box::new(ComprehensiveFuzzer::insert_grammar(testcase)));
        ret.push(Box::new(ComprehensiveFuzzer::insert_single(testcase)));
        ret.push(Box::new(ComprehensiveFuzzer::remove_single(testcase)));
    }

    if testcase.depth == 0 {
        ret.push(Box::new(ComprehensiveFuzzer::insert_unicode(testcase)));
        ret.push(Box::new(ComprehensiveFuzzer::replace_unicode(testcase)));
        ret.push(Box::new(ComprehensiveFuzzer::insert_single_word(testcase)));
        ret.push(Box::new(ComprehensiveFuzzer::replace_single_word(testcase)));

        //     if testcase.json.len() < 30 {
        //         ret.push(Box::new(ComprehensiveFuzzer::insert_two(testcase)));
        //         ret.push(Box::new(ComprehensiveFuzzer::remove_two(testcase)));
        //     }
        //     ret.push(Box::new(Radamsa::new(testcase, Nne)));
    }

    if let Ok(r) = Radamsa::new(testcase) {
        ret.push(Box::new(r));
    }

    ret
}
