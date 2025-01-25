use clap::ValueEnum;
use std::ops::Range;

#[derive(ValueEnum, Debug, Clone)]
pub enum FuzzingType {
    ReplaceOneByte,
    ReplaceTwoBytes,
    ReplaceThreeBytes,
    InsertOneByte,
    InsertTwoBytes,
    InsertThreeBytes,
    ReplaceOneUnicodeByte,
    ReplaceTwoUnicodeBytes,
    ReplaceThreeUnicodeBytes,
    InsertOneUnicodeByte,
    InsertTwoUnicodeBytes,
    InsertThreeUnicodeBytes,
}

pub trait Payload {
    fn next(&mut self) -> bool;
    fn get_payload(&mut self) -> &Vec<u8>;
}

pub struct ReplaceBytes {
    pub data: Vec<u8>,
    pub buffer: Vec<u8>,
    //pub charset: Vec<u8>,
    pub indices: Vec<usize>,
    pub has_next: bool,
}

pub struct InsertBytes {
    // pub data: Vec<u8>,
    pub buffer: Vec<u8>,
    //pub charset: Vec<u8>,
    pub indices: Vec<usize>,
    pub has_next: bool,
}

pub struct ReplaceFormatted {
    pub data: Vec<u8>,
    pub buffer: Vec<u8>,
    pub charset: Range<usize>,
    pub indices: Vec<usize>,
    pub fuzzed: Vec<usize>,
    pub has_next: bool,
    format: fn(usize) -> String,
}

pub struct InsertFormatted {
    pub data: Vec<u8>,
    pub buffer: Vec<u8>,
    pub charset: Range<usize>,
    pub indices: Vec<usize>,
    pub fuzzed: Vec<usize>,
    pub has_next: bool,
    format: fn(usize) -> String,
}

// pub struct ReplaceUnicodeBytes {
//     pub data: Vec<u8>,
//     pub buffer: Vec<u8>,
//     //pub charset: Vec<u8>,
//     pub indices: Vec<usize>,
//     pub has_next: bool,
// }
//
// pub struct InsertUnicodeBytes {
//     // pub data: Vec<u8>,
//     pub buffer: Vec<u8>,
//     //pub charset: Vec<u8>,
//     pub indices: Vec<usize>,
//     pub has_next: bool,
// }

impl std::fmt::Display for FuzzingType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FuzzingType::ReplaceOneByte => write!(f, "replace-one-byte"),
            FuzzingType::ReplaceTwoBytes => write!(f, "replace-two-bytes"),
            FuzzingType::ReplaceThreeBytes => write!(f, "replace-three-bytes"),
            FuzzingType::InsertOneByte => write!(f, "insert-one-byte"),
            FuzzingType::InsertTwoBytes => write!(f, "insert-two-bytes"),
            FuzzingType::InsertThreeBytes => write!(f, "insert-three-bytes"),
            FuzzingType::ReplaceOneUnicodeByte => write!(f, "replace-one-unicode-byte"),
            FuzzingType::ReplaceTwoUnicodeBytes => write!(f, "replace-two-unicode-bytes"),
            FuzzingType::ReplaceThreeUnicodeBytes => write!(f, "replace-three-unicode-bytes"),
            FuzzingType::InsertOneUnicodeByte => write!(f, "insert-one-unicode-byte"),
            FuzzingType::InsertTwoUnicodeBytes => write!(f, "insert-two-unicode-bytes"),
            FuzzingType::InsertThreeUnicodeBytes => write!(f, "insert-three-unicode-bytes"),
        }
    }
}

impl ReplaceBytes {
    pub fn new(data: &[u8], bytes_to_replace: usize) -> ReplaceBytes {
        assert!(data.len() >= bytes_to_replace, "Payload too small");

        let mut ret = ReplaceBytes {
            data: data.to_vec(),
            buffer: data.to_vec(),
            //charset: Vec::from_iter(0u8..=255u8),
            indices: Vec::from_iter((0..bytes_to_replace).rev()),
            has_next: true,
        };

        for i in &ret.indices {
            ret.buffer[*i] = 0;
        }

        ret
    }
}

