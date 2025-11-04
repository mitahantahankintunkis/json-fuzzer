#![allow(unused)]
use crossbeam_channel::{select, unbounded, Receiver, Sender};
use std::{
    collections::{BTreeSet, HashSet},
    fs::{self, create_dir},
    net::TcpStream,
    os::unix::net::UnixStream,
    thread,
    time::Duration,
};

use crate::{
    compression::Encoder,
    fuzz::{create_fuzzers, load_testcases, TestCase},
    util::byte_to_string,
    Args,
};
use std::io::{prelude::*, Error};

pub enum CombinedStream {
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

struct Client {
    name: String,
    job_tx: Sender<TestCase>,
    #[allow(unused)]
    handle: thread::JoinHandle<()>,
}

struct ClientDone {
    id: usize,
    testcase: TestCase,
    fuzzed: bool,
    times: BTreeSet<TimeData>,
}

impl Client {
    fn new(
        mut stream: CombinedStream,
        done_tx: Sender<ClientDone>,
        args: &Args,
        id: usize,
    ) -> Self {
        let (job_tx, job_rx) = unbounded::<TestCase>();

        let mut name_buffer: [u8; 64] = [0; 64];
        let no_batching = args.no_batching.clone();

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

        let mut fuzzed = false;

        if !fs::exists("data/").expect("Could not check if data directory exists") {
            create_dir("data").expect("Could not create data/");
        }

        let name_clone = client_name.clone();
        let handle = thread::spawn(move || {
            let client_name = name_clone.clone();
            let buffer_size = 1 << 16;
            let mut read_buffer: Box<Vec<u8>> = Box::new(vec![0; buffer_size << 2]);
            let mut send_buffer: Box<Vec<u8>> = Box::new(vec![0; buffer_size]);

            while let Ok(testcase) = job_rx.recv() {
                println!("Client {:25} received testcase: {}", client_name, testcase);
                let mut times: BTreeSet<TimeData> = BTreeSet::new();

                for mut fuzzer in create_fuzzers(&testcase) {
                    let digest = format!("{:x}", md5::compute(&testcase.json));
                    let file_name = format!(
                        "data/{};{};{};{}.bin",
                        client_name,
                        fuzzer.id(),
                        &digest[0..8],
                        testcase.json[0..std::cmp::min(testcase.json.len(), 20)]
                            .replace("/", "\\x2f")
                            .replace(";", "\\x3b"),
                    );

                    if fs::exists(&file_name).expect("Could not check if file exists") {
                        continue;
                    }

                    fuzzed = true;

                    let start = std::time::Instant::now();

                    // RLE for storing results. Practically no overhead and achieves
                    // under 0.1% compression ratios for large datasets
                    let mut compression_encoder = Encoder::new();

                    compression_encoder.add_bytes(format!("{}", client_name,).as_bytes());

                    let key_len = testcase.key.len();
                    let header_bytes = 9 + key_len;
                    send_buffer[4] = 0;
                    send_buffer[5..9].copy_from_slice(&(key_len as u32).to_le_bytes());
                    send_buffer[9..9 + key_len]
                        .copy_from_slice(&testcase.key.bytes().collect::<Vec<u8>>());

                    loop {
                        // Fill send_buffer with fuzzed payloads and send it to the client
                        let mut write_offset: usize = header_bytes;
                        let mut has_next = true;

                        while write_offset < send_buffer.len() - 2 {
                            match fuzzer.copy_to_slice(&mut send_buffer[write_offset + 2..]) {
                                Ok(n) => {
                                    send_buffer[write_offset..write_offset + 2].copy_from_slice(
                                        &(u16::try_from(n).unwrap()).to_le_bytes(),
                                    );
                                    write_offset += n + 2;
                                }
                                Err(_) => break,
                            }

                            if fuzzer.advance().is_err() {
                                has_next = false;
                                break;
                            }

                            if no_batching {
                                break;
                            }
                        }

                        send_buffer[0..4]
                            .copy_from_slice(&((write_offset - header_bytes) as u32).to_le_bytes());

                        stream
                            .write_all(&send_buffer[0..write_offset])
                            .expect("Write error");

                        // Read client response size
                        if stream.read_exact(&mut read_buffer[0..4]).is_err() {
                            eprintln!(
                                "Client {} crashed. Input from {} leading to crash:",
                                client_name,
                                fuzzer.id(),
                            );

                            let mut i = header_bytes;
                            while i < write_offset {
                                let size: u16 =
                                    u16::from_le_bytes(send_buffer[i..(i + 2)].try_into().unwrap());
                                i += 2;

                                let mut s = String::new();
                                for j in i..i + size as usize {
                                    s.push_str(&byte_to_string(send_buffer[j]));
                                }

                                i += size as usize;
                                eprintln!("{}  ", s);
                            }

                            eprintln!("");
                            panic!();
                        }

                        let package_size =
                            u32::from_le_bytes(read_buffer[0..4].try_into().unwrap()) as usize;

                        if package_size > read_buffer.len() - 4 {
                            panic!("Client sent too many bytes: {}", package_size);
                        }

                        // Read client response
                        let mut send_offset: usize = header_bytes;
                        let mut read_offset: usize = 0;

                        stream
                            .read_exact(&mut read_buffer[0..package_size])
                            .expect("Read error");

                        while read_offset < package_size {
                            let micros: u32 = u32::from_le_bytes(
                                read_buffer[read_offset..(read_offset + 4)]
                                    .try_into()
                                    .unwrap(),
                            );
                            read_offset += 4;

                            let package_size: u16 = u16::from_le_bytes(
                                read_buffer[read_offset..(read_offset + 2)]
                                    .try_into()
                                    .unwrap(),
                            );
                            read_offset += 2;

                            let sent_size = u16::from_le_bytes(
                                send_buffer[send_offset..(send_offset + 2)]
                                    .try_into()
                                    .unwrap(),
                            ) as usize;
                            send_offset += 2;

                            let data = &read_buffer
                                [read_offset..(read_offset + usize::from(package_size))];
                            read_offset += package_size as usize;

                            // Parsing duration
                            let should_insert = if let Some(t) = times.first() {
                                micros > t.duration
                            } else {
                                true
                            };

                            if should_insert {
                                if times.len() >= 10000 {
                                    times.pop_first();
                                }

                                let buf = &send_buffer[send_offset..send_offset + sent_size];
                                let payload_str = buf
                                    .iter()
                                    .map(|c| byte_to_string(*c))
                                    .collect::<Vec<String>>()
                                    .join("");

                                if micros > 100_000 {
                                    println!(
                                        "\n    Possible DoS in {}: {} -> {} took {}ms to parse\n",
                                        client_name,
                                        payload_str,
                                        String::from_utf8(data.to_vec())
                                            .unwrap_or("<?>".to_string()),
                                        micros / 1000,
                                    );
                                }

                                times.insert(TimeData {
                                    duration: micros,
                                    testcase: TestCase {
                                        weight: 0.0,
                                        parent_json: Some(testcase.json.clone()),
                                        depth: testcase.depth + 1,
                                        json: payload_str,
                                        key: testcase.key.clone(),
                                    },
                                });
                            }

                            send_offset += sent_size;
                            compression_encoder.add_bytes(&data);
                        }

                        if !has_next {
                            break;
                        }
                    }

                    compression_encoder.finish();

                    let elapsed = start.elapsed();

                    // println!(
                    //     "{:25} {} {}\nn: {:10}k, {:7.1}k/s  dur: {}s  zip: {:.1}kb, {:.2}%\n",
                    //     client_name,
                    //     testcase.json,
                    //     fuzzer.id(),
                    //     compression_encoder.message_count / 1000,
                    //     (compression_encoder.message_count as f64) / (elapsed.as_millis() as f64),
                    //     elapsed.as_secs(),
                    //     compression_encoder.bytes.len() as f64 / 1000.0,
                    //     compression_encoder.bytes.len() as f64
                    //         / compression_encoder.uncompressed_bytes as f64
                    //         * 100.0,
                    // );

                    // let mut file =
                    //     fs::File::create(&file_name.as_str()).expect("Could not create file");
                    // file.write_all(&compression_encoder.bytes)
                    //     .expect("Could not write to file");
                }

                // thread::sleep(Duration::from_secs(2));

                done_tx
                    .send(ClientDone {
                        id: id,
                        testcase: testcase,
                        fuzzed,
                        times,
                    })
                    .unwrap();
            }
        });

        Client {
            name: client_name,
            job_tx,
            handle,
            // done_rx,
        }
    }
}

pub struct Job {
    pub testcase: TestCase,
    pub clients: Vec<String>,
}

pub struct Orchestrator {
    pub pool_size: usize,
    clients: Vec<Client>,
    task_queue: Vec<Vec<TestCase>>,
    round_robin_i: usize,
    active: HashSet<usize>,
    done_tx: Sender<ClientDone>,
    done_rx: Receiver<ClientDone>,
    pub connection_tx: Sender<CombinedStream>,
    connection_rx: Receiver<CombinedStream>,
    pub job_tx: Sender<Job>,
    job_rx: Receiver<Job>,
    result_tx: Sender<ParsingResult>,
}

impl Orchestrator {
    pub fn new(pool_size: usize, result_tx: Sender<ParsingResult>) -> Self {
        let (done_tx, done_rx) = unbounded();
        let (connection_tx, connection_rx) = unbounded();
        let (job_tx, job_rx) = unbounded();

        Orchestrator {
            pool_size,
            clients: Vec::new(),
            task_queue: Vec::new(),
            round_robin_i: 0,
            active: HashSet::new(),
            done_tx,
            done_rx,
            connection_tx,
            connection_rx,
            job_tx,
            job_rx,
            result_tx,
        }
    }

