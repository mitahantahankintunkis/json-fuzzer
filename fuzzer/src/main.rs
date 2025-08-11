mod analyze;
mod compression;
mod payload;

use clap::Parser;
use compression::*;
// use flate2::write::ZlibEncoder;
// use flate2::Compression;
use payload::*;
use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::ops::Range;
use std::thread;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, action)]
    analyze: bool,

    #[arg(short, long, default_value_t=String::from(r#"{"q":0,"q":1}"#))]
    payload: String,

    #[arg(short, long, value_delimiter = ',', help = "Leave empty for all")]
    fuzzing_types: Vec<FuzzingType>,

    #[arg(short, long, default_value_t=1 << 20)]
    buffer_size: usize,
}

fn fuzz(stream: &mut TcpStream, args: &Args, payload: &mut Box<dyn Payload>) -> Encoder {
    let batch_size: u16 = std::cmp::min(args.buffer_size / 128, u16::MAX as usize) as u16;
    let mut read_buffer: Box<Vec<u8>> = Box::new(vec![0; args.buffer_size]);
    let mut send_buffer: Box<Vec<u8>> = Box::new(vec![0; args.buffer_size]);

    let mut received_messages: usize = 0;
    let mut sent_messages: usize = 0;
    let mut has_next: bool = true;
    let in_buffer_count: usize = 1;

    // RLE for storing results. Practically no overhead and achieves
    // under 0.1% compression ratios for large datasets
    let mut compression_encoder = Encoder::new();
    // let mut compression_encoder = ZlibEncoder::new(Vec::new(), Compression::best());

    compression_encoder.add_bytes(&args.payload.as_bytes());

    // let range: Range<usize> = Range {
    //     start: 0,
    //     end: 0x10000,
    // };
    // let mut test_payload = InsertFormatted::new(&args.payload.as_bytes(), 2, range, |b| {
    //     format!("\\u{:04x}", b)
    // });

    send_buffer[0..4].copy_from_slice(&u32::try_from(args.buffer_size).unwrap().to_le_bytes());
    send_buffer[4..6].copy_from_slice(
        &u16::try_from(payload.get_payload().len())
            .unwrap()
            .to_le_bytes(),
    );
    send_buffer[6..8].copy_from_slice(&batch_size.to_le_bytes());
    let header_bytes = 8;

    loop {
        // Fill send_buffer with fuzzed payloads and send it to the client
        if has_next && (sent_messages - received_messages) < in_buffer_count {
            let mut sent_bytes: usize = header_bytes;

            for _ in 0..(in_buffer_count - (sent_messages - received_messages)) {
                for _ in 0..batch_size {
                    if has_next && !payload.next() {
                        has_next = false;
                    }

                    send_buffer[sent_bytes..(sent_bytes + payload.get_payload().len())]
                        .copy_from_slice(&payload.get_payload());
                    sent_bytes += payload.get_payload().len();
                }

                stream
                    .write_all(
                        &send_buffer[0..sent_bytes],
                        // [0..(header_bytes + payload.get_payload().len() * batch_size as usize)],
                    )
                    .expect("Write error");

                sent_messages += 1;
            }
        }

        if !has_next && received_messages == sent_messages {
            break;
        }

        // Read client response size
        stream
            .read_exact(&mut read_buffer[0..4])
            .expect("Read error");

        let batched_package_size: usize =
            usize::try_from(u32::from_le_bytes(read_buffer[0..4].try_into().unwrap())).unwrap();

        if batched_package_size > args.buffer_size - 4 {
            panic!("Client sent too many bytes: {}", batched_package_size);
        }

        // Read client response
        let mut byte_offset: usize = 4;
        stream
            .read_exact(&mut read_buffer[byte_offset..(byte_offset + batched_package_size)])
            .expect("Read error");

        for _i in 0..(batch_size as usize) {
            let package_size: u16 = u16::from_le_bytes(
                read_buffer[byte_offset..(byte_offset + 2)]
                    .try_into()
                    .unwrap(),
            );

            byte_offset += 2;
            let data: &[u8] = &read_buffer[byte_offset..(byte_offset + usize::from(package_size))];
            byte_offset += data.len();

            compression_encoder.add_bytes(&data);

            // let str = match std::str::from_utf8(data) {
            //     Ok(s) => s,
            //     Err(_) => "utf-8 parse error",
            // };
            //
            // test_payload.next();
            // let buffer = test_payload.get_payload();
            // let mut payload_str = String::with_capacity(buffer.len());
            //
            // for i in 0..buffer.len() {
            //     match buffer[i] {
            //         0u8..0x20 => payload_str.push_str(format!("\\x{:02x}", buffer[i]).as_str()),
            //         0x7fu8..=0xff => payload_str.push_str(format!("\\x{:02x}", buffer[i]).as_str()),
            //         b => payload_str.push(b as char),
            //     }
            // }
            //
            // println!("{} {}", payload_str, str);

            //
            // if str != "PARSE_ERROR" {
            //     let mut buffer = test_payload.buffer.clone();
            //
            //     for i in 0..buffer.len() {
            //         for b in 0u8..0x20 {
            //             if buffer[i] == b {
            //                 buffer[i] = 0;
            //             }
            //         }
            //
            //         for b in 0x7fu8..=0xff {
            //             if buffer[i] == b {
            //                 buffer[i] = 0;
            //             }
            //         }
            //     }
            //
            //     let payload_str = match std::str::from_utf8(&buffer) {
            //         Ok(s) => s,
            //         Err(_) => "utf-8 parse error",
            //     };
            //     let mut cleaned_str = str.replace("\n", "\\n");
            //     cleaned_str = cleaned_str.replace("\r", "\\r");
            //
            //     let mut cleaned_payload = payload_str.replace("\n", "\\n");
            //     cleaned_payload = cleaned_payload.replace("\r", "\\r");
            //
            //     println!("{} -> {:0X?} {}", cleaned_payload, data, cleaned_str);
            // }
            //
            // test_payload.next();
        }

        received_messages += 1;
    }

    compression_encoder.finish();

    return compression_encoder;
}