impl InsertBytes {
    pub fn new(data: &[u8], bytes_to_insert: usize) -> InsertBytes {
        assert!(data.len() >= bytes_to_insert, "Payload too small");

        InsertBytes {
            // data: data.to_vec(),
            buffer: [vec![0; bytes_to_insert], data.to_vec()].concat(),
            //charset: Vec::from_iter(0u8..=255u8),
            indices: Vec::from_iter((0..bytes_to_insert).rev()),
            has_next: true,
        }
    }
}

impl ReplaceFormatted {
    pub fn new(
        data: &[u8],
        bytes_to_replace: usize,
        charset: Range<usize>,
        format: fn(usize) -> String,
    ) -> ReplaceFormatted {
        assert!(data.len() >= bytes_to_replace, "Payload too small");

        let mut ret = ReplaceFormatted {
            data: data.to_vec(),
            buffer: [
                vec![0; format(0).as_bytes().len() * bytes_to_replace],
                data[bytes_to_replace..].to_vec(),
            ]
            .concat(),
            indices: Vec::from_iter((0..bytes_to_replace).rev()),
            fuzzed: vec![0; bytes_to_replace],
            has_next: true,
            charset,
            format,
        };

        for i in &ret.indices {
            ret.buffer[*i] = 0;
        }

        ret
    }
}

impl InsertFormatted {
    pub fn new(
        data: &[u8],
        bytes_to_insert: usize,
        charset: Range<usize>,
        format: fn(usize) -> String,
    ) -> InsertFormatted {
        assert!(data.len() >= bytes_to_insert, "Payload too small");

        InsertFormatted {
            data: data.to_vec(),
            buffer: [
                vec![0; format(0).as_bytes().len() * bytes_to_insert],
                data.to_vec(),
            ]
            .concat(),
            indices: vec![0; bytes_to_insert], //Vec::from_iter((0..bytes_to_insert).rev()),
            fuzzed: vec![0x0; bytes_to_insert],
            has_next: true,
            charset,
            format,
        }
    }
}

impl Payload for ReplaceBytes {
    fn next(&mut self) -> bool {
        if !self.has_next {
            return false;
        }

        for i in 0..self.indices.len() {
            if self.buffer[self.indices[i]] != 0xff {
                self.buffer[self.indices[i]] = self.buffer[self.indices[i]].wrapping_add(1);
                return true;
            }

            self.buffer[self.indices[i]] = self.data[self.indices[i]];
            let mut finished: bool = false;

            if self.indices[i] < self.buffer.len() - i - 1 {
                self.indices[i] += 1;
                finished = true;
            } else {
                if i == self.indices.len() - 1 {
                    self.has_next = false;
                    return false;
                }

                self.indices[i] = self.indices[i + 1] + 1;
            }

            for j in 0..i {
                self.buffer[self.indices[j]] = self.data[self.indices[j]];
            }

            for j in (0..i).rev() {
                self.indices[j] = self.indices[j + 1] + 1;
                self.buffer[self.indices[j]] = 0;
            }

            self.buffer[self.indices[i]] = 0;

            if finished {
                return true;
            }
        }

        return false;
    }

    fn get_payload(&mut self) -> &Vec<u8> {
        return &self.buffer;
    }
}

