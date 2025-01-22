use std::io::prelude::*;
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:5000")?;
    stream.set_nodelay(true)?;

    let mut header: [u8; 8] = [0; 8];

    stream
        .read_exact(&mut header)
        .expect("Could not read buffer size");

    let buffer_size = u32::from_le_bytes(header[0..4].try_into().unwrap());
    let payload_size = u16::from_le_bytes(header[4..6].try_into().unwrap());
    let batch_size = u16::from_le_bytes(header[6..8].try_into().unwrap());

    let mut read_buffer: Box<Vec<u8>> =
        Box::new(vec![0; usize::from(batch_size) * usize::from(payload_size)]);
    let mut write_buffer: Box<Vec<u8>> = Box::new(vec![0; buffer_size.try_into().unwrap()]);
    // read_buffer.resize(buffer_size * batch_size, 0);
    // write_buffer.resize(128 * batch_size, 0);

    loop {
        match stream.read_exact(&mut read_buffer) {
            Ok(()) => {
                let size_buffer: &[u8] = &payload_size.to_le_bytes();
                let mut byte_offset: usize = 4;

                // stream.write_all(&size_buffer).expect("Write error");
                for batch in 0..usize::from(batch_size) {
                    let data: &[u8] = &read_buffer[(batch * usize::from(payload_size))
                        ..((batch + 1) * usize::from(payload_size))];

                    write_buffer[byte_offset..(byte_offset + 2)].copy_from_slice(&size_buffer);
                    byte_offset += 2;

                    write_buffer[byte_offset..(byte_offset + data.len())].copy_from_slice(&data);
                    byte_offset += data.len();
                    // println!("size: {:0X?}   data: {:0X?}", &size_buffer, &data);
                    // for i in 0..128 {
                    //     if i < buffer_size {
                    //         write_buffer[batch * 128 + i] = read_buffer[batch * buffer_size + i];
                    //     } else {
                    //         write_buffer[batch * 128 + i] = 0;
                    //     }
                    // }
                }

                write_buffer[0..4]
                    .copy_from_slice(&u32::try_from(byte_offset - 4).unwrap().to_le_bytes());

                stream
                    // .write_all(&write_buffer)
                    .write_all(&write_buffer[0..byte_offset])
                    .expect("Write error");
            }
            Err(_e) => {
                println!("Connection closed");
                return Ok(());
            }
        }
        // match stream.read_exact(&mut read_buffer) {
        //     Ok(()) => {
        //         // stream.write_all(&size_buffer).expect("Write error");
        //         stream.write_all(&read_buffer).expect("Write error");
        //     }
        //     Err(_e) => {
        //         println!("Connection closed");
        //         return Ok(());
        //     }
        // }
    }
}