fn handle_client(stream: &mut TcpStream, args: &Args) {
    let mut name_buffer: [u8; 64] = [0; 64];
    let base_payload: &[u8] = args.payload.as_bytes();

    if let Err(e) = stream.set_nodelay(true) {
        eprintln!("Could not set NODELAY: {}", e);
    }

    // Read client name
    stream
        .read_exact(&mut name_buffer)
        .expect("Could not read client name");

    let mut name_length = 0;

    for i in 0..name_buffer.len() {
        if name_buffer[i] == 0 {
            break;
        }

        name_length = i;
    }

    let client_name: String = std::str::from_utf8(&name_buffer[0..name_length + 1])
        .expect("Could not parse client name")
        .to_string();

    let fuzzing_types = if args.fuzzing_types.len() != 0 {
        &args.fuzzing_types
    } else {
        &vec![
            FuzzingType::InsertOneByte,
            FuzzingType::ReplaceOneByte,
            FuzzingType::InsertTwoBytes,
            FuzzingType::ReplaceTwoBytes,
            FuzzingType::InsertOneUnicodeByte,
            FuzzingType::ReplaceOneUnicodeByte,
            FuzzingType::InsertTwoUnicodeBytes,
            FuzzingType::ReplaceTwoUnicodeBytes,
            FuzzingType::InsertThreeBytes,
            FuzzingType::ReplaceThreeBytes,
        ]
    };

    for fuzzing_type in fuzzing_types {
        if !fs::exists("../data/").expect("Could not check if directory exists") {
            let _ = fs::create_dir("../data");
        }

        let digest = md5::compute(base_payload);
        let file_name: String =
            format!("../data/{}_{}_{:x}.bin", client_name, fuzzing_type, digest);

        if fs::exists(&file_name).expect("Could not check if file exists") {
            println!("'{}' already exists. Skipping fuzzing.", file_name);
            continue;
        }

        let mut payload: Box<dyn Payload> = match fuzzing_type {
            FuzzingType::ReplaceOneByte => Box::new(ReplaceBytes::new(&base_payload, 1)),
            FuzzingType::ReplaceTwoBytes => Box::new(ReplaceBytes::new(&base_payload, 2)),
            FuzzingType::ReplaceThreeBytes => Box::new(ReplaceBytes::new(&base_payload, 3)),
            FuzzingType::InsertOneByte => Box::new(InsertBytes::new(&base_payload, 1)),
            FuzzingType::InsertTwoBytes => Box::new(InsertBytes::new(&base_payload, 2)),
            FuzzingType::InsertThreeBytes => Box::new(InsertBytes::new(&base_payload, 3)),
            FuzzingType::ReplaceOneUnicodeByte => Box::new(ReplaceFormatted::new(
                &base_payload,
                1,
                Range::<usize> {
                    start: 0,
                    end: 0x10000,
                },
                |b| format!("\\u{:04x}", b),
            )),
            FuzzingType::ReplaceTwoUnicodeBytes => Box::new(ReplaceFormatted::new(
                &base_payload,
                2,
                Range::<usize> {
                    start: 0,
                    end: 0x10000,
                },
                |b| format!("\\u{:04x}", b),
            )),

            FuzzingType::ReplaceThreeUnicodeBytes => Box::new(ReplaceFormatted::new(
                &base_payload,
                3,
                Range::<usize> {
                    start: 0,
                    end: 0x10000,
                },
                |b| format!("\\u{:04x}", b),
            )),

            FuzzingType::InsertOneUnicodeByte => Box::new(InsertFormatted::new(
                &base_payload,
                1,
                Range::<usize> {
                    start: 0,
                    end: 0x10000,
                },
                |b| format!("\\u{:04x}", b),
            )),

            FuzzingType::InsertTwoUnicodeBytes => Box::new(InsertFormatted::new(
                &base_payload,
                2,
                Range::<usize> {
                    start: 0,
                    end: 0x10000,
                },
                |b| format!("\\u{:04x}", b),
            )),

            FuzzingType::InsertThreeUnicodeBytes => Box::new(InsertFormatted::new(
                &base_payload,
                3,
                Range::<usize> {
                    start: 0,
                    end: 0x10000,
                },
                |b| format!("\\u{:04x}", b),
            )),
        };

        let start = std::time::Instant::now();
        let compressed = fuzz(stream, &args, &mut payload);

        println!(
            "{:25} {:25}  n: {:10}k, {:7.1}k/s  dur: {}s  zip: {:.1}kb, {:.2}%",
            client_name,
            fuzzing_type.to_string(),
            compressed.message_count / 1000,
            (compressed.message_count as f64) / (start.elapsed().as_millis() as f64),
            start.elapsed().as_secs(),
            compressed.bytes.len() as f64 / 1000.0,
            compressed.bytes.len() as f64 / compressed.uncompressed_bytes as f64 * 100.0,
        );

        let mut file = fs::File::create(&file_name.as_str()).expect("Could not create file");
        file.write_all(&compressed.bytes)
            .expect("Could not write to file");
    }

    // println!("{:02X?}", compression_encoder.bytes);
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    if args.analyze {
        analyze::analyze(&args);
        return Ok(());
    }

    let listener = TcpListener::bind("127.0.0.1:5000")?;
    // let listener = TcpListener::bind("::1:5000")?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let cloned_args = args.clone();

                thread::spawn(move || {
                    handle_client(&mut stream, &cloned_args);
                });
            }
            Err(e) => {
                eprintln!("Error: {}", e)
            }
        }
    }

    Ok(())
}