impl Payload for InsertBytes {
    fn next(&mut self) -> bool {
        if !self.has_next {
            return false;
        }

        for i in 0..self.indices.len() {
            if self.buffer[self.indices[i]] != 0xff {
                self.buffer[self.indices[i]] = self.buffer[self.indices[i]].wrapping_add(1);
                return true;
            }

            if self.indices[i] < self.buffer.len() - i - 1 {
                self.buffer[self.indices[i]] = self.buffer[self.indices[i] + 1];
                // self.indices[i] += 1;

                for j in 0..(i + 1) {
                    // let temp = self.buffer[self.indices[j]];
                    self.buffer[self.indices[j]] = self.buffer[self.indices[j] + 1];
                    self.indices[j] += 1;
                    self.buffer[self.indices[j]] = 0;
                }
                self.buffer[self.indices[i]] = 0;
                return true;
            } else {
                if i == self.indices.len() - 1 {
                    self.has_next = false;
                    return false;
                }

                // self.indices[i] = self.indices[i + 1] + 1;

                for j in (0..(i + 1)).rev() {
                    while self.indices[j] > self.indices[j + 1] + 1 {
                        // println!("switch {} {}", self.indices[j] - 1, self.indices[j]);
                        let temp = self.buffer[self.indices[j]];
                        self.buffer[self.indices[j]] = self.buffer[self.indices[j] - 1];
                        self.buffer[self.indices[j] - 1] = temp;
                        self.indices[j] -= 1;
                    }
                }

                self.buffer[self.indices[i]] = 0;
                // self.indices[j] = self.indices[j + 1] + 1;
                // self.buffer[self.indices[j]] = 0;
            }

            // for j in 0..i {
            //     // self.buffer[self.indices[j]] = self.data[self.indices[j]];
            //     self.buffer[self.indices[j]] = self.data[self.indices[j]];
            // }

            // println!("{:?} {}", self.indices, self.indices[i]);
            // for j in (0..i).rev() {
            //     println!("{} {}", self.indices[i], self.indices[j]);
            //     while self.indices[j] > self.indices[j + 1] + 1 {
            //         println!("switch {} {}", self.indices[j] - 1, self.indices[j]);
            //         let temp = self.buffer[self.indices[j]];
            //         self.buffer[self.indices[j]] = self.buffer[self.indices[j] - 1];
            //         self.buffer[self.indices[j] - 1] = temp;
            //         self.indices[j] -= 1;
            //     }
            //     // self.indices[j] = self.indices[j + 1] + 1;
            //     // self.buffer[self.indices[j]] = 0;
            // }
        }

        return false;
    }

    fn get_payload(&mut self) -> &Vec<u8> {
        return &self.buffer;
    }
}

impl Payload for ReplaceFormatted {
    fn next(&mut self) -> bool {
        if !self.has_next {
            return false;
        }

        for i in 0..self.indices.len() {
            self.fuzzed[i] += 1;

            if self.charset.contains(&self.fuzzed[i]) {
                let mut formatted: Vec<String> = Vec::with_capacity(self.indices.len());

                for j in 0..self.indices.len() {
                    formatted.push((self.format)(self.fuzzed[j]));
                }

                let mut pieces: Vec<&[u8]> = vec![
                    &self.data[0..self.indices[self.indices.len() - 1]],
                    &formatted[formatted.len() - 1].as_bytes(),
                ];

                for j in (0..(self.indices.len() - 1)).rev() {
                    pieces.push(&self.data[(self.indices[j + 1] + 1)..self.indices[j]]);
                    pieces.push(&formatted[j].as_bytes());
                }

                pieces.push(&self.data[(self.indices[0] + 1)..self.data.len()]);
                // println!("{:02x?}", pieces);

                self.buffer = pieces.concat();

                return true;
            }

            self.fuzzed[i] = 0;

            if self.indices[i] < self.data.len() - i - 1 {
                self.indices[i] += 1;

                for j in (0..i).rev() {
                    self.indices[j] = self.indices[j + 1] + 1;
                }
                return true;
            } else {
                if i == self.indices.len() - 1 {
                    self.has_next = false;
                    return false;
                }

                self.indices[i] = self.indices[i + 1] + 1;

                for j in (0..i).rev() {
                    self.indices[j] = self.indices[j + 1] + 1;
                }
            }
        }

        return false;
    }

