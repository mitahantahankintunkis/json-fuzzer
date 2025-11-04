use std::{process::Command, sync::Arc};

use crate::{
    fuzz::{Fuzzer, TestCase},
    util::decode_str,
};

pub struct Radamsa {
    store: Arc<Box<Vec<Vec<u8>>>>,
    seed: usize,
    json: String,
    n: usize,
    i: usize,
}

impl Radamsa {
    pub fn new(test_case: &TestCase, parent: Option<Radamsa>) -> Self {
        // let n = 1000_000;
        let n = 100;
        let seed: usize = 905391675;

        match parent {
            Some(p) => Radamsa {
                store: p.store.clone(),
                seed,
                json: String::from_utf8_lossy(&decode_str(&test_case.json)).to_string(),
                n,
                i: 0,
            },
            None => {
                let b = Box::new(Vec::with_capacity(n));

                Radamsa {
                    store: Arc::new(b),
                    seed,
                    json: String::from_utf8_lossy(&decode_str(&test_case.json)).to_string(),
                    n,
                    i: 0,
                }
            }
        }
    }
}

impl Fuzzer for Radamsa {
    fn advance(&mut self) -> Result<(), ()> {
        if self.i >= self.n - 1 {
            Err(())
        } else {
            self.i += 1;
            Ok(())
        }
    }

    fn copy_to_slice(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let cmd = Command::new("sh")
            .args(&[
                "-c",
                &format!(
                    "echo '{}' | radamsa --seed {} --truncate 65535",
                    self.json.replace("'", "\\'"),
                    self.seed + self.i,
                ),
            ])
            .output()
            .expect(&format!("Could not execute radamsa {}", self.json));

        let test_case =
            &cmd.stdout[0..std::cmp::min((1 << 16) - 1, cmd.stdout.len())].trim_ascii_end();

        if test_case.len() > buf.len() {
            return Err(());
        }

        for (i, b) in test_case.iter().enumerate() {
            buf[i] = *b;
        }

        Ok(test_case.len())
    }

    fn id(&self) -> String {
        "radamsa".to_string()
    }
}