    pub fn join(&mut self, args: &Args) {
        self.round_robin_i = self.clients.len();

        loop {
            select! {
                recv(self.connection_rx) -> conn => {
                    self.create_client(conn.unwrap(), args);
                },
                recv(self.job_rx) -> job => {
                    let job = job.unwrap();
                    // println!("Server received job: {:?} {:?}", job.testcase, job.clients);

                    if job.clients.len() == 0 {
                        self.queue(&job.testcase, None);
                    } else {
                        self.queue(&job.testcase, Some(job.clients));
                    }

                    self.try_pop_queue();
                },
                recv(self.done_rx) -> res => {
                    let res = res.unwrap();
                    self.active.remove(&res.id);

                    // println!("Server received result: {} {:?}", res.id, res.testcase);

                    if res.fuzzed {
                        self.result_tx.send(ParsingResult {
                            client_name: self.clients[res.id].name.clone(),
                            testcase: res.testcase,
                            times: res.times,
                        }).unwrap();
                    }

                    self.try_pop_queue();
                },
            }
        }
    }

    pub fn create_client(&mut self, stream: CombinedStream, args: &Args) {
        self.clients.push(Client::new(
            stream,
            self.done_tx.clone(),
            args,
            self.clients.len(),
        ));

        let seeds = load_testcases();
        self.task_queue.push(seeds);

        // Give the client some time to initialize and CPU usage to stabilize
        thread::sleep(Duration::from_secs(2));
        self.try_pop_queue();
    }