    fn get_payload(&mut self) -> &Vec<u8> {
        return &self.buffer;
    }
}

impl Payload for InsertFormatted {
    fn next(&mut self) -> bool {
        if !self.has_next {
            return false;
        }

        for i in 0..self.indices.len() {
            self.fuzzed[i] += 1;

            if self.charset.contains(&self.fuzzed[i]) {
                let mut formatted: Vec<String> = Vec::with_capacity(self.indices.len());

                for j in 0..self.indices.len() {
                    formatted.push((self.format)(self.fuzzed[j]));
                }

                let mut pieces: Vec<&[u8]> = vec![
                    &self.data[0..self.indices[self.indices.len() - 1]],
                    &formatted[formatted.len() - 1].as_bytes(),
                ];

                for j in (0..(self.indices.len() - 1)).rev() {
                    pieces.push(&self.data[self.indices[j + 1]..self.indices[j]]);
                    pieces.push(&formatted[j].as_bytes());
                }

                pieces.push(&self.data[self.indices[0]..self.data.len()]);
                // println!("{:02x?}", pieces);

                self.buffer = pieces.concat();

                return true;
            }

            self.fuzzed[i] = 0;

            if self.indices[i] < self.data.len() {
                self.indices[i] += 1;

                for j in (0..i).rev() {
                    self.indices[j] = self.indices[j + 1];
                }
                return true;
            } else {
                if i == self.indices.len() - 1 {
                    self.has_next = false;
                    return false;
                }

                self.indices[i] = self.indices[i + 1];

                for j in (0..i).rev() {
                    self.indices[j] = self.indices[j + 1];
                }
            }
        }

        return false;
    }

