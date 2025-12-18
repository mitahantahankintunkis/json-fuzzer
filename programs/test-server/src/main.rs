use clap::Parser;
use regex::Regex;
use rustyline;
use std::io::{prelude::*, Error};
use std::net::{TcpListener, TcpStream};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, RwLock};
use std::thread;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "q")]
    key: String,
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

fn byte_to_string(byte: u8) -> String {
    match byte {
        0x00..0x07 | 0x0e..0x20 | 0x7f.. => format!("\\x{:02x}", byte),
        0x07 => "\\a".to_string(),
        0x08 => "\\b".to_string(),
        0x09 => "\\t".to_string(),
        0x0a => "\\n".to_string(),
        0x0b => "\\v".to_string(),
        0x0c => "\\f".to_string(),
        0x0d => "\\r".to_string(),
        _ => (byte as char).to_string(),
    }
}

struct Client {
    name: String,
    flags: u8,
    stream: CombinedStream,
}

impl Client {
    fn parse_json(&mut self, json: &[u8], key: &str) -> (u64, String) {
        let header_size = 9 + key.len();

        let mut send_buffer: Box<Vec<u8>> = Box::new(vec![0; header_size + json.len() + 2]);
        let mut read_buffer: Box<Vec<u8>> = Box::new(vec![0; 1 << 18]);

        send_buffer[0..4].copy_from_slice(&u32::try_from(json.len() + 2).unwrap().to_le_bytes());
        send_buffer[5..9].copy_from_slice(&(key.len() as u32).to_le_bytes());
        send_buffer[9..header_size].copy_from_slice(&key.bytes().collect::<Vec<u8>>());

        send_buffer[header_size..header_size + 2]
            .clone_from_slice(&(json.len() as u16).to_le_bytes());
        send_buffer[header_size + 2..header_size + 2 + json.len()].copy_from_slice(json);

        if self.stream.write_all(&send_buffer).is_err() {
            return (0, "CRITICAL_ERROR (write)".into());
        }

        // Read client response size
        if self.stream.read_exact(&mut read_buffer[0..4]).is_err() {
            println!(
                "\nClient {} crashed on input {} with key {}\n",
                self.name,
                json.iter()
                    .map(|j| byte_to_string(*j))
                    .collect::<Vec<String>>()
                    .join(""),
                key
            );
            return (0, "CRITICAL_ERROR (read)".into());
        }

        let batched_package_size: usize =
            usize::try_from(u32::from_le_bytes(read_buffer[0..4].try_into().unwrap())).unwrap();

        if batched_package_size > read_buffer.len() - 4 {
            panic!("Client sent too many bytes: {}", batched_package_size);
        }

        // Read client response
        let mut read_offset: usize = 4;
        self.stream
            .read_exact(&mut read_buffer[read_offset..(read_offset + batched_package_size)])
            .expect("Read error");

        let ns: u64 = (u32::from_le_bytes(
            read_buffer[read_offset..(read_offset + 4)]
                .try_into()
                .unwrap(),
        )) as u64
            * 10;
        read_offset += 4;

        // Skip coverage information
        if self.flags & 0b0000_0001 == 1 {
            let coverage_len: usize = u16::from_le_bytes(
                read_buffer[read_offset..(read_offset + 2)]
                    .try_into()
                    .unwrap(),
            ) as usize;
            read_offset += coverage_len + 2;
        }

        // let _coverage_hash: u32 = if self.flags & 1 == 1 {
        //     read_offset += 4;
        //     u32::from_le_bytes(
        //         read_buffer[(read_offset - 4)..read_offset]
        //             .try_into()
        //             .unwrap(),
        //     )
        // } else {
        //     0
        // };

        let package_size: u16 = u16::from_le_bytes(
            read_buffer[read_offset..(read_offset + 2)]
                .try_into()
                .unwrap(),
        );

        read_offset += 2;
        let data: &[u8] = &read_buffer[read_offset..(read_offset + usize::from(package_size))];

        let str = match std::str::from_utf8(data) {
            Ok(s) => s,
            Err(_) => "utf-8 parse error",
        };

        return (ns, str.to_string());
    }
}

