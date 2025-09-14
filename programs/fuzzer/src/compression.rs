pub struct Encoder {
    pub bytes: Box<Vec<u8>>,
    pub prev_added: Option<Vec<u8>>,
    pub prev_count: u32,
    pub uncompressed_bytes: u64,
    pub message_count: u64,
}

#[derive(Clone)]
pub struct Decoder {
    pub bytes: Box<Vec<u8>>,
    pub messages_parsed: usize,
    message_index: usize,
    cur_repeat: u32,
}

impl Encoder {
    pub fn new() -> Encoder {
        Encoder {
            bytes: Box::new(Vec::with_capacity(1_000_000)),
            prev_added: None,
            prev_count: 0,
            uncompressed_bytes: 0,
            message_count: 0,
        }
    }

    fn write_prev(&mut self) {
        if let Some(prev_added) = &self.prev_added {
            // Repeats
            for byte in self.prev_count.to_le_bytes() {
                self.bytes.push(byte);
            }

            // Size
            let length: u32 = prev_added
                .len()
                .try_into()
                .expect("Integer overflow in compression");

            for byte in length.to_le_bytes() {
                self.bytes.push(byte);
            }

            // Payload
            for byte in prev_added {
                self.bytes.push(*byte);
            }
        }

        self.prev_count = 0;
        self.prev_added = None;
    }

    pub fn add_bytes(&mut self, bytes: &[u8]) {
        self.message_count += 1;
        self.uncompressed_bytes += bytes.len() as u64;

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
        };

        self.prev_count += 1;

        if self.prev_count == u32::MAX {
            self.write_prev();
        }
    }

    pub fn finish(&mut self) -> &Box<Vec<u8>> {
        if self.prev_added.is_some() {
            self.write_prev();
        }

        return &self.bytes;
    }
}

#[derive(Default)]
pub struct DecoderState {
    pub message_index: usize,
    pub cur_repeat: u32,
}

impl Decoder {
    pub fn new(bytes: Box<Vec<u8>>) -> Decoder {
        Decoder {
            bytes,
            messages_parsed: 0,
            message_index: 0,
            cur_repeat: 0,
        }
    }

    pub fn next_message_with_state(&self, state: &mut DecoderState) -> Option<&str> {
        if state.message_index >= self.bytes.len() {
            return None;
        }

        let repeat_bytes = &self.bytes[state.message_index..(state.message_index + 4)];

        // ~30% faster with unsafe functions
        unsafe {
            let repeat = u32::from_le_bytes(repeat_bytes.try_into().unwrap_unchecked());

            let length_bytes = &self.bytes[(state.message_index + 4)..(state.message_index + 8)];
            let length: usize =
                u32::from_le_bytes(length_bytes.try_into().unwrap_unchecked()) as usize;

            let data = &self.bytes[(state.message_index + 8)..(state.message_index + 8 + length)];

            let str = std::str::from_utf8_unchecked(&data);

            state.cur_repeat += 1;

            if state.cur_repeat >= repeat {
                state.cur_repeat = 0;
                state.message_index += length + 8;
            }
            return Some(str);
        }
    }

    #[allow(dead_code)]
    pub fn next_message(&mut self) -> Option<&str> {
        if self.message_index >= self.bytes.len() {
            return None;
        }

        let repeat_bytes = &self.bytes[self.message_index..(self.message_index + 4)];

        // ~30% faster with unsafe functions
        unsafe {
            let repeat = u32::from_le_bytes(repeat_bytes.try_into().unwrap_unchecked());

            let length_bytes = &self.bytes[(self.message_index + 4)..(self.message_index + 8)];
            let length: usize =
                u32::from_le_bytes(length_bytes.try_into().unwrap_unchecked()) as usize;

            let data = &self.bytes[(self.message_index + 8)..(self.message_index + 8 + length)];

            let str = std::str::from_utf8_unchecked(&data);

            self.cur_repeat += 1;
            self.messages_parsed += 1;

            if self.cur_repeat >= repeat {
                self.cur_repeat = 0;
                self.message_index += length + 8;
            }
            return Some(str);
        }
    }

