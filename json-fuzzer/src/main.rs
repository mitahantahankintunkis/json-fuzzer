mod payload;

use payload::*;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(stream: &mut TcpStream) {
    const BUFFER_SIZE: usize = 1 << 16;
    let data_buffer_size: u16 = 4;
    let batch_size: u16 = 100;
    // let mut read_size_buffer: [u8; 2] = [0; 2];
    let mut read_buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let mut send_buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let data: Vec<u8> = vec![0x11; data_buffer_size.into()];
    let start = std::time::Instant::now();
    let n = 100000000;
    let mut payload: ReplaceBytes = ReplaceBytes::new(&data, 3);
    let mut test_payload: ReplaceBytes = ReplaceBytes::new(&data, 3);

    if let Err(e) = stream.set_nodelay(true) {
        eprintln!("Could not set NODELAY: {}", e);
    }

    // Send data buffer size
    stream
        .write_all(&data_buffer_size.to_le_bytes())
        .expect("Could not write buffer size");

    // Send batch size
    stream
        .write_all(&batch_size.to_le_bytes())
        .expect("Could not write batch size");

    let mut received_messages: usize = 0;
    let mut sent_messages: usize = 0;
    let mut has_next: bool = true;
    let in_buffer_count: usize = 1;

    let mut sent_bytes: usize = 0;
    // let mut read_bytes: usize = 2;

    loop {
        if has_next && (sent_messages - received_messages) < in_buffer_count {
            for _ in 0..(in_buffer_count - (sent_messages - received_messages)) {
                for _ in 0..batch_size {
                    if has_next && !payload.next() {
                        has_next = false;
                    }

                    send_buffer[sent_bytes..(sent_bytes + payload.buffer.len())]
                        .copy_from_slice(&payload.buffer);
                    sent_bytes += payload.buffer.len();
                }

                // if sent_bytes >= payload.buffer.len() {
                // }

                stream
                    .write_all(&send_buffer[0..(data_buffer_size as usize * batch_size as usize)])
                    //.write_all(&payload.buffer[sent_bytes..payload.buffer.len()])
                    .expect("Write error");

                sent_bytes = 0;
                sent_messages += 1;
            }
        }

        if sent_messages * batch_size as usize >= n {
            has_next = false;
        }

        if !has_next && received_messages == sent_messages {
            break;
        }

        if sent_messages <= received_messages {
            continue;
        }

        // stream
        //     .read_exact(&mut read_size_buffer)
        //     .expect("Read error");

        // let cur_read_size = u16::from_le_bytes(read_size_buffer).into();
        let cur_read_size: usize = 128 * (batch_size as usize);

        if cur_read_size >= BUFFER_SIZE {
            panic!("Client sent too many bytes: {}", cur_read_size);
        }

        stream
            .read_exact(&mut read_buffer[0..cur_read_size])
            .expect("Read error");

        for i in 0..(batch_size as usize) {
            let data: &[u8] = &read_buffer[(i * 128)..((i + 1) * 128)];
            test_payload.next();

            assert!(
                data.iter().zip(&test_payload.buffer).all(|(a, b)| a == b),
                "\nbatch: {}\nexpected: {:0X?}\ngot:      {:0X?}",
                i,
                &test_payload.buffer,
                &data,
            );
        }

        received_messages += 1;
    }

    println!(
        "n: {}  elapsed: {:.1}ms  count: {:.1}k/s",
        received_messages * batch_size as usize,
        start.elapsed().as_millis(),
        (received_messages as f64 * batch_size as f64) / (start.elapsed().as_millis() as f64)
    );
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:5000")?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        println!("\nNew connection");

        match stream {
            Ok(mut stream) => {
                thread::spawn(move || {
                    handle_client(&mut stream);
                });
            }
            Err(e) => {
                eprintln!("Error: {}", e)
            }
        }
    }

    Ok(())
}
