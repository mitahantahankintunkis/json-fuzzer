mod analyze;
mod compression;
mod payload;
mod util;

use clap::Parser;
use compression::*;
use payload::{load_payloads, Payload, PayloadConfig};
use std::cmp::{max, min};
use std::fs::{exists, read, remove_file};
use std::io::{prelude::*, Error};
use std::net::{TcpListener, TcpStream};
use std::os::unix::net::{UnixListener, UnixStream};
use std::time::Instant;
use std::{fs, thread};

use crate::util::byte_to_string;

const SOCK_FILE: &str = "/tmp/fuzzer.sock";

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, action)]
    analyze: bool,
    #[arg(short, long, action)]
    fuzz_dos: bool,
    #[arg(long)]
    find_payload: Option<String>,
    #[arg(long)]
    find_parser: Option<String>,
    #[arg(long)]
    debug: Option<String>,
}

enum CombinedStream {
    Unix(UnixStream),
    TCP(TcpStream),
}

impl CombinedStream {
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        match self {
            CombinedStream::Unix(s) => s.read_exact(buf),
            CombinedStream::TCP(s) => s.read_exact(buf),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Error> {
        match self {
            CombinedStream::Unix(s) => s.write_all(buf),
            CombinedStream::TCP(s) => s.write_all(buf),
        }
    }
}

fn fuzz(stream: &mut CombinedStream, payload_config: &PayloadConfig) -> Encoder {
    let mut payload: Payload = payload_config.clone().into();
    // let mut test_payload: Payload = payload_config.clone().into();
    let buffer_size = 1 << 20;
    let mut read_buffer: Box<Vec<u8>> = Box::new(vec![0; buffer_size << 2]);
    let mut send_buffer: Box<Vec<u8>> = Box::new(vec![0; buffer_size]);

    let mut received_messages: usize = 0;
    let mut sent_messages: usize = 0;
    let mut has_next: bool = true;
    let in_buffer_count: usize = 1;
    let payload_size = payload.byte_count;

    // RLE for storing results. Practically no overhead and achieves
    // under 0.1% compression ratios for large datasets
    let mut compression_encoder = Encoder::new();

    compression_encoder
        .add_bytes(format!("{} ({})", payload_config.name, payload_config.payload).as_bytes());

    let header_bytes = 10;
    let batch_size: u32 = ((send_buffer.len() - header_bytes) / payload_size) as u32;
    send_buffer[0..4].copy_from_slice(&u32::try_from(buffer_size).unwrap().to_le_bytes());
    send_buffer[4..6].copy_from_slice(&u16::try_from(payload_size).unwrap().to_le_bytes());

    loop {
        // Fill send_buffer with fuzzed payloads and send it to the client
        if has_next && (sent_messages - received_messages) < in_buffer_count {
            let mut sent_bytes: usize = header_bytes;
            let mut batch_size = batch_size;

            for _ in 0..(in_buffer_count - (sent_messages - received_messages)) {
                // println!("    generate");
                for batch in 0..batch_size {
                    for byte in payload.into_iter() {
                        send_buffer[sent_bytes] = byte;
                        sent_bytes += 1;
                    }

                    if has_next && payload.advance().is_err() {
                        has_next = false;
                        batch_size = batch + 1;
                        // println!("    fin");
                        break;
                    }
                }

                send_buffer[6..10].copy_from_slice(&batch_size.to_le_bytes());

                stream
                    .write_all(&send_buffer[0..sent_bytes])
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

        let batched_package_size =
            u32::from_le_bytes(read_buffer[0..4].try_into().unwrap()) as usize;

        if batched_package_size > read_buffer.len() - 4 {
            panic!("Client sent too many bytes: {}", batched_package_size);
        }

        // Read client response
        let mut byte_offset: usize = 4;

        stream
            .read_exact(&mut read_buffer[byte_offset..(byte_offset + batched_package_size)])
            .expect("Read error");

        while byte_offset < batched_package_size + 4 {
            let package_size: u16 = u16::from_le_bytes(
                read_buffer[byte_offset..(byte_offset + 2)]
                    .try_into()
                    .unwrap(),
            );

            byte_offset += 2;
            let data = &read_buffer[byte_offset..(byte_offset + usize::from(package_size))];
            byte_offset += package_size as usize;

            compression_encoder.add_bytes(&data);

            // print!("red {} size {}:  ", batched_package_size, package_size);
            // for byte in test_payload.into_iter() {
            //     print!("{}", byte_to_string(byte));
            // }
            // let _ = test_payload.advance();
            //
            // print!(" -> ");
            //
            // for byte in data.into_iter() {
            //     print!("{}", byte_to_string(*byte));
            // }
            //
            // println!("");
        }

        received_messages += 1;
    }

    compression_encoder.finish();

    return compression_encoder;
}

struct TreeNode {
    json: String,
    nanoseconds: u64,
    children: Vec<usize>,
    fuzzed: bool,
    verified: bool,
}

struct Arena {
    nodes: Box<Vec<TreeNode>>,
}

impl Arena {
    pub fn max2(&self) -> Option<usize> {
        if self.nodes.len() == 0 {
            return None;
        } else if self.nodes.len() <= 1 {
            return Some(0);
        }

        let mut best_scl: [f64; 3] = [0f64; 3];
        let mut scl: [f64; 3] = [0f64; 3];
        let mut best_node = 0;
        // let mut best_is: [usize; 3] = [0; 3];
        // let mut is: [usize; 3] = [0; 3];
        scl[0] = self.nodes[0].nanoseconds as f64;

        for child0 in &self.nodes[0].children {
            scl[1] = self.nodes[*child0].nanoseconds as f64;
            // is[1] = *child0;

            if self.nodes[*child0].children.len() == 0 {
                let x = scl[0] + scl[1] * 1000.0;
                let y = best_scl[0] + best_scl[1] * 1000.0 + best_scl[2] * 1000_000.0;

                if x > y {
                    best_scl.copy_from_slice(&scl);
                    best_node = *child0;
                    // best_is.copy_from_slice(&is);
                }
            } else {
                for child1 in &self.nodes[*child0].children {
                    if self.nodes[*child1].fuzzed {
                        continue;
                    }

                    scl[2] = self.nodes[*child1].nanoseconds as f64;
                    // is[2] = *child1;

                    let x = scl[0] + scl[1] * 1000.0 + scl[2] * 1000_000.0;
                    let y = best_scl[0] + best_scl[1] * 1000.0 + best_scl[2] * 1000_000.0;

                    if x > y {
                        best_scl.copy_from_slice(&scl);
                        // best_is.copy_from_slice(&is);
                        best_node = *child1;
                    }

                    scl[2] = 0.0;
                }
            }
        }

        println!(
            "{} + {}x + {}x^2 -> {}",
            best_scl[0],
            best_scl[1],
            best_scl[2],
            best_scl[0] + best_scl[1] * 1000.0 + best_scl[2] * 1000_000.0,
        );

        if best_node == 0 {
            None
        } else {
            Some(best_node)
        }
    }

    // pub fn max(&self, node: usize) -> Option<(usize, usize)> {
    //     if self.nodes[node].children.len() == 0 && self.nodes[node].fuzzed {
    //         return None;
    //     }
    //
    //     if self.nodes[node].children.len() == 0 && self.nodes[node].json.len() > 20 {
    //         return None;
    //     }
    //
    //     let mut ret = (node, 0);
    //
    //     for child in &self.nodes[node].children {
    //         if let Some((max_child, depth)) = self.max(*child) {
    //             if self.nodes[max_child].nanoseconds > self.nodes[ret.0].nanoseconds
    //                 || self.nodes[ret.0].fuzzed
    //             {
    //                 ret = (max_child, 1 + depth);
    //             }
    //         }
    //     }
    //
    //     if self.nodes[ret.0].fuzzed {
    //         return None;
    //     }
    //
    //     Some(ret)
    // }
}

fn fuzz_dos(
    stream: &mut CombinedStream,
    payload_config: &PayloadConfig,
    parent: usize,
    arena: &mut Arena,
) {
    let mut config = payload_config.clone();
    config.payload = arena.nodes[parent].json.clone();
    let mut payload: Payload = config.clone().into();

    let mut read_buffer: Box<Vec<u8>> = Box::new(vec![0; 1 << 20]);
    let mut send_buffer: Box<Vec<u8>> = Box::new(vec![0; 1 << 20]);

    let payload_size = payload.byte_count;

    // let header_bytes = 10;
    let batch_size: u32 = 1000u32;
    send_buffer[0..4].copy_from_slice(
        &u32::try_from(payload_size * batch_size as usize)
            .unwrap()
            .to_le_bytes(),
    );
    send_buffer[4..6].copy_from_slice(&u16::try_from(payload_size).unwrap().to_le_bytes());
    send_buffer[6..10].copy_from_slice(&batch_size.to_le_bytes());

    #[inline]
    fn time_parsing(
        stream: &mut CombinedStream,
        send_buffer: &mut Box<Vec<u8>>,
        read_buffer: &mut Box<Vec<u8>>,
        bytes: &[u8],
        repeats: u32,
    ) -> u64 {
        let mut sent_bytes = 10;

        for _ in 0..repeats {
            for byte in bytes {
                send_buffer[sent_bytes] = *byte;
                sent_bytes += 1;
            }
        }

        stream
            .write_all(&send_buffer[0..sent_bytes])
            .expect("Write error");

        let start_time = Instant::now();

        stream
            .read_exact(&mut read_buffer[0..4])
            .expect("Read error");

        let dur = start_time.elapsed().as_nanos() as u64;
        let package_size = u32::from_le_bytes(read_buffer[0..4].try_into().unwrap()) as usize;
        let mut byte_offset = 4;

        stream
            .read_exact(&mut read_buffer[byte_offset..(byte_offset + package_size)])
            .expect("Read error");

        let package_size: u16 = u16::from_le_bytes(
            read_buffer[byte_offset..(byte_offset + 2)]
                .try_into()
                .unwrap(),
        );

        byte_offset += 2;
        let data = &read_buffer[byte_offset..(byte_offset + usize::from(package_size))];

        if let Ok(s) = String::from_utf8(data.into()) {
            if s == "KEY_NOT_FOUND" || s == "PARSE_ERROR" {
                return 0;
            }
        }

        dur / (repeats as u64)
    }

    let mut durations: Vec<u64> = Vec::new();

    if !arena.nodes[parent].verified {
        let json = arena.nodes[parent].json.clone();
        send_buffer[0..4].copy_from_slice(&(json.bytes().len() as u32 * batch_size).to_le_bytes());
        send_buffer[4..6].copy_from_slice(&(json.bytes().len() as u16).to_le_bytes());

        // Warmup
        for _ in 0..1001 {
            let _ = time_parsing(
                stream,
                &mut send_buffer,
                &mut read_buffer,
                &json.bytes().collect::<Vec<u8>>(),
                batch_size,
            );
        }

        // Verify
        for _ in 0..1001 {
            let dur = time_parsing(
                stream,
                &mut send_buffer,
                &mut read_buffer,
                &json.bytes().collect::<Vec<u8>>(),
                batch_size,
            );
            durations.push(dur);
        }

        durations.sort();
        let median = durations[durations.len() / 2];
        durations.clear();

        let delta = (arena.nodes[parent].nanoseconds as i64) - (median as i64);

        if delta > 10000 || delta < -10000 {
            println!(
                "verified {}  orig: {}  median: {}   delta: {}ns",
                arena.nodes[parent].json, arena.nodes[parent].nanoseconds, median, delta
            );
        }

        arena.nodes[parent].nanoseconds = median;
        arena.nodes[parent].verified = true;
        send_buffer[0..4].copy_from_slice(
            &u32::try_from(payload_size * batch_size as usize)
                .unwrap()
                .to_le_bytes(),
        );
        send_buffer[4..6].copy_from_slice(&u16::try_from(payload_size).unwrap().to_le_bytes());
    }

    loop {
        for _ in 0..3 {
            let dur = time_parsing(
                stream,
                &mut send_buffer,
                &mut read_buffer,
                &payload.into_iter().collect::<Vec<u8>>(),
                batch_size,
            );

            if dur == 0 {
                let _ = payload.advance();
                break;
            }

            durations.push(dur);
        }

        if durations.len() == 0 {
            continue;
        }

        let dur = max(
            min(durations[0], durations[1]),
            min(durations[1], durations[2]),
        );

        durations.clear();
        // println!("    median: {}", dur);

        let payload_str = payload
            .into_iter()
            .map(|c| byte_to_string(c))
            .collect::<Vec<String>>()
            .join("");

        // println!("    Parsed {} in {}ns {:?}", payload_str, dur, durations);

        // durations.sort();
        // let dur = durations[durations.len() / 2];
        // durations.clear();

        // let dur = max(
        //     min(durations[0], durations[1]),
        //     min(durations[1], durations[2]),
        // );
        // println!("    median: {}", dur);
        // durations.clear();

        let brk = payload.advance().is_err();

        let l = arena.nodes.len();
        arena.nodes[parent].children.push(l);
        arena.nodes.push(TreeNode {
            json: payload_str,
            nanoseconds: dur,
            children: Vec::new(),
            fuzzed: false,
            verified: false,
        });

        if brk {
            break;
        }
    }

    // println!(
    //     "found {} {} {}",
    //     arena.nodes[parent].children.len(),
    //     arena.nodes[parent]
    //         .children
    //         .iter()
    //         .map(|i| { arena.nodes[*i].nanoseconds })
    //         .max()
    //         .unwrap(),
    //     arena.nodes[parent]
    //         .children
    //         .iter()
    //         .map(|i| { arena.nodes[*i].nanoseconds })
    //         .min()
    //         .unwrap()
    // );
}

fn handle_client(stream: &mut CombinedStream, args: &Args) {
    let mut name_buffer: [u8; 64] = [0; 64];

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

    let mut skipped_configs = 0;

    if !fs::exists("data/").expect("Could not check if data directory exists") {
        let _ = fs::create_dir("data");
    }

    let config = if args.fuzz_dos {
        let payloads_string =
            fs::read_to_string("payloads_dos.toml").expect("Could not read payloads_dos.toml");
        toml::from_str(&payloads_string).expect("Error while parsing payloads.toml")
    } else {
        load_payloads()
    };

    for payload_config in &config.payloads {
        // let digest = md5::compute(base_payload);
        // let file_name: String = format!(
        //     "../data/{}-{}-{}.bin",
        //     client_name, payload_config.name, payload_config.payload
        // );
        if args.fuzz_dos {
            let mut arena = Arena {
                nodes: Box::new(vec![TreeNode {
                    json: payload_config.payload.clone(),
                    nanoseconds: 0,
                    children: Vec::new(),
                    fuzzed: false,
                    verified: false,
                }]),
            };

            let mut highest_json = payload_config.payload.clone();
            let mut highest_ns = 0;

            loop {
                // let (node, depth) = match arena.max(0) {
                //     Some((n, d)) => (n, d),
                //     None => {
                //         println!("All fuzzed");
                //         break;
                //     }
                // };
                let node = match arena.max2() {
                    Some(n) => n,
                    None => {
                        println!("All fuzzed");
                        break;
                    }
                };

                arena.nodes[node].fuzzed = true;

                let start = std::time::Instant::now();
                fuzz_dos(stream, &payload_config, node, &mut arena);
                let elapsed = start.elapsed();

                if arena.nodes[node].nanoseconds > highest_ns {
                    highest_ns = arena.nodes[node].nanoseconds;
                    highest_json = arena.nodes[node].json.clone();
                }

                println!(
                    "{:30} {:8}ns {:20}  max: {:8}ns {:20} {:7.1}k/s  dur: {}ms",
                    client_name,
                    arena.nodes[node].nanoseconds,
                    arena.nodes[node].json,
                    highest_ns,
                    highest_json,
                    (arena.nodes[node].children.len() as f64) / (elapsed.as_millis() as f64),
                    elapsed.as_millis(),
                );
            }
        } else {
            let file_name: String = format!("data/{}-{}.bin", client_name, payload_config.name);

            if fs::exists(&file_name).expect("Could not check if file exists") {
                skipped_configs += 1;
                continue;
            }

            let start = std::time::Instant::now();
            let compressed = fuzz(stream, &payload_config);
            let elapsed = start.elapsed();

            println!(
                "{:30} {}\nn: {:10}k, {:7.1}k/s  dur: {}s  zip: {:.1}kb, {:.2}%\n",
                client_name,
                payload_config.name,
                compressed.message_count / 1000,
                (compressed.message_count as f64) / (elapsed.as_millis() as f64),
                elapsed.as_secs(),
                compressed.bytes.len() as f64 / 1000.0,
                compressed.bytes.len() as f64 / compressed.uncompressed_bytes as f64 * 100.0,
            );

            let mut file = fs::File::create(&file_name.as_str()).expect("Could not create file");
            file.write_all(&compressed.bytes)
                .expect("Could not write to file");
        }
    }

    if skipped_configs != config.payloads.len() {
        println!("{} finished parsing.", client_name);
    }
}

fn debug(path: String) {
    let bytes = read(&path).expect("Could not open file");
    let mut decoder = Decoder::new(Box::new(bytes));
    let config = load_payloads();

    // Pop parser name
    decoder.next_message();

    let id = path
        .clone()
        .split("-")
        .map(|s| s.to_string())
        .collect::<Vec<String>>()[1]
        .clone()
        .split(".")
        .map(|s| s.to_string())
        .collect::<Vec<String>>()[0]
        .clone();

    let payload_config = config.payloads.iter().find(|c| c.name == id).unwrap();
    let mut payload: Payload = payload_config.clone().into();

    loop {
        let json = payload
            .into_iter()
            .map(|c| byte_to_string(c))
            .collect::<Vec<String>>()
            .join("");

        let output = decoder.next_message().expect("Not enough outputs");

        println!("{} -> {}", json, output);

        if payload.advance().is_err() {
            break;
        }
    }
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    if let Some(path) = args.debug {
        debug(path);
        return Ok(());
    }

    if args.analyze {
        analyze::analyze(&args);
        return Ok(());
    }

    if let Some(parser_name) = &args.find_parser {
        let config = load_payloads();
        let target_payload = args
            .find_payload
            .clone()
            .expect("find_payload needs to be specified");

        for payload_config in &config.payloads {
            let files = std::fs::read_dir("data/").expect("Could not open 'data/'");

            for file in files {
                if let Ok(f) = file {
                    let file_name: String = f.file_name().to_str().unwrap().to_string();
                    let split: Vec<&str> = file_name.splitn(2, '-').collect();

                    if split.len() != 2 {
                        eprintln!("Malformed filename '{}'", file_name);
                        continue;
                    }

                    let file_parser_name = &split[0];
                    let file_config_name = &split[1][0..split[1].len() - 4];

                    if file_config_name != payload_config.name {
                        continue;
                    }

                    if file_parser_name != parser_name {
                        continue;
                    }

                    let bytes = std::fs::read(f.path()).expect("Could not read file");
                    let mut decoder = Decoder::new(Box::new(bytes));
                    let mut payload: Payload = payload_config.clone().into();

                    while let Some(output) = decoder.next_message() {
                        let cur_payload = payload
                            .into_iter()
                            .map(|c| byte_to_string(c))
                            .collect::<Vec<String>>()
                            .join("");

                        if cur_payload == *target_payload {
                            println!("{}: {} -> {}", payload_config.name, cur_payload, output);
                        }

                        if let Err(_) = payload.advance() {
                            break;
                        }
                    }
                }
            }
        }
    }

    if exists(SOCK_FILE).unwrap() {
        remove_file(SOCK_FILE).expect(&format!(
            "Could not remove previous sock ({}) file",
            SOCK_FILE
        ));
    }

    let unix_args = args.clone();

    // accept connections and process them serially
    // Unix domain sockets
    thread::spawn(move || {
        let unix_listener = UnixListener::bind(SOCK_FILE).unwrap();

        for stream in unix_listener.incoming() {
            match stream {
                Ok(stream) => {
                    let unix_args = unix_args.clone();
                    thread::spawn(move || {
                        handle_client(&mut CombinedStream::Unix(stream), &unix_args);
                    });
                }
                Err(e) => {
                    eprintln!("Error: {}", e)
                }
            }
        }
    });

    // TCP sockets
    let tcp_listener = TcpListener::bind("127.0.0.1:5000")?;

    for stream in tcp_listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = stream.set_nodelay(true) {
                    eprintln!("Could not set NODELAY: {}", e);
                }

                let tcp_args = args.clone();
                thread::spawn(move || {
                    handle_client(&mut CombinedStream::TCP(stream), &tcp_args);
                });
            }
            Err(e) => {
                eprintln!("Error: {}", e)
            }
        }
    }

    Ok(())
}