    #[allow(dead_code)]
    pub fn cur_message(&self) -> Option<&str> {
        if self.message_index >= self.bytes.len() {
            return None;
        }

        // ~30% faster with unsafe functions
        unsafe {
            let length_bytes = &self.bytes[(self.message_index + 4)..(self.message_index + 8)];
            let length: usize =
                u32::from_le_bytes(length_bytes.try_into().unwrap_unchecked()) as usize;

            let data = &self.bytes[(self.message_index + 8)..(self.message_index + 8 + length)];

            let str = std::str::from_utf8_unchecked(&data);

            return Some(str);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let enc = Encoder::new();
        let mut dec = Decoder::new(Box::new(*enc.bytes));
        assert!(dec.next_message() == None);
    }

    #[test]
    fn test_single() {
        let data: String = String::from("hello");
        let mut enc = Encoder::new();
        enc.add_bytes(data.as_bytes());
        enc.finish();

        let expected: Vec<u8> = [vec![1u8, 0, 0, 0, 5, 0, 0, 0], data.as_bytes().to_vec()].concat();

        println!("enc: {:02x?}  expected: {:02x?}", enc.bytes, expected);
        assert!(enc.bytes.iter().zip(expected).all(|(a, b)| *a == b));

        let mut dec = Decoder::new(Box::new(*enc.bytes));

        if let Some(m) = dec.next_message() {
            println!("next: {}  expected: {}", m, data);
            assert!(m.to_string() == data);
        }

        assert!(dec.next_message() == None);
    }

    #[test]
    fn test_multiple() {
        let data: &[String] = &[
            String::from("hello"),
            String::from("hello"),
            String::from("hello"),
            String::from("sup"),
            String::from("hello"),
            String::from("hello"),
            String::from("sup"),
            String::from("sup"),
            String::from("sup"),
            String::from("hello"),
            String::from("hi"),
            String::from("sup"),
            String::from("hello"),
        ];

        let mut enc = Encoder::new();

        for s in data {
            enc.add_bytes(s.as_bytes());
        }

        enc.finish();

        let expected: Vec<u8> = [
            vec![3, 0, 0, 0, 5, 0, 0, 0],
            data[0].as_bytes().to_vec(),
            vec![1, 0, 0, 0, 3, 0, 0, 0],
            data[3].as_bytes().to_vec(),
            vec![2, 0, 0, 0, 5, 0, 0, 0],
            data[4].as_bytes().to_vec(),
            vec![3, 0, 0, 0, 3, 0, 0, 0],
            data[6].as_bytes().to_vec(),
            vec![1, 0, 0, 0, 5, 0, 0, 0],
            data[9].as_bytes().to_vec(),
            vec![1, 0, 0, 0, 2, 0, 0, 0],
            data[10].as_bytes().to_vec(),
            vec![1, 0, 0, 0, 3, 0, 0, 0],
            data[11].as_bytes().to_vec(),
            vec![1, 0, 0, 0, 5, 0, 0, 0],
            data[12].as_bytes().to_vec(),
        ]
        .concat();
        // vec![vec![1u8, 0, 0, 0, 5, 0, 0, 0], data.as_bytes().to_vec()].concat();

        println!("enc:      {:02x?}\nexpected: {:02x?}", enc.bytes, expected);
        assert!(enc.bytes.iter().zip(expected).all(|(a, b)| *a == b));

        let mut dec = Decoder::new(Box::new(*enc.bytes));
        let mut i = 0;

        while let Some(m) = dec.next_message() {
            assert!(i < data.len());
            println!("next: {}  expected: {}", m, data[i]);
            assert!(m.to_string() == data[i]);
            i += 1;
        }

        assert!(i == data.len());
        assert!(dec.next_message() == None);
    }
}
