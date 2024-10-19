use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};
use std::collections::HashMap;
use bytes::Bytes;

use mini_redis::threadpool::ThreadPool;

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let pool = ThreadPool::new(4);
    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let db = Arc::clone(&db);

        pool.execute(|| {
            handle_client(stream, db);
        });
    }
}

fn handle_client(mut stream: TcpStream, db: Db) {
    let mut buffer = [0; 512];
    while match stream.read(&mut buffer) {
        Ok(size) if size > 0 => {
            let command = String::from_utf8_lossy(&buffer[..size]);
            let response = process_command(&command, Arc::clone(&db));
            stream.write(response.as_bytes()).unwrap();
            true
        },
        _ => false,
    } {}
}

fn process_command(command: &str, db: Db) -> String {
    let parts: Vec<String> = command.trim().split_whitespace().map(|s| s.to_string()).collect();
    let mut parts_iter = parts.into_iter();
    let cmd = parts_iter.next().unwrap();

    match cmd.as_str() {
        "SET" => {
            if let (Some(key), Some(value)) = (parts_iter.next(), parts_iter.next()) {
                let mut db = db.lock().unwrap();
                db.insert(key.to_string(), Bytes::from(value));
                "+OK\r\n".to_string()
            } else {
                "-ERR wrong number of arguments\r\n".to_string()
            }
        }
        "GET" => {
            if let Some(key) = parts_iter.next() {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(&key) {
                    format!("${}\r\n{}\r\n", value.len(), String::from_utf8_lossy(&value))
                } else {
                    "$-1\r\n".to_string()
                }
            } else {
                "-ERR wrong number of arguments\r\n".to_string()
            }
        }
        _ => "-ERR unknown command\r\n".to_string(),
    }
}