    pub fn queue(&mut self, testcase: &TestCase, parsers: Option<Vec<String>>) {
        match parsers {
            Some(parsers) => {
                for parser in &parsers {
                    for i in 0..self.clients.len() {
                        if &self.clients[i].name == parser {
                            self.task_queue[i].push(testcase.clone());
                            break;
                        }
                    }
                }
            }
            None => {
                for i in 0..self.clients.len() {
                    self.task_queue[i].push(testcase.clone());
                }
            }
        }
    }

    fn try_pop_queue(&mut self) {
        // println!(
        //     "{:?}",
        //     self.task_queue
        //         .iter()
        //         .map(|e| e.len())
        //         .collect::<Vec<usize>>()
        // );
        if self.active.len() >= self.pool_size {
            return;
        }

        // Find task with lowest weight
        let mut min_i: Option<usize> = None;
        for i in 0..self.clients.len() {
            if self.active.contains(&i) {
                continue;
            }

            if let Some(cur) = self.task_queue[i].first() {
                if let Some(j) = min_i {
                    let best = self.task_queue[j].first().unwrap();
                    if cur.weight < best.weight {
                        min_i = Some(i);
                    }
                } else {
                    min_i = Some(i);
                }
            }
        }

        if let Some(i) = min_i {
            let testcase = self.task_queue[i].pop().unwrap();
            // println!("New tasks: {:?}", testcase);
            self.active.insert(i);
            self.clients[i].job_tx.send(testcase).unwrap();
        }
    }
}

pub struct TimeData {
    pub duration: u32,
    pub testcase: TestCase,
}

impl PartialEq for TimeData {
    fn eq(&self, other: &Self) -> bool {
        if self.duration == other.duration {
            return self.testcase.json == other.testcase.json;
        }

        self.duration == other.duration
    }
}

impl Eq for TimeData {}

impl Ord for TimeData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.duration.cmp(&other.duration)
    }
}

impl PartialOrd for TimeData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct ParsingResult {
    pub client_name: String,
    pub testcase: TestCase,
    #[allow(unused)]
    pub times: BTreeSet<TimeData>,
}
