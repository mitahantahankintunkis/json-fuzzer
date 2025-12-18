use std::env;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::process::exit;
use std::time::Instant;

use postgres::{Client, NoTls};
use rusqlite::Connection;

extern crate json;

const KEY_NOT_FOUND: &str = "KEY_NOT_FOUND";
const PARSE_ERROR: &str = "PARSE_ERROR";

fn parse_serde(data: &[u8], key: &str) -> String {
    match serde_json::from_slice::<serde_json::Value>(data) {
        Ok(parsed) => match parsed.get(key) {
            Some(q) => q.to_string(),
            None => String::from(KEY_NOT_FOUND),
        },
        Err(_) => String::from(PARSE_ERROR),
    }
}

fn parse_json(data: &[u8], key: &str) -> String {
    let parsed = json::parse(&String::from_utf8_lossy(&data).to_string());

    match parsed {
        Ok(parsed) => match parsed[key] {
            json::JsonValue::Null => String::from(KEY_NOT_FOUND),
            _ => json::stringify(parsed[key].clone()),
        },
        Err(_) => String::from(PARSE_ERROR),
    }
}

fn parse_sqlite(data: &[u8], key: &str, conn: &Connection) -> String {
    // conn.execute("UPDATE test SET x = ?1 WHERE id = 1", [data])
    //     .unwrap();
    let str = match String::from_utf8(data.to_vec()) {
        Ok(s) => s,
        Err(_) => return PARSE_ERROR.into(),
    };

    let query = conn.query_one(
        &format!(
            "SELECT json_type(x, '$.{key}'), json_extract(x, '$.{key}') FROM (SELECT '{json}' AS x)",
            key=key.replace("'", "''"),
            json=str.replace("'", "''"),
        ),
        [],
        |row| {
            let t: Option<String> = row.get(0)?;

            if t.is_none() {
                return Ok(KEY_NOT_FOUND.into());
            }

            let val = match t.unwrap().as_ref() {
                "null" => "null".into(),
                "true" => {
                    let val: bool = row.get(1)?;
                    val.to_string()
                }
                "false" => {
                    let val: bool = row.get(1)?;
                    val.to_string()
                }
                "integer" => {
                    let val: i64 = row.get(1)?;
                    val.to_string()
                }
                "real" => {
                    let val: f64 = row.get(1)?;
                    val.to_string()
                }
                "text" => {
                    let val: String = row.get(1)?;
                    format!("\"{}\"", val)
                }
                "array" => {
                    let val: String = row.get(1)?;
                    val
                }
                "object" => {
                    let val: String = row.get(1)?;
                    val
                }
                _ => PARSE_ERROR.into(),
            };

            Ok(val)
        },
    );

    match query {
        Ok(val) => val,
        Err(_) => String::from(PARSE_ERROR),
    }
}

fn parse_postgres(data: &[u8], key: &str, client: &mut Client, _parser_number: i32) -> String {
    let str = match String::from_utf8(data.to_vec()) {
        Ok(s) => s,
        Err(_) => return PARSE_ERROR.into(),
    };

    // let res = client.execute(
    //     &format!(
    //         "UPDATE test SET x = '{}' WHERE id = {}",
    //         str.replace("'", "\\'"),
    //         parser_number
    //     ),
    //     &[],
    // );

    // if let Err(_) = res {
    //     return PARSE_ERROR.into();
    // }

    let query = client.query_one(
        &format!(
            "SELECT x -> '{}' FROM (SELECT '{}'::jsonb AS x)",
            key.replace("'", "''"),
            str.replace("'", "''")
        ),
        &[],
    ); // &[&key, &str])
       // let query = client.query_one(
       //     "SELECT x -> $1 FROM test WHERE id = $2",
       //     &[&key, &parser_number],
       // );

    match query {
        Ok(row) => {
            let val: Result<serde_json::Value, postgres::Error> = row.try_get(0);
            if let Ok(val) = val {
                if val.is_i64() {
                    if let Some(val) = val.as_i64() {
                        return format!("{}", val);
                    }
                }
                if val.is_f64() {
                    if let Some(val) = val.as_f64() {
                        return format!("{}", val);
                    }
                }
                if val.is_string() {
                    if let Some(val) = val.as_str() {
                        return format!("\"{}\"", val);
                    }
                }
            }

            PARSE_ERROR.into()
        }

        Err(_) => PARSE_ERROR.into(),
    }
}

