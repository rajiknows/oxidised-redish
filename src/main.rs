use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        thread::spawn(move || match stream {
            Ok(mut stream) => {
                handle_connection(&mut stream);
            }
            Err(e) => {
                println!("error {}", e);
            }
        });
    }
}

fn handle_connection(stream: &mut TcpStream) {
    let mut memory_map = HashMap::new();

    loop {
        let mut buf = [0; 512];
        let _bytes_read = stream.read(&mut buf);

        let byte_slice = std::str::from_utf8(&buf[..]).expect("could not convert byte to slice");
        redis_parser(byte_slice, stream, &mut memory_map);
    }
}

fn redis_parser(
    byte_slice: &str,
    stream: &mut TcpStream,
    memory_map: &mut HashMap<Option<String>, Option<String>>,
) {
    let mut instruction_array = Vec::new();

    if byte_slice.starts_with('*') {
        if let Some(index) = byte_slice.find("\r\n") {
            let count: usize = byte_slice[1..index].parse().expect("Invalid RESP format");

            let mut remaining_bytes = &byte_slice[index + 2..];

            for _ in 0..count {
                if remaining_bytes.starts_with('$') {
                    if let Some(index) = remaining_bytes.find("\r\n") {
                        let arg_len: usize = remaining_bytes[1..index]
                            .parse()
                            .expect("Invalid RESP format");
                        let arg = &remaining_bytes[index + 2..index + 2 + arg_len];
                        instruction_array.push(arg.to_string());
                        remaining_bytes = &remaining_bytes[index + 2 + arg_len + 2..];
                    }
                }
            }
        }
    }

    // Use instruction_array as needed
    if let Some(cmd) = instruction_array.first() {
        let cmd_uppercase = cmd.to_uppercase();
        match cmd_uppercase.as_str() {
            "PING" => {
                let _ = stream.write_all(b"+PONG\r\n");
            }
            "ECHO" => {
                if let Some(arg) = instruction_array.get(1) {
                    let _ = stream.write_all(format!("+{}\r\n", arg).as_bytes());
                } else {
                    let _ = stream.write_all(b"-ERR Missing argument for ECHO\r\n");
                }
            }
            "SET" => {
                if let (Some(key), Some(value)) =
                    (instruction_array.get(1), instruction_array.get(2))
                {
                    memory_map.insert(Some(key.to_string()), Some(value.to_string()));
                    let _ = stream.write_all("+OK\r\n".as_bytes());
                } else {
                    let _ = stream.write_all(b"-ERR Missing arguments for SET\r\n");
                }
            }
            "GET" => match memory_map.get(&instruction_array.get(1).cloned()) {
                Some(Some(val)) => {
                    let _ = stream.write_all(format!("+{}\r\n", val).as_bytes());
                }
                Some(None) => {
                    let _ = stream.write_all(b"-ERR Missing value for GET\r\n");
                }
                None => {
                    let _ = stream.write_all(b"-ERR Key not found for GET\r\n");
                }
            },
            _ => {
                let _ = stream.write_all(b"-ERR Unknown command\r\n");
            }
        }
    }
}
