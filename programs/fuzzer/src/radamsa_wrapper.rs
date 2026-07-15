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

        // djb2 hashing algorithm
        let mut seed: u32 = 5381;

        for byte in test_case.json.bytes() {
            seed = ((seed << 5) + seed) + byte as u32;
        }

        let json = match String::from_utf8(decode_str(&test_case.json)) {
            Ok(j) => j,
            Err(_) => return Err(()),
        };

        Ok(Radamsa {
            seed: seed as usize,
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

        let test_case =
            &cmd.stdout[0..std::cmp::min((1 << 15) - 1, cmd.stdout.len())].trim_ascii_end();

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

// 3.3s in total
#[cfg(test)]
mod tests {
    use super::*;

    // 0.29s
    #[test]
    fn performance1() {
        let testcase = TestCase::new(r#"{"q":2,"q":3}"#.into(), "q".into(), None);
        let mut fuzzer = Radamsa::new(&testcase).unwrap();
        let mut buf = vec![0u8; 1 << 16];

        fuzzer.n = 10000;
        let n = fuzzer.copy_to_slice(&mut buf).unwrap();
        let mut bytes = n;

        while fuzzer.advance().is_ok() {
            let n = fuzzer.copy_to_slice(&mut buf).unwrap();
            bytes += n;
        }
        panic!("{}", bytes);
    }
}