    fn get_payload(&mut self) -> &Vec<u8> {
        return &self.buffer;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_creation_and_next() {
        let data: Vec<u8> = vec![0; 4];
        let mut payload: ReplaceBytes = ReplaceBytes::new(&data, 3);
        payload.next();
        payload.buffer[1] = 0xff;
    }

    #[test]
    fn replace_one_byte() {
        let data: Vec<u8> = vec![0; 4];
        let mut buffer: Vec<u8> = data.clone();
        let mut payload: ReplaceBytes = ReplaceBytes::new(&data, 1);

        for i in 0..data.len() {
            for _ in 0..=255u8 {
                buffer[i] = buffer[i].wrapping_add(1);
                payload.next();

                assert!(buffer.iter().zip(&payload.buffer).all(|(a, b)| a == b));
            }

            buffer[i] = data[i];
        }
    }

    #[test]
    fn replace_two_bytes() {
        let data: Vec<u8> = Vec::from("{\"q\":0}");
        let mut buffer: Vec<u8> = data.clone();
        let mut payload: ReplaceBytes = ReplaceBytes::new(&data, 2);

        for i in 0..(data.len() - 1) {
            buffer[i] = 0;

            for _ in 0..=255u8 {
                for j in (i + 1)..(data.len()) {
                    buffer[j] = 0;

                    for b in 0..=255u8 {
                        assert!(
                            buffer.iter().zip(&payload.buffer).all(|(a, b)| a == b),
                            "i,j,b: {},{},{}\nexpected: {:02X?}\ngot:      {:02X?}",
                            i,
                            j,
                            b,
                            &buffer,
                            &payload.buffer
                        );

                        buffer[j] = buffer[j].wrapping_add(1);
                        assert!(
                            payload.has_next,
                            "should have next i,j,b: {},{},{}  indices: {:?}\nbuffer: {:02x?}",
                            i, j, b, &payload.indices, &payload.buffer
                        );
                        payload.next();
                    }

                    buffer[j] = data[j];
                }

                buffer[i] = buffer[i].wrapping_add(1);
            }

            buffer[i] = data[i];
        }

        assert!(
            !payload.next(),
            "Should not have next.\nindices: {:?}\nbuffer: {:02x?}",
            &payload.indices,
            &payload.buffer
        );
    }

    #[test]
    fn replace_three_bytes() {
        let data: Vec<u8> = vec![0xcc; 4];
        let mut buffer: Vec<u8> = data.clone();
        let mut payload: ReplaceBytes = ReplaceBytes::new(&data, 3);

        for i in 0..(data.len() - 2) {
            buffer[i] = 0;

            for _ in 0..=255u8 {
                for j in (i + 1)..(data.len() - 1) {
                    buffer[j] = 0;

                    for _ in 0..=255u8 {
                        for k in (j + 1)..data.len() {
                            buffer[k] = 0;

                            for b in 0..=255u8 {
                                assert!(
                                    buffer.iter().zip(&payload.buffer).all(|(a, b)| a == b),
                                    "i,j,k,b: {},{},{},{}  indices: {:?}\nexpected: {:02X?}\ngot:      {:02X?}",
                                    i,
                                    j,
                                    k,
                                    b,
                                    &payload.indices,
                                    &buffer,
                                    &payload.buffer
                                );

                                buffer[k] = buffer[k].wrapping_add(1);
                                assert!(
                                    payload.has_next,
                                    "should have next i,j,j,b: {},{},{},{}  indices: {:?}\nbuffer: {:02x?}",
                                    i,
                                    j,
                                    k,
                                    b,
                                    &payload.indices,
                                    &payload.buffer
                                );
                                payload.next();
                            }

                            buffer[k] = data[k];
                        }

                        buffer[j] = buffer[j].wrapping_add(1);
                    }

                    buffer[j] = data[j];
                }

                buffer[i] = buffer[i].wrapping_add(1);
            }

            buffer[i] = data[i];
        }

        assert!(
            !payload.next(),
            "Should not have next.\nindices: {:?}\nbuffer: {:02x?}",
            &payload.indices,
            &payload.buffer
        );
    }

    #[test]
    fn insert_one_byte() {
        let data: Vec<u8> = vec![0x11; 4];
        let mut buffer: Vec<u8> = [vec![0], data.clone()].concat();
        let mut payload: InsertBytes = InsertBytes::new(&data, 1);

        for i in 0..buffer.len() {
            buffer[i] = 0;

            for _ in 0..=255u8 {
                // println!("{:02x?} {:02x?}", buffer, payload.buffer);
                assert!(buffer.iter().zip(&payload.buffer).all(|(a, b)| a == b));

                buffer[i] = buffer[i].wrapping_add(1);
                payload.next();
            }

            if i < buffer.len() - 1 {
                buffer[i] = buffer[i + 1];
            }
        }
    }

    #[test]
    fn insert_two_bytes() {
        let data: Vec<u8> = Vec::from(r#"{"q":0}"#);
        let mut buffer: Vec<u8> = [vec![0, 0], data.clone()].concat();
        let mut payload: InsertBytes = InsertBytes::new(&data, 2);

        for i in 0..(buffer.len() - 1) {
            buffer[i] = 0;

            for _ in 0..=255u8 {
                for j in (i + 1)..(buffer.len()) {
                    buffer[j] = 0;

                    for b in 0..=255u8 {
                        // println!("{:02x?} {:02x?}", buffer, payload.buffer);
                        assert!(
                            buffer.iter().zip(&payload.buffer).all(|(a, b)| a == b),
                            "i,j,b: {},{},{}\nexpected: {:02X?}\ngot:      {:02X?}",
                            i,
                            j,
                            b,
                            &buffer,
                            &payload.buffer
                        );

                        buffer[j] = buffer[j].wrapping_add(1);
                        assert!(
                            payload.has_next,
                            "should have next i,j,b: {},{},{}  indices: {:?}\nbuffer: {:02x?}",
                            i, j, b, &payload.indices, &payload.buffer
                        );
                        payload.next();
                    }

                    // buffer[j] = data[j];
                    if j < buffer.len() - 1 {
                        buffer[j] = buffer[j + 1];
                    } else {
                        for k in ((i + 1)..(buffer.len() - 1)).rev() {
                            let temp = buffer[k];
                            buffer[k] = buffer[k + 1];
                            buffer[k + 1] = temp;
                        }
                    }
                }

                buffer[i] = buffer[i].wrapping_add(1);
            }

            // buffer[i] = data[i];
            if i < buffer.len() - 2 {
                buffer[i] = buffer[i + 2];
            }
        }

        assert!(
            !payload.next(),
            "Should not have next.\nindices: {:?}\nbuffer: {:02x?}",
            &payload.indices,
            &payload.buffer
        );
    }

    #[test]
    fn insert_one_formatted() {
        let data: Vec<u8> = vec![0x10, 0x11, 0x12, 0x13];
        let buffer: Vec<u8> = [vec![0], data.clone()].concat();

        let range: Range<usize> = Range {
            start: 0,
            end: 0x10000,
        };

        let mut payload: InsertFormatted =
            InsertFormatted::new(&data, 1, range, |b| format!("\\u{:04x}", b));

        for _i in 0..buffer.len() {
            // buffer[i] = 0;

            for _ in 0..0x10000 {
                println!("{:02x?}", payload.buffer);
                // println!("{:02x?} {:02x?}", buffer, payload.buffer);
                // assert!(buffer.iter().zip(&payload.buffer).all(|(a, b)| a == b));
                //
                // buffer[i] = buffer[i].wrapping_add(1);
                payload.next();
                assert!(payload.has_next);
            }

            // if i < buffer.len() - 1 {
            //     buffer[i] = buffer[i + 1];
            // }
        }

        assert!(false);
    }

    // #[test]
    // fn insert_three_bytes() {
    //     let data: Vec<u8> = vec![0xcc; 4];
    //     let mut buffer: Vec<u8> = data.clone();
    //     let mut payload: ReplaceBytes = ReplaceBytes::new(&data, 3);
    //
    //     for i in 0..(data.len() - 2) {
    //         buffer[i] = 0;
    //
    //         for _ in 0..=255u8 {
    //             for j in (i + 1)..(data.len() - 1) {
    //                 buffer[j] = 0;
    //
    //                 for _ in 0..=255u8 {
    //                     for k in (j + 1)..data.len() {
    //                         buffer[k] = 0;
    //
    //                         for b in 0..=255u8 {
    //                             assert!(
    //                                 buffer.iter().zip(&payload.buffer).all(|(a, b)| a == b),
    //                                 "i,j,k,b: {},{},{},{}  indices: {:?}\nexpected: {:02X?}\ngot:      {:02X?}",
    //                                 i,
    //                                 j,
    //                                 k,
    //                                 b,
    //                                 &payload.indices,
    //                                 &buffer,
    //                                 &payload.buffer
    //                             );
    //
    //                             buffer[k] = buffer[k].wrapping_add(1);
    //                             assert!(
    //                                 payload.has_next,
    //                                 "should have next i,j,j,b: {},{},{},{}  indices: {:?}\nbuffer: {:02x?}",
    //                                 i,
    //                                 j,
    //                                 k,
    //                                 b,
    //                                 &payload.indices,
    //                                 &payload.buffer
    //                             );
    //                             payload.next();
    //                         }
    //
    //                         buffer[k] = data[k];
    //                     }
    //
    //                     buffer[j] = buffer[j].wrapping_add(1);
    //                 }
    //
    //                 buffer[j] = data[j];
    //             }
    //
    //             buffer[i] = buffer[i].wrapping_add(1);
    //         }
    //
    //         buffer[i] = data[i];
    //     }
    //
    //     assert!(
    //         !payload.next(),
    //         "Should not have next.\nindices: {:?}\nbuffer: {:02x?}",
    //         &payload.indices,
    //         &payload.buffer
    //     );
    // }
}
