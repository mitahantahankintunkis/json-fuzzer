pub struct Encoder {
    pub bytes: Box<Vec<u8>>,
    pub prev_added: Option<Vec<u8>>,
    pub prev_count: u32,
    pub uncompressed_bytes: usize,
}

impl Encoder {
    pub fn new() -> Encoder {
        Encoder {
            bytes: Box::new(Vec::with_capacity(1_000_000)),
            prev_added: None,
            prev_count: 0,
            uncompressed_bytes: 0,
        }
    }

    fn write_prev(&mut self) {
        if let Some(prev_added) = &self.prev_added {
            self.bytes.push(0xff);
            self.bytes.push(0xff);
            self.bytes.push(0xee);
            for byte in self.prev_count.to_le_bytes() {
                self.bytes.push(byte);
            }
            self.bytes.push(0xee);
            self.bytes.push(0xff);
            self.bytes.push(0xff);

            for byte in prev_added {
                self.bytes.push(*byte);
            }
        }

        self.prev_count = 0;
        self.prev_added = None;
    }

    pub fn add_bytes(&mut self, bytes: &[u8]) {
        match &self.prev_added {
            Some(prev_added) => {
                let equal = bytes.len() == prev_added.len()
                    && prev_added.iter().zip(bytes).all(|(a, b)| a == b);

                if !equal {
                    self.write_prev();
                    self.prev_added = Some(bytes.to_vec());
                }
            }
            None => {
                self.prev_added = Some(bytes.to_vec());
            }
        }

        self.uncompressed_bytes += bytes.len();
        self.prev_count += 1;

        if self.prev_count == u32::MAX {
            self.write_prev();
        }
    }
}