fn handle_client(mut stream: CombinedStream, clients: Arc<RwLock<Vec<Client>>>) {
    let mut client_info: [u8; 65] = [0; 65];

    // Read client name
    stream
        .read_exact(&mut client_info)
        .expect("Could not read client name");

    let mut name_length = 0;

    for i in 0..client_info.len() {
        if client_info[i] == 0 {
            break;
        }

        name_length = i;
    }

    let client_name: String = std::str::from_utf8(&client_info[0..name_length + 1])
        .unwrap_or("<?>")
        .to_string();
    let client_flags = client_info[64];

    match clients.write() {
        Ok(mut s) => s.push(Client {
            name: client_name,
            flags: client_flags,
            stream,
        }),
        Err(e) => eprintln!("{}", e),
    }
}

enum TimeType {
    Nanos,
    Micros,
    Millis,
    Secs,
    Mins,
}

fn ns_to_string(ns: f64) -> (String, TimeType) {
    match ns.abs() {
        0.0..1000.0 => (format!("{:.0}ns", ns), TimeType::Nanos),
        1000.0..1000_000.0 => (format!("{:.0}µs", ns / 1000.0), TimeType::Micros),
        1000_000.0..1000_000_000.0 => (format!("{:.0}ms", ns / 1000_000.0), TimeType::Millis),
        1000_000_000.0..60_000_000_000.0 => {
            (format!("{:.0}s", ns / 1000_000_000.0), TimeType::Secs)
        }
        _ => (format!("{:.0}m", ns / 60_000_000_000.0), TimeType::Mins),
    }
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let clients: Arc<RwLock<Vec<Client>>> = Arc::new(RwLock::new(Vec::new()));

    // Listen for connections
    let listener = TcpListener::bind("127.0.0.1:5000")?;
    let tcp_thread_streams = clients.clone();

    thread::spawn(move || loop {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    handle_client(CombinedStream::TCP(stream), tcp_thread_streams.clone());
                }
                Err(e) => {
                    eprintln!("Error: {}", e)
                }
            }
        }
    });

    // Listen for connections
    let sock = "/tmp/fuzzer.sock";

    if std::fs::exists(sock).unwrap() {
        std::fs::remove_file(sock).unwrap();
    }

    let listener = UnixListener::bind(sock)?;
    let unix_thread_streams = clients.clone();

    thread::spawn(move || loop {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    handle_client(CombinedStream::Unix(stream), unix_thread_streams.clone());
                }
                Err(e) => {
                    eprintln!("Error: {}", e)
                }
            }
        }
    });

    let mut reader = rustyline::DefaultEditor::new().unwrap();
    let re = Regex::new(r"\\x[0-9a-f]{2}").unwrap();

    loop {
        let line = reader.readline("\n\n\x1b[2K\x1b[1m=> ");
        println!("\x1b[22m");

        match line {
            Ok(mut input_string) => {
                if input_string.len() == 0 {
                    input_string = r#"{"q":1,"q":1234}"#.to_string();
                }

                reader.add_history_entry(input_string.clone()).unwrap();
                let max_name_len = clients
                    .read()
                    .map(|c| c.iter().map(|c| c.name.len()).max())
                    .unwrap()
                    .unwrap();

                match clients.write() {
                    Ok(mut c) => {
                        c.sort_by(|a, b| a.name.cmp(&b.name));

                        for client in c.iter_mut() {
                            let json = input_string.trim().to_string();
                            let mut prev = 0;
                            let mut json_bytes = Vec::new();
                            for mat in re.find_iter(&json) {
                                json_bytes.extend_from_slice(json[prev..mat.start()].as_bytes());
                                let byte =
                                    u8::from_str_radix(&json[(mat.start() + 2)..mat.end()], 16)
                                        .unwrap();

                                json_bytes.push(byte);
                                prev = mat.end();
                            }

                            json_bytes.extend_from_slice(json[prev..].as_bytes());

                            let (ns, parsed) = client.parse_json(&json_bytes, &args.key);
                            let (ns, t) = ns_to_string(ns as f64);
                            let color = match t {
                                TimeType::Micros => "\x1b[1;97m",
                                TimeType::Millis => "\x1b[1;93m",
                                TimeType::Secs => "\x1b[1;91m",
                                _ => "",
                            };

                            println!(
                                "{}{:>9}\x1b[0m {}[\"{}\"]{:spacing$} = {}",
                                color,
                                ns,
                                client.name,
                                args.key,
                                " ",
                                parsed,
                                spacing = max_name_len + 1 - client.name.len()
                            );
                        }
                    }
                    Err(e) => eprintln!("{}", e),
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    Ok(())
}
