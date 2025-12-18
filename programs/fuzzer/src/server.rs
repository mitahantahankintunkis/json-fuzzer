// #![allow(unused)]
use crossbeam_channel::{select, unbounded, Receiver, Sender};
use rusqlite::{params, Connection};
use std::{
    collections::{BTreeSet, HashSet},
    fs::{self, create_dir},
    net::TcpStream,
    os::unix::net::UnixStream,
    thread,
    time::{Duration, Instant},
};

use crate::{
    compression::Encoder,
    fuzz::{create_fuzzers, TestCase},
    tui::ClientStatus,
    util::byte_to_string,
    Args,
};
use std::io::{prelude::*, Error};

pub enum CombinedStream {
    Unix(UnixStream),
    TCP(TcpStream),
}

pub struct ConnectionInfo {
    pub stream: CombinedStream,
    pub tui_tx: Sender<ClientStatus>,
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
    handle: thread::JoinHandle<()>,
    name: String,
    job_tx: Sender<TestCase>,
}

#[derive(Debug)]
pub struct ParsingResult {
    pub parser_id: usize,
    pub parser_name: String,
    pub testcase: TestCase,
    pub times: BTreeSet<TimeData>,
    pub parent_time: f64,
    pub parent_output: String,
    pub coverage: Vec<(TestCase, String, Vec<bool>)>,
    pub compressed_bytes: Vec<(String, Encoder)>,
    pub fuzzed: bool,
    pub parser_count: usize,
    // pub fuzzer: Box<dyn Fuzzer>,
    // pub coverage: HashMap<u32, TestCase>,
}

fn minimum_parse_time(
    stream: &mut CombinedStream,
    testcase: &TestCase,
    client_flags: u8,
    repeats: usize,
) -> (String, f64) {
    let json_len = testcase.json.len();
    let json_bytes = testcase.json.bytes().collect::<Vec<u8>>();
    let key_len = testcase.key.len();
    let header_bytes = 9 + key_len;
    let mut min: f64 = f64::MAX;
    // let mut times: Vec<f64> = Vec::with_capacity(repeats);
    let mut send_buffer: Box<Vec<u8>> = Box::new(vec![0; json_len + header_bytes + 2]);
    let mut read_buffer: Box<Vec<u8>> = Box::new(vec![0; 64]);
    let mut output = String::new();

    send_buffer[4] = 0;
    send_buffer[5..9].copy_from_slice(&(key_len as u32).to_le_bytes());
    send_buffer[9..9 + key_len].copy_from_slice(&testcase.key.bytes().collect::<Vec<u8>>());

    // Measure parent testcase parsing time accurately
    for _ in 0..repeats {
        // Send testcases one at a time to hopefully trash the CPU cache
        let mut write_offset: usize = header_bytes;

        send_buffer[write_offset..write_offset + 2]
            .copy_from_slice(&(u16::try_from(json_len).unwrap()).to_le_bytes());
        write_offset += 2;

        send_buffer[write_offset..write_offset + json_len].copy_from_slice(&json_bytes);
        write_offset += json_len;

        send_buffer[0..4].copy_from_slice(&((write_offset - header_bytes) as u32).to_le_bytes());

        stream
            .write_all(&send_buffer[0..write_offset])
            .expect("Write error");

        // Read client response size
        stream
            .read_exact(&mut read_buffer[0..4])
            .expect("Client crashed. Could not verify time");

        let package_size = u32::from_le_bytes(read_buffer[0..4].try_into().unwrap()) as usize;

        if package_size > read_buffer.len() - 4 {
            read_buffer.resize(package_size + 4, 0);
            // panic!("Client sent too many bytes: {}", package_size);
        }

        // Read client response
        // let mut send_offset: usize = header_bytes;

        stream
            .read_exact(&mut read_buffer[0..package_size])
            .expect("Read error");

        // Parsing duration
        let ns: f64 = u32::from_le_bytes(read_buffer[0..4].try_into().unwrap()) as f64 * 10.0;
        if min > ns {
            min = ns;
        }

        if output.is_empty() {
            let mut read_offset = 4;

            // Skip coverage information
            if client_flags & 0b0000_0001 == 1 {
                let coverage_len: usize = u16::from_le_bytes(
                    read_buffer[read_offset..(read_offset + 2)]
                        .try_into()
                        .unwrap(),
                ) as usize;
                read_offset += coverage_len + 2;
            }

            // Length of parsed data
            let package_size: usize = u16::from_le_bytes(
                read_buffer[read_offset..(read_offset + 2)]
                    .try_into()
                    .unwrap(),
            ) as usize;
            read_offset += 2;

            // Parsed data
            let data = &read_buffer[read_offset..(read_offset + package_size)];
            output = String::from_utf8(data.to_vec()).unwrap_or("UTF8_ERROR".to_string());
        }
    }

    (output, min)
}

