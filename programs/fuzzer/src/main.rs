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
mod compression;
mod fuzz;
mod okfuzz;
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
    #[arg(short, long)]
    display_results: Option<Vec<String>>,

    #[arg(
        short,
        long,
        default_value_t = 8,
        help = "How many threads are used for JSON parsing. Recommended two \
        less than the maximum supported by the system."
    )]
    workers: usize,

    #[arg(long, default_value_t = 1 << 16, help = "Maximum size of the batched test cases send to the clients")]
    batch_size: usize,

    #[arg(
        short,
        long,
        action,
        help = "Send one payload at a time. Useful for detecting crashes"
    )]
    no_batching: bool,

    #[arg(
        long,
        help = "Measure parsing duration of the specified parser 10 million times"
    )]
    measure_timing: Option<String>,

    #[arg(
        long,
        action,
        help = "Measure how different batch sizes affect parsing performance"
    )]
    measure_batch_size_timings: bool,

    #[arg(
        long,
        help = "Test case used to measure parsing durations. Default '{\"q\":2,\"q\":3}'. Uses key 'q'",
        default_value = r#"{"q":2,"q":3}"#
    )]
    measure_testcase: String,

    #[arg(long, help = "Measure timing only once with more accuracy")]
    measure_timing_once: bool,

    #[arg(
        long,
        help = "How many times larger does a parsing time have to be related to its parent value to be logged",
        default_value_t = 2.0
    )]
    dos_ratio_treshold: f64,

    #[arg(long, help = "Bias value alpha", default_value_t = 1.0)]
    alpha: f64,

    #[arg(long, help = "Bias value beta", default_value_t = 2.0)]
    beta: f64,

    #[arg(long, help = "Bias value gamma", default_value_t = 3.0)]
    gamma: f64,

    #[arg(
        long,
        help = "DoS large test case array size",
        default_value_t = 1_000_000
    )]
    dos_testcase_array_size: usize,

    #[arg(long, default_value_t = 10_000)]
    testcase_measurement_repeats: usize,
}

fn main() -> color_eyre::Result<()> {
    let args = Args::parse();

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
    let (connection_tx, connection_rx) = unbounded::<ConnectionInfo>();
    let mut orchestrator = Orchestrator::new(
        args.workers,
        args.batch_size,
        connection_rx,
        result_tx.clone(),
    );
    let mut analyzer = Analyzer::new(args.clone(), result_rx, orchestrator.job_tx.clone());

    // accept connections and process them serially
    // Unix domain sockets
    let unix_conn_tx = connection_tx.clone();
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
    let tcp_conn_tx = connection_tx.clone();
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

    if args.measure_timing.is_some() {
        orchestrator.measure_timing(&args);
    } else if args.measure_batch_size_timings {
        orchestrator.measure_batching(&args);
    } else {
        thread::spawn(move || {
            orchestrator.join();
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
                terminal.draw(|frame| {
                    frame.render_widget(&tui, frame.area());
                })?;
            }
        }

        ratatui::restore();

        // TODO - join threads
        analyzer_handle.join().unwrap();
    }

    Ok(())
}
