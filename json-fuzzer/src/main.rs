mod compression;
mod payload;

use clap::{Parser, ValueEnum};
use compression::*;
// use flate2::write::ZlibEncoder;
// use flate2::Compression;
use payload::*;
use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

#[derive(ValueEnum, Debug, Clone)]
enum FuzzingType {
    ReplaceOneByte,
    ReplaceTwoBytes,
    ReplaceThreeBytes,
    All,
}

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t=String::from("{\"q\":0,\"q\":1}"))]
    payload: String,

    #[arg(short, long, value_delimiter = ',', help = "Leave empty for all")]
    fuzzing_types: Vec<FuzzingType>,

    #[arg(short, long, default_value_t=1 << 20)]
    buffer_size: usize,
    // /// Number of times to greet
    // #[arg(short, long, default_value_t = 1)]
    // count: u8,
}

impl std::fmt::Display for FuzzingType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FuzzingType::ReplaceOneByte => write!(f, "replace-one-byte"),
            FuzzingType::ReplaceTwoBytes => write!(f, "replace-two-bytes"),
            FuzzingType::ReplaceThreeBytes => write!(f, "replace-three-bytes"),
            _ => write!(f, "<?>"),
        }
    }
}

fn fuzz(stream: &mut TcpStream, args: &Args, payload: &mut Box<dyn Payload>) -> Encoder {
    let batch_size: u16 = std::cmp::min(args.buffer_size / 128, u16::MAX as usize) as u16;
    let mut read_buffer: Box<Vec<u8>> = Box::new(vec![0; args.buffer_size]);
    let mut send_buffer: Box<Vec<u8>> = Box::new(vec![0; args.buffer_size]);
    let start = std::time::Instant::now();

    // let data: &[u8] = args.payload.as_bytes();
    // let mut payload: ReplaceBytes = ReplaceBytes::new(&data, 2);
    // let mut test_payload: ReplaceBytes = ReplaceBytes::new(&data, 2);
    // let mut compression_encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    let mut compression_encoder = Encoder::new();

    let mut received_messages: usize = 0;
    let mut sent_messages: usize = 0;
    let mut has_next: bool = true;
    let mut sent_bytes: usize = 0;
    let in_buffer_count: usize = 1;

    loop {
        // Fill send_buffer with fuzzed payloads and send it to the client
        if has_next && (sent_messages - received_messages) < in_buffer_count {
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
                    .write_all(&send_buffer[0..(payload.get_payload().len() * batch_size as usize)])
                    .expect("Write error");

                sent_bytes = 0;
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

            compression_encoder
                .add_bytes(&read_buffer[(byte_offset - data.len() - 2)..byte_offset]);

            // let str = match std::str::from_utf8(data) {
            //     Ok(s) => s,
            //     Err(_) => "utf-8 parse error",
            // };
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

    println!(
        "n: {}  elapsed: {}s  count: {:.1}k/s  compressed size: {:.1}kb  compression ratio: {:.1}%",
        received_messages * batch_size as usize,
        start.elapsed().as_secs(),
        (received_messages as f64 * batch_size as f64) / (start.elapsed().as_millis() as f64),
        compression_encoder.bytes.len() as f64 / 1000.0,
        compression_encoder.bytes.len() as f64 / compression_encoder.uncompressed_bytes as f64
            * 100.0,
    );

    // let compressed: Vec<u8> = compression_encoder
    //     .finish()
    //     .expect("Could not finish compression");

    return compression_encoder;
}

fn handle_client(stream: &mut TcpStream, args: &Args) {
    let batch_size: u16 = std::cmp::min(args.buffer_size / 128, u16::MAX as usize) as u16;
    // let mut read_buffer: Box<Vec<u8>> = Box::new(vec![0; args.buffer_size]);
    // let mut send_buffer: Box<Vec<u8>> = Box::new(vec![0; args.buffer_size]);
    // let start = std::time::Instant::now();
    // let mut read_size_buffer: [u8; 2] = [0; 2];
    // let data: Vec<u8> = vec![0x2e; data_buffer_size.into()];
    // let data: &[u8] = args.payload.as_bytes();
    // let n = 1_000_000_000;
    // let mut payload: ReplaceBytes = ReplaceBytes::new(&data, 2);
    // let mut test_payload: ReplaceBytes = ReplaceBytes::new(&data, 2);
    // // let mut compression_encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    // let mut compression_encoder = Encoder::new();
    let mut name_buffer: [u8; 32] = [0; 32];
    let base_payload: &[u8] = args.payload.as_bytes();

    if let Err(e) = stream.set_nodelay(true) {
        eprintln!("Could not set NODELAY: {}", e);
    }

    // Send communication buffer size
    stream
        .write_all(&u32::try_from(args.buffer_size).unwrap().to_le_bytes())
        .expect("Could not write buffer size");

    // Send payload size
    stream
        .write_all(&u16::try_from(base_payload.len()).unwrap().to_le_bytes())
        .expect("Could not write buffer size");

    // Send batch size
    stream
        .write_all(&batch_size.to_le_bytes())
        .expect("Could not write batch size");

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
            FuzzingType::ReplaceOneByte,
            FuzzingType::ReplaceTwoBytes,
            FuzzingType::ReplaceThreeBytes,
        ]
    };

    for fuzzing_type in fuzzing_types {
        if !fs::exists("../data/").expect("Could not check if directory exists") {
            fs::create_dir("../data").expect("Could not create directory");
        }

        let file_name: String = format!("../data/{}_{}.bin", client_name, fuzzing_type);

        if fs::exists(&file_name).expect("Could not check if file exists") {
            println!("'{}' already exists. Skipping.", file_name);
            continue;
        }

        let mut payload: Box<dyn Payload> = match fuzzing_type {
            FuzzingType::ReplaceOneByte => Box::new(ReplaceBytes::new(&base_payload, 1)),
            FuzzingType::ReplaceTwoBytes => Box::new(ReplaceBytes::new(&base_payload, 2)),
            FuzzingType::ReplaceThreeBytes => Box::new(ReplaceBytes::new(&base_payload, 3)),
            _ => Box::new(ReplaceBytes::new(&base_payload, 2)),
        };

        let compressed = fuzz(stream, &args, &mut payload);

        let mut file = fs::File::create(&file_name.as_str()).expect("Could not create file");
        file.write_all(&compressed.bytes)
            .expect("Could not write to file");
    }

    // println!("{:02X?}", compression_encoder.bytes);
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let listener = TcpListener::bind("127.0.0.1:5000")?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        println!("\nNew connection");

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
