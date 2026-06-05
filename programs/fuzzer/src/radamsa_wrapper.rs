use std::process::Command;

use crate::{
    fuzz::{Fuzzer, TestCase},
    util::decode_str,
};

pub struct Radamsa {
    seed: usize,
    json: String,
    n: usize,
    i: usize,
}

impl Radamsa {
    pub fn new(test_case: &TestCase) -> Result<Self, ()> {
        let n = 100;
        let seed: usize = 905391675;
        let json = match String::from_utf8(decode_str(&test_case.json)) {
            Ok(j) => j,
            Err(_) => return Err(()),
        };

        Ok(Radamsa {
            seed,
            json,
            n,
            i: 0,
        })
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

    fn copy_to_slice(&self, buf: &mut [u8]) -> Result<usize, ()> {
        let cmd = match Command::new("sh")
            .args(&[
                "-c",
                &format!(
                    "echo '{}' | radamsa --seed {} --truncate 32767",
                    self.json.replace("'", "\\'"),
                    self.seed + self.i,
                ),
            ])
            .output()
        {
            Ok(c) => c,
            Err(_) => return Ok(0),
        };
        // .expect(&format!("Could not execute radamsa {}", self.json));

        let test_case =
            &cmd.stdout[0..std::cmp::min((1 << 15) - 1, cmd.stdout.len())].trim_ascii_end();

        // eprintln!(
        //     "radamsa: {:?}",
        //     String::from_utf8_lossy(test_case).to_string()
        // );

        if test_case.len() > buf.len() {
            return Err(());
            // panic!("Radamsa too large");
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
