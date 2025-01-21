use std::io::prelude::*;
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:5000")?;
    stream.set_nodelay(true)?;

    let mut read_buffer: Vec<u8> = Vec::new();
    let mut write_buffer: Vec<u8> = Vec::new();
    let mut size_buffer: [u8; 2] = [0; 2];

    stream
        .read_exact(&mut size_buffer)
        .expect("Could not read buffer size");
    let buffer_size: usize = u16::from_le_bytes(size_buffer).into();

    stream
        .read_exact(&mut size_buffer)
        .expect("Could not read batch size");
    let batch_size: usize = u16::from_le_bytes(size_buffer).into();

    // read_buffer.resize(buffer_size.into(), 0);
    read_buffer.resize(buffer_size * batch_size, 0);
    write_buffer.resize(128 * batch_size, 0);

    loop {
        match stream.read_exact(&mut read_buffer) {
            Ok(()) => {
                // stream.write_all(&size_buffer).expect("Write error");
                for batch in 0..batch_size {
                    for i in 0..128 {
                        if i < buffer_size {
                            write_buffer[batch * 128 + i] = read_buffer[batch * buffer_size + i];
                        } else {
                            write_buffer[batch * 128 + i] = 0;
                        }
                    }
                }

                stream.write_all(&write_buffer).expect("Write error");
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
