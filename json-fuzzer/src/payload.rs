pub struct ReplaceBytes {
    pub data: Vec<u8>,
    pub buffer: Vec<u8>,
    //pub charset: Vec<u8>,
    pub indices: Vec<usize>,
    pub has_next: bool,
}

impl ReplaceBytes {
    pub fn new(data: &[u8], bytes_to_replace: usize) -> ReplaceBytes {
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

    pub fn next(&mut self) -> bool {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation_and_next() {
        let data: Vec<u8> = vec![0; 4];
        let mut payload: ReplaceBytes = ReplaceBytes::new(&data, 3);
        payload.next();
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
    fn two_bytes() {
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
    fn three_bytes() {
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
}
