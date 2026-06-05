//            ______________
//      ,===:'.,            `-._
//           `:.`---.__         `-._
//             `:.     `--.         `.
//               \.        `.         `.
//       (,,(,    \.         `.   ____,-`.,
//    (,'     `/   \.   ,--.___`.'
//,  ,'  ,--.  `,   \.;'         `
// `{D, {    \  :    \;
//   V,,'    /  /    //
//   j;;    /  ,' ,-//.    ,---.      ,
//   \;'   /  ,' /  _  \  /  _  \   ,'/
//         \   `'  / \  `'  / \  `.' /
//          `.___,'   `.__,'   `.__,'
//
//           Here be dragons
extern crate libc;

mod analyze;
mod comprehensive_fuzzer;
mod compression;
mod fuzz;
mod radamsa_wrapper;
mod server;
mod tui;
mod util;

use crate::analyze::Analyzer;
use crate::server::{CombinedStream, ConnectionInfo, Orchestrator, ParsingResult};
use crate::tui::ClientStatus;
use clap::Parser;
use crossbeam_channel::unbounded;
use std::fs::{self, exists, remove_file};
use std::net::TcpListener;
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixListener;
use std::thread;
use std::time::Duration;

const SOCK_FILE: &str = "/tmp/fuzzer.sock";

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    // #[arg(short, long, action)]
    // analyze: bool,
    #[arg(short, long)]
    display_results: Option<Vec<String>>,

    #[arg(
        short,
        long,
        default_value = "8",
        help = "How many threads are used for JSON parsing. Recommended two \
        less than the maximum supported by the system."
    )]
    workers: usize,

    #[arg(
        short,
        long,
        action,
        help = "Send one JSON text at a time. Useful for detecting crashes"
    )]
    no_batching: bool,
}

// fn debug(paths: &[String]) {
//     for path in paths {
//         let file_name = path.split("/").last().unwrap();
//         let split: Vec<&str> = file_name.split(';').collect();
//
//         if split.len() != 4 {
//             eprintln!("Malformed filename '{}'", file_name);
//             continue;
//         }
//
//         let file_parser_name = &split[0];
//         let file_fuzzer_name = &split[1];
//         let file_json_id = &split[2];
//
//         for testcase in &load_testcases() {
//             for fuzzer in &mut create_fuzzers(&testcase) {
//                 let digest = format!("{:x}", md5::compute(&testcase.json))[0..8].to_string();
//
//                 if *file_fuzzer_name == fuzzer.id() && *file_json_id == digest {
//                     println!("\n{} {}:", file_parser_name, testcase.json);
//
//                     let bytes = std::fs::read(path).expect("Could not read file");
//                     let mut decoder = Decoder::new(Box::new(bytes));
//
//                     // Pop parser name
//                     decoder.next_message();
//
//                     loop {
//                         let mut buf = vec![0u8; 1 << 16];
//                         let n = fuzzer.copy_to_slice(&mut buf).unwrap();
//                         let json = buf[0..n]
//                             .iter()
//                             .map(|c| byte_to_string(*c))
//                             .collect::<Vec<String>>()
//                             .join("");
//
//                         let output = decoder.next_message().expect("Not enough outputs");
//
//                         println!("{} -> {}", json, output);
//
//                         if fuzzer.advance().is_err() {
//                             break;
//                         }
//                     }
//                     break;
//                 }
//             }
//         }
//     }
// }

fn main() -> color_eyre::Result<()> {
    let args = Args::parse();

    // if let Some(paths) = args.display_results {
    //     debug(&paths);
    //     return Ok(());
    // }

    if exists(SOCK_FILE).unwrap() {
        remove_file(SOCK_FILE).expect(&format!(
            "Could not remove previous sock ({}) file",
            SOCK_FILE
        ));
    }

    if !fs::exists("analyzed/").expect("Could not check if analyzed directory exists") {
        fs::create_dir("analyzed").expect("Could not create analyzed/");
    }

    let (result_tx, result_rx) = unbounded::<ParsingResult>();
    let (tui_tx, tui_rx) = unbounded::<ClientStatus>();
    let mut orchestrator = Orchestrator::new(args.workers, result_tx.clone());
    let mut analyzer = Analyzer::new(result_rx, orchestrator.job_tx.clone());

    // accept connections and process them serially
    // Unix domain sockets
    let unix_conn_tx = orchestrator.connection_tx.clone();
    let unix_tui_tx = tui_tx.clone();
    thread::spawn(move || {
        let unix_listener = UnixListener::bind(SOCK_FILE).unwrap();

        // Increase buffer size
        unsafe {
            let optval: libc::c_int = 10_000_000;
            let ret = libc::setsockopt(
                unix_listener.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_SNDBUF,
                &optval as *const _ as *const libc::c_void,
                std::mem::size_of_val(&optval) as libc::socklen_t,
            );
            if ret != 0 {
                eprintln!(
                    "Could not increase socket buffer size: {}",
                    std::io::Error::last_os_error()
                );
                return;
            }
        }

        for stream in unix_listener.incoming() {
            let stream = stream.unwrap();
            unix_conn_tx
                .send(ConnectionInfo {
                    stream: CombinedStream::Unix(stream),
                    tui_tx: unix_tui_tx.clone(),
                })
                .unwrap();
        }
    });

    // TCP sockets
    let tcp_conn_tx = orchestrator.connection_tx.clone();
    let tcp_tui_tx = tui_tx.clone();
    thread::spawn(move || {
        let tcp_listener = TcpListener::bind("127.0.0.1:5000").unwrap();

        for stream in tcp_listener.incoming() {
            let stream = stream.unwrap();
            if let Err(e) = stream.set_nodelay(true) {
                eprintln!("Could not set NODELAY: {}", e);
            }

            tcp_conn_tx
                .send(ConnectionInfo {
                    stream: CombinedStream::TCP(stream),
                    tui_tx: tcp_tui_tx.clone(),
                })
                .unwrap();
        }
    });

    thread::spawn(move || {
        orchestrator.join(&args);
    });

    let tui_dos_rx = analyzer.dos_rx.clone();
    let tui_discrepancy_rx = analyzer.discrepancy_rx.clone();

    // Starts the fuzzing process
    let analyzer_handle = thread::spawn(move || {
        // Wait for all clients to connect.
        // Seed cases will not be loaded for clients that connect after this.
        thread::sleep(Duration::from_secs(5));
        analyzer.analyze();
    });

    color_eyre::install()?;
    let mut tui = tui::App::new();
    let mut terminal = ratatui::init();

    while !tui.quit {
        if let Err(e) = tui.handle_events() {
            eprintln!("{}", e);
            break;
        }

        while let Ok(status) = tui_rx.try_recv() {
            tui.push_status(status);
        }

        while let Ok(dos) = tui_dos_rx.try_recv() {
            tui.push_dos(dos);
        }

        while let Ok(discrepancy) = tui_discrepancy_rx.try_recv() {
            tui.push_discrepancy(discrepancy);
        }

        if tui.updated {
            // tui.updated = false;

            terminal.draw(|frame| {
                frame.render_widget(&tui, frame.area());
            })?;
        }
    }

    ratatui::restore();

    // TODO - join threads
    analyzer_handle.join().unwrap();

    Ok(())
}
