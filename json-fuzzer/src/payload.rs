pub struct ReplaceBytes {
    pub data: Vec<u8>,
    pub buffer: Vec<u8>,
    //pub charset: Vec<u8>,
    pub indices: Vec<usize>,
    pub fuzzed_bytes: Vec<u8>,
}

impl ReplaceBytes {
    pub fn new(data: &Vec<u8>, bytes_to_replace: usize) -> ReplaceBytes {
        ReplaceBytes {
            data: data.clone(),
            buffer: data.clone(),
            //charset: Vec::from_iter(0u8..=255u8),
            indices: Vec::from_iter((0..bytes_to_replace).rev()),
            fuzzed_bytes: vec![0; bytes_to_replace],
        }
    }

    pub fn next(&mut self) -> bool {
        for i in 0..self.indices.len() {
            let buffer_i = self.indices[i];

            if buffer_i >= self.buffer.len() {
                return false;
            }

            self.fuzzed_bytes[i] = self.fuzzed_bytes[i].wrapping_add(1);
            self.buffer[buffer_i] = self.fuzzed_bytes[i];

            if self.fuzzed_bytes[i] != 0 {
                break;
            }

            self.indices[i] += 1;

            if self.indices[i] < self.buffer.len() - i {
                break;
            }

            if i == self.indices.len() - 1 {
                // println!("indices: {:0X?}", self.indices);
                // println!("buffer:  {:0X?}", self.buffer);
                return false;
            }
        }

        for i in (0..(self.indices.len() - 1)).rev() {
            if self.buffer[self.indices[i] - 1] == 0 && self.indices[i] >= self.buffer.len() - i {
                self.buffer[self.indices[i] - 1] = self.data[self.indices[i] - 1];
                self.indices[i] = self.indices[i + 1] + 1;
            }
        }

        return true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation_and_next() {
        let data: Vec<u8> = vec![0; 4];
        let mut payload: ReplaceBytes = ReplaceBytes::new(&data, 3);
        payload.next();
        payload.fuzzed_bytes[1] = 0xff;
        payload.buffer[1] = 0xff;
    }

    #[test]
    fn one_byte() {
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
    fn three_bytes() {
        let data: Vec<u8> = vec![0; 4];
        let mut buffer: Vec<u8> = data.clone();
        let mut payload: ReplaceBytes = ReplaceBytes::new(&data, 3);

        for i in 0..(data.len() - 2) {
            for _ in 0..=255u8 {
                for j in (i + 1)..(data.len() - 1) {
                    for _ in 0..=255u8 {
                        for k in (j + 1)..data.len() {
                            for b in 0..=255u8 {
                                assert!(
                                    buffer.iter().zip(&payload.buffer).all(|(a, b)| a == b),
                                    "i,j,k,b: {},{},{},{}\nexpected: {:0X?}\ngot:      {:0X?}",
                                    i,
                                    j,
                                    k,
                                    b,
                                    &buffer,
                                    &payload.buffer
                                );
                                // println!("{:0X?}\n{:0X?}\n", &buffer, &payload.buffer);

                                buffer[k] = buffer[k].wrapping_add(1);
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
    }
}
