use regex::Regex;
use rustyline;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread;

// struct Input {
//     str: String,
//     n: usize,
// }

struct Client {
    name: String,
    stream: TcpStream,
}

impl Client {
    fn parse_json(&mut self, json: &[u8]) -> String {
        let batch_size: u16 = 1;
        let mut read_buffer: Box<Vec<u8>> = Box::new(vec![0; 1 << 16]);
        let mut send_buffer: Box<Vec<u8>> = Box::new(vec![0; 1 << 16]);

        send_buffer[0..4].copy_from_slice(&u32::try_from(1024).unwrap().to_le_bytes());
        send_buffer[6..8].copy_from_slice(&batch_size.to_le_bytes());
        // let mut prev_message_count = 0;

        // let payload: String;

        // loop {
        //     let user_input = input.read().unwrap();
        //     if prev_message_count != user_input.n {
        //         prev_message_count = user_input.n;
        //         payload = user_input.str.clone();
        //         break;
        //     }
        //
        //     std::thread::sleep(std::time::Duration::from_millis(10));
        // }

        send_buffer[4..6].copy_from_slice(&u16::try_from(json.len()).unwrap().to_le_bytes());
        send_buffer[8..(8 + json.len())].copy_from_slice(json);

        self.stream
            .write_all(&send_buffer[0..(8 + json.len())])
            .expect("Write error");

        // Read client response size
        self.stream
            .read_exact(&mut read_buffer[0..4])
            .expect("Read error");

        let batched_package_size: usize =
            usize::try_from(u32::from_le_bytes(read_buffer[0..4].try_into().unwrap())).unwrap();

        if batched_package_size > read_buffer.len() - 4 {
            panic!("Client sent too many bytes: {}", batched_package_size);
        }

        // Read client response
        let mut byte_offset: usize = 4;
        self.stream
            .read_exact(&mut read_buffer[byte_offset..(byte_offset + batched_package_size)])
            .expect("Read error");

        let package_size: u16 = u16::from_le_bytes(
            read_buffer[byte_offset..(byte_offset + 2)]
                .try_into()
                .unwrap(),
        );

        byte_offset += 2;
        let data: &[u8] = &read_buffer[byte_offset..(byte_offset + usize::from(package_size))];

        let str = match std::str::from_utf8(data) {
            Ok(s) => s,
            Err(_) => "utf-8 parse error",
        };

        return str.to_string();
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:5000")?;

    // let user_input: Arc<RwLock<Input>> = Arc::new(RwLock::new(Input {
    //     str: String::new(),
    //     n: 0,
    // }));

    let clients: Arc<RwLock<Vec<Client>>> = Arc::new(RwLock::new(Vec::new()));
    // let main_thread_input = user_input.clone();

    let thread_streams = clients.clone();
    // Listen for connections
    thread::spawn(move || loop {
        for stream in listener.incoming() {
            // let input = user_input.clone();

            match stream {
                Ok(mut stream) => {
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
                        .unwrap_or("<?>")
                        .to_string();

                    match thread_streams.write() {
                        Ok(mut s) => s.push(Client {
                            name: client_name,
                            stream,
                        }),
                        Err(e) => eprintln!("{}", e),
                    }
                    // thread::spawn(move || {
                    //     handle_client(&mut stream, input);
                    // });
                }
                Err(e) => {
                    eprintln!("Error: {}", e)
                }
            }
        }
    });

    let mut reader = rustyline::DefaultEditor::new().unwrap();
    let re = Regex::new(r"\\x\d{2}").unwrap();

    loop {
        let line = reader.readline("\n\n\x1b[2K\x1b[1m=> ");
        // let _n = std::io::stdin().read_line(&mut input_string).unwrap();
        // std::io::stdout().flush().unwrap();
        println!("\x1b[22m");

        match line {
            Ok(input_string) => {
                reader.add_history_entry(input_string.clone()).unwrap();

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
                                // let byte =
                                //     json[(mat.start() + 2)..mat.end()].parse::<u8>().unwrap();

                                json_bytes.push(byte);
                                prev = mat.end();
                            }

                            json_bytes.extend_from_slice(json[prev..].as_bytes());

                            let parsed = client.parse_json(&json_bytes);

                            println!(
                                "{}[\"q\"]{:spacing$} = {}",
                                client.name,
                                " ",
                                parsed,
                                spacing = 25 - client.name.len()
                            );
                        }
                    }
                    Err(e) => eprintln!("{}", e),
                }
            }
            // match main_thread_input.write() {
            //     Ok(mut w) => {
            //     }
            //     Err(e) => {
            //         eprintln!("Could not send input to threads: {}", e);
            //     }
            // },
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