impl Client {
    fn new(
        mut conn: ConnectionInfo,
        done_tx: Sender<ParsingResult>,
        args: &Args,
        id: usize,
    ) -> Self {
        let (job_tx, job_rx) = unbounded::<TestCase>();

        let mut client_info: [u8; 65] = [0; 65];
        let no_batching = args.no_batching.clone();

        // Read client name
        conn.stream
            .read_exact(&mut client_info)
            .expect("Could not read client info");

        let mut name_length = 0;

        for i in 0..client_info.len() {
            if client_info[i] == 0 {
                break;
            }

            name_length = i;
        }

        let client_name: String = std::str::from_utf8(&client_info[0..name_length + 1])
            .expect("Could not parse client name")
            .to_string();
        let client_flags = client_info[client_info.len() - 1];

        if !fs::exists("data/").expect("Could not check if data directory exists") {
            create_dir("data").expect("Could not create data/");
        }

        // Update TUI
        conn.tui_tx
            .send(ClientStatus {
                name: client_name.clone(),
                data: None,
                parses_per_second: 0.0,
            })
            .unwrap();

        let db_conn = Connection::open("analyzed/db.sqlite").unwrap();
        let name_clone = client_name.clone();
        let handle = thread::spawn(move || {
            let client_name = name_clone.clone();
            let buffer_size = 1 << 16;
            let mut read_buffer: Box<Vec<u8>> = Box::new(vec![0; buffer_size << 4]);
            let mut send_buffer: Box<Vec<u8>> = Box::new(vec![0; buffer_size]);
            let mut coverage: Vec<bool> = Vec::new();
            let mut first_job = true;
            let mut stopped = false;

            while let Ok(mut testcase) = job_rx.recv() {
                if testcase.id < 0 {
                    panic!("{} Invalid testcase id: {:?}", client_name, testcase);
                }

                // Update TUI
                // conn.tui_tx
                //     .send(ClientStatus {
                //         name: client_name.clone(),
                //         data: Some(TimeData {
                //             duration: 0.0,
                //             testcase: testcase.clone(),
                //             output: "".into(),
                //         }),
                //         parses_per_second: 0.0,
                //     })
                //     .unwrap();

                let mut times: BTreeSet<TimeData> = BTreeSet::new();
                let mut new_coverage: Vec<(TestCase, String, Vec<bool>)> = Vec::new();
                let mut compressed: Vec<(String, Encoder)> = Vec::new();

                // Give the client some time to initialize and CPU usage to stabilize
                if first_job {
                    thread::sleep(Duration::from_secs(2));
                    first_job = false;
                }

                testcase.parser = Some(client_name.clone());

                let query = db_conn.query_one(
                    "SELECT 1
                    FROM parsing_times
                    WHERE parser = ?1 AND testcase = ?2
                    LIMIT 1",
                    params![client_name, testcase.id],
                    |row| {
                        let i: usize = row.get(0)?;
                        Ok(i)
                    },
                );

                if query.is_ok() {
                    // println!("Skipping {}", client_name);
                    done_tx
                        .send(ParsingResult {
                            parser_id: id,
                            parser_name: name_clone.clone(),
                            testcase: testcase,
                            times,
                            coverage: new_coverage,
                            parent_time: 0.0,
                            parent_output: "".to_string(),
                            compressed_bytes: compressed,
                            fuzzed: false,
                            parser_count: 0,
                        })
                        .unwrap();
                    continue;
                }

                let key_len = testcase.key.len();
                // let json_len = testcase.json.len();
                // let json_bytes = &testcase.json.bytes().collect::<Vec<u8>>();
                let header_bytes = 9 + key_len;
                send_buffer[4] = 0;
                send_buffer[5..9].copy_from_slice(&(key_len as u32).to_le_bytes());
                send_buffer[9..9 + key_len]
                    .copy_from_slice(&testcase.key.bytes().collect::<Vec<u8>>());
                // let mut verify_sent = 0;

                // Seed cases get more repetitions
                let (parent_output, parent_median) = if testcase.weight == 0.0 {
                    minimum_parse_time(&mut conn.stream, &testcase, client_flags, 100_000)
                } else {
                    minimum_parse_time(&mut conn.stream, &testcase, client_flags, 10_000)
                };

                // println!(
                //     "Client {:25} received testcase: {}   {} {}ns  {}",
                //     client_name,
                //     testcase.json,
                //     parent_output,
                //     parent_median,
                //     testcase.weight.round()
                // );

                // Update TUI
                conn.tui_tx
                    .send(ClientStatus {
                        name: client_name.clone(),
                        data: Some(TimeData {
                            duration: parent_median,
                            testcase: testcase.clone(),
                            output: parent_output.clone(),
                        }),
                        parses_per_second: 0.0,
                    })
                    .unwrap();

                let fuzzers = create_fuzzers(&testcase);
                let mut elapsed = 0.0;
                let mut total_payloads = 0;

                for mut fuzzer in fuzzers {
                    if stopped {
                        break;
                    }

                    // RLE for storing results. Practically no overhead and achieves
                    // under 0.1% compression ratios for large datasets
                    let mut compression_encoder = Encoder::new();
                    compression_encoder.add_bytes(format!("{}", client_name).as_bytes());

                    loop {
                        // Fill send_buffer with fuzzed payloads and send it to the client
                        let mut write_offset: usize = header_bytes;
                        let mut has_next = true;
                        let start = std::time::Instant::now();

                        while write_offset < send_buffer.len() - 2 {
                            match fuzzer.copy_to_slice(&mut send_buffer[write_offset + 2..]) {
                                Ok(n) => {
                                    // let testcase_str = send_buffer
                                    //     [write_offset + 2..write_offset + 2 + n]
                                    //     .iter()
                                    //     .map(|c| byte_to_string(*c))
                                    //     .collect::<Vec<String>>()
                                    //     .join("");
                                    // println!("{}", testcase_str);
                                    send_buffer[write_offset..write_offset + 2].copy_from_slice(
                                        &(u16::try_from(n).unwrap()).to_le_bytes(),
                                    );
                                    write_offset += n + 2;
                                    total_payloads += 1;
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

                        conn.stream
                            .write_all(&send_buffer[0..write_offset])
                            .expect("Write error");

                        // Read client response size
                        if conn.stream.read_exact(&mut read_buffer[0..4]).is_err() {
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

                            conn.tui_tx
                                .send(ClientStatus {
                                    name: client_name.clone(),
                                    data: Some(TimeData {
                                        duration: 0.0,
                                        testcase: TestCase::new(
                                            "<CRASHED>".into(),
                                            "".into(),
                                            None,
                                        ),
                                        output: "".into(),
                                    }),
                                    parses_per_second: (total_payloads as f64 * 1000_000.0)
                                        / elapsed,
                                })
                                .unwrap();

                            stopped = true;
                            break;
                        }

                        // Measuring only the parsing times for the test cases.
                        // Testing each test case takes longer than what is displayed
                        // in the TUI, as it does not include parsing time verification.
                        elapsed += start.elapsed().as_micros() as f64;

                        let package_size =
                            u32::from_le_bytes(read_buffer[0..4].try_into().unwrap()) as usize;

                        if package_size > read_buffer.len() - 4 {
                            read_buffer.resize(package_size, 0);
                            // panic!("Client sent too many bytes: {}", package_size);
                        }

                        // Read client response
                        let mut send_offset: usize = header_bytes;
                        let mut read_offset: usize = 0;

                        conn.stream
                            .read_exact(&mut read_buffer[0..package_size])
                            .expect("Read error");

                        while read_offset < package_size {
                            // Length of JSON test case
                            let sent_size = u16::from_le_bytes(
                                send_buffer[send_offset..(send_offset + 2)]
                                    .try_into()
                                    .unwrap(),
                            ) as usize;
                            send_offset += 2;

                            // JSON test case as String
                            let buf = &send_buffer[send_offset..send_offset + sent_size];
                            let testcase_str = buf
                                .iter()
                                .map(|c| byte_to_string(*c))
                                .collect::<Vec<String>>()
                                .join("");
                            send_offset += sent_size;

                            // Parsing duration
                            let ns: f64 = u32::from_le_bytes(
                                read_buffer[read_offset..(read_offset + 4)]
                                    .try_into()
                                    .unwrap(),
                            ) as f64
                                * 10.0;
                            read_offset += 4;

                            // Coverage information
                            let coverage_info = if client_flags & 0b0000_0001 == 1 {
                                let coverage_len: usize = u16::from_le_bytes(
                                    read_buffer[read_offset..(read_offset + 2)]
                                        .try_into()
                                        .unwrap(),
                                )
                                    as usize;
                                read_offset += 2;

                                let coverage_info =
                                    &read_buffer[read_offset..read_offset + coverage_len];
                                read_offset += coverage_len;

                                if coverage_len > coverage.len() {
                                    coverage.resize(coverage_len, false);
                                }

                                Some(coverage_info)
                            } else {
                                None
                            };

                            // Length of parsed data
                            let package_size: usize = u16::from_le_bytes(
                                read_buffer[read_offset..(read_offset + 2)]
                                    .try_into()
                                    .unwrap(),
                            ) as usize;
                            read_offset += 2;

                            // Parsed data
                            let data = &read_buffer[read_offset..(read_offset + package_size)];
                            read_offset += package_size;

                            if let Some(coverage_info) = coverage_info {
                                let mut saved_coverage = false;

                                for (i, b) in coverage_info.iter().enumerate() {
                                    if *b > 0 && !coverage[i] {
                                        coverage[i] = true;

                                        if !saved_coverage {
                                            saved_coverage = true;

                                            let new_testcase = TestCase::new(
                                                testcase_str.clone(),
                                                testcase.key.clone(),
                                                Some(testcase.clone()),
                                            );

                                            let new_coverage_info =
                                                coverage_info.iter().map(|c| *c > 0).collect();

                                            let output = data
                                                .iter()
                                                .map(|c| byte_to_string(*c))
                                                .collect::<Vec<String>>()
                                                .join("");

                                            new_coverage.push((
                                                new_testcase,
                                                output,
                                                new_coverage_info,
                                            ));
                                        }
                                    }
                                }
                            }

                            // Store parsing duration. Only a subset of the
                            // parsing durations are stored
                            let fastest = match times.first() {
                                Some(t) => t.duration,
                                None => 0.0,
                            };

                            if ns > fastest {
                                let (output, verified_ns) = minimum_parse_time(
                                    &mut conn.stream,
                                    &TestCase::new(
                                        testcase_str.clone(),
                                        testcase.key.clone(),
                                        None,
                                    ),
                                    client_flags,
                                    10,
                                );

                                if verified_ns > fastest {
                                    if times.len() >= 10 {
                                        times.pop_first();
                                    }

                                    times.insert(TimeData {
                                        duration: ns,
                                        testcase: TestCase {
                                            id: -1,
                                            weight: 0.0,
                                            parent_id: Some(testcase.id),
                                            depth: testcase.depth + 1,
                                            json: testcase_str.clone(),
                                            key: testcase.key.clone(),
                                            parser: Some(client_name.clone()),
                                        },
                                        output,
                                    });
                                }
                            }

                            compression_encoder.add_bytes(&data);
                        }

                        if !has_next {
                            break;
                        }
                    }

                    if stopped {
                        break;
                    }

                    let _ = compression_encoder.finish();
                    compressed.push((fuzzer.id(), compression_encoder));

                    // done_tx
                    //     .send(ParsingResult {
                    //         client_id: id,
                    //         client_name: name_clone.clone(),
                    //         testcase: testcase,
                    //         fuzzed,
                    //         times,
                    //         coverage: new_coverage,
                    //         compressed_bytes: *compression_encoder.finish(),
                    //         fuzzer: fuzzer,
                    //     })
                    //     .unwrap();

                    //
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

                if stopped {
                    break;
                }

                // thread::sleep(Duration::from_secs(2));

                // Update TUI
                conn.tui_tx
                    .send(ClientStatus {
                        name: client_name.clone(),
                        data: Some(TimeData {
                            duration: parent_median,
                            testcase: testcase.clone(),
                            output: parent_output.clone(),
                        }),
                        parses_per_second: (total_payloads as f64 * 1000_000.0) / elapsed,
                    })
                    .unwrap();

                done_tx
                    .send(ParsingResult {
                        parser_id: id,
                        parser_name: name_clone.clone(),
                        testcase: testcase,
                        times,
                        coverage: new_coverage,
                        parent_time: parent_median,
                        parent_output,
                        compressed_bytes: compressed,
                        fuzzed: true,
                        parser_count: 0,
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
    pub parsers: Vec<String>,
}

pub struct Orchestrator {
    pub pool_size: usize,
    clients: Vec<Client>,
    task_queue: Vec<BTreeSet<TestCase>>,
    active: HashSet<usize>,
    last_job_time: Vec<Instant>,
    done_tx: Sender<ParsingResult>,
    done_rx: Receiver<ParsingResult>,
    pub connection_tx: Sender<ConnectionInfo>,
    connection_rx: Receiver<ConnectionInfo>,
    pub job_tx: Sender<Job>,
    job_rx: Receiver<Job>,
    result_tx: Sender<ParsingResult>,
}

impl Orchestrator {
    pub fn new(pool_size: usize, result_tx: Sender<ParsingResult>) -> Self {
        let (done_tx, done_rx) = unbounded();
        let (connection_tx, connection_rx) = unbounded::<ConnectionInfo>();
        let (job_tx, job_rx) = unbounded();

        Orchestrator {
            pool_size,
            clients: Vec::new(),
            task_queue: Vec::new(),
            active: HashSet::new(),
            last_job_time: Vec::new(),
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
        loop {
            select! {
                recv(self.connection_rx) -> conn => {
                    self.create_client(conn.unwrap(), args);
                },
                recv(self.job_rx) -> job => {
                    let job = job.unwrap();
                    // println!("Server received job: {:?} {:?}", job.testcase, job.clients);

                    if job.parsers.len() == 0 {
                        self.queue(&job.testcase, None);
                    } else {
                        self.queue(&job.testcase, Some(job.parsers));
                    }

                    self.try_pop_queue();
                },
                recv(self.done_rx) -> res => {
                    let mut res = res.unwrap();
                    self.active.remove(&res.parser_id);

                    // println!("Server received result: {} {:?}", res.id, res.testcase);
                    if res.fuzzed {
                        res.parser_count = self.clients.len();
                        self.last_job_time[res.parser_id] = Instant::now();
                        self.result_tx.send(res).unwrap();
                    }

                    self.try_pop_queue();
                },
            }
        }
    }

    pub fn create_client(&mut self, conn: ConnectionInfo, args: &Args) {
        self.clients.push(Client::new(
            conn,
            self.done_tx.clone(),
            args,
            self.clients.len(),
        ));

        // let seeds = load_testcases();
        // let mut set = BTreeSet::new();
        //
        // for seed in seeds {
        //     set.insert(seed);
        // }

        self.task_queue.push(BTreeSet::new());
        self.last_job_time.push(Instant::now());
        self.try_pop_queue();
    }

    pub fn queue(&mut self, testcase: &TestCase, parsers: Option<Vec<String>>) {
        match parsers {
            Some(parsers) => {
                for parser in &parsers {
                    for i in 0..self.clients.len() {
                        if &self.clients[i].name == parser {
                            self.task_queue[i].insert(testcase.clone());
                            break;
                        }
                    }
                }
            }
            None => {
                for i in 0..self.clients.len() {
                    self.task_queue[i].insert(testcase.clone());
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

        // self.round_robin_i = i + 1;
        // while self.active.contains(&self.round_robin_i) {
        //     self.round_robin_i += 1;
        // }

        let mut indices = self
            .last_job_time
            .iter()
            .enumerate()
            .collect::<Vec<(usize, &Instant)>>();
        indices.sort_by(|a, b| a.1.cmp(&b.1));

        for (i, _) in &indices {
            // let i = (i + self.round_robin_i) % self.clients.len();

            // println!(
            //     "{} {} {:?}",
            //     i,
            //     self.active.contains(&i),
            //     self.task_queue[i].first()
            // );

            // Check if the client has crashed
            if self.clients[*i].handle.is_finished() {
                self.active.remove(i);
                continue;
            }

            if self.active.contains(&i) {
                continue;
            }

            if let Some(cur) = self.task_queue[*i].first() {
                if let Some(j) = min_i {
                    let best = self.task_queue[j].first().unwrap();
                    if cur.weight < best.weight {
                        min_i = Some(*i);
                    }
                } else {
                    min_i = Some(*i);
                }
            }
        }

        if let Some(i) = min_i {
            // for (i, t) in &indices {
            //     println!(
            //         "{} {} {}s ",
            //         i,
            //         self.clients[*i].name,
            //         t.elapsed().as_secs()
            //     );
            // }
            // println!("best: {} {:?}\n", i, self.task_queue[i].first());
            // println!();

            let testcase = self.task_queue[i].pop_first().unwrap();
            self.active.insert(i);
            self.clients[i].job_tx.send(testcase).unwrap();
        }
    }
}

#[derive(Debug)]
pub struct TimeData {
    pub duration: f64,
    pub testcase: TestCase,
    pub output: String,
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
        self.duration.total_cmp(&other.duration)
    }
}

impl PartialOrd for TimeData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