fn main() -> std::io::Result<()> {
    let mut stream: UnixStream;
    let args: Vec<String> = env::args().collect();

    let parser_number = if args.len() == 2 {
        args[1].parse::<usize>().unwrap()
    } else {
        0
    };

    let sqlite_conn = Connection::open_in_memory().unwrap();
    sqlite_conn
        .execute("CREATE TABLE test(id INTEGER PRIMARY KEY, x JSON)", [])
        .unwrap();

    let mut postgres_client = Client::connect("host=postgres user=postgres", NoTls)
        .expect("Rust: Could not connect to postgres");
    postgres_client
        .batch_execute("CREATE TABLE IF NOT EXISTS test(id INTEGER PRIMARY KEY, x JSONB)")
        .unwrap();

    sqlite_conn
        .execute("INSERT INTO test VALUES (1, '{}')", [])
        .unwrap();
    postgres_client
        .execute(
            "INSERT INTO test VALUES ($1, '{}') ON CONFLICT DO NOTHING",
            &[&(parser_number as i32)],
        )
        .unwrap();

    let name = match parser_number {
        0 => "rust_serde",
        1 => "rust_json",
        2 => "sqlite3",
        3 => "postgres",
        _ => exit(1),
    };

    loop {
        if let Ok(s) = UnixStream::connect("/tmp/fuzzer.sock") {
            stream = s;
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let mut info_buf: [u8; 65] = [0; 65];
    info_buf[0..name.len()].copy_from_slice(name.as_bytes());

    stream
        .write_all(&info_buf)
        .expect("Rust: Could not write name");

    let mut read_buffer: Box<Vec<u8>> = Box::new(vec![0u8; 1000_000]);
    let mut write_buffer: Box<Vec<u8>> = Box::new(vec![0u8; 1000_000]);

    loop {
        let mut header = [0u8; 9];

        if stream.read_exact(&mut header).is_err() {
            return Ok(());
        }

        let input_buffer_size = u32::from_le_bytes(header[0..4].try_into().unwrap()) as usize;
        // let datatype = header[4];
        let key_len = u32::from_le_bytes(header[5..9].try_into().unwrap()) as usize;

        stream.read_exact(&mut read_buffer[0..key_len]).unwrap();
        let key = String::from_utf8(read_buffer[0..key_len].to_vec())
            .expect("Rust client: Key should be valid UTF-8");

        match stream.read_exact(&mut read_buffer[0..input_buffer_size]) {
            Ok(()) => {
                let mut read_offset = 0;
                let mut write_offset: usize = 4;

                while read_offset < input_buffer_size {
                    let json_size = u16::from_le_bytes(
                        read_buffer[read_offset..read_offset + 2]
                            .try_into()
                            .unwrap(),
                    ) as usize;
                    read_offset += 2;

                    let data: &[u8] = &read_buffer[read_offset..read_offset + json_size];
                    read_offset += json_size;

                    let start = Instant::now();
                    // let message = parse_fn(data, &key, &params);
                    let message = match parser_number {
                        0 => parse_serde(data, &key),
                        1 => parse_json(data, &key),
                        2 => parse_sqlite(data, &key, &sqlite_conn),
                        3 => parse_postgres(data, &key, &mut postgres_client, parser_number as i32),
                        _ => exit(1),
                    };

                    let ns = (start.elapsed().as_nanos() / 10) as u32;

                    write_buffer[write_offset..write_offset + 4].copy_from_slice(&ns.to_le_bytes());
                    write_offset += 4;

                    write_buffer[write_offset..write_offset + 2]
                        .copy_from_slice(&(message.len() as u16).to_le_bytes());
                    write_offset += 2;

                    write_buffer[write_offset..write_offset + message.len()]
                        .copy_from_slice(message.as_bytes());
                    write_offset += message.len();
                }

                write_buffer[0..4]
                    .copy_from_slice(&u32::try_from(write_offset - 4).unwrap().to_le_bytes());

                stream
                    .write_all(&write_buffer[0..write_offset])
                    .expect("Write error");
            }
            Err(_e) => {
                return Ok(());
            }
        }
    }
}
