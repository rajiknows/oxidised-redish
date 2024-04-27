use std::{
    collections::HashMap,
    fmt::format,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let mut memory_map = HashMap::new();

    for stream in listener.incoming() {
        thread::spawn(move || match stream {
            Ok(mut stream) => {
                handle_connection(&mut stream, &mut memory_map);
            }
            Err(e) => {
                println!("error {}", e);
            }
        });
    }
}

fn handle_connection(stream: &mut TcpStream, memory_map: &mut HashMap<Option<&str>, Option<&str>>) {
    loop {
        let mut buf = [0; 512];
        let _bytes_read = stream.read(&mut buf);

        let byte_slice = std::str::from_utf8(&buf[..]).expect("could not convert byte to slice");
        redis_parser(byte_slice, stream, &mut memory_map);

        /*match bytes_read {
            Ok(n) => {
                if n == 0 {
                    break;
                }

                let res = stream.write_all(b"+PONG\r\n");
                match res {
                    Ok(_) => {}
                    Err(e) => {
                        println!("error {}", e);
                    }
                }
            }
            Err(e) => {
                println!("error {}", e);
            }
        }*/
    }
}

fn redis_parser(
    byte_slice: &str,
    stream: &mut TcpStream,
    memory_map: &mut HashMap<Option<&str>, Option<&str>>,
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
                        instruction_array.push(arg);
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
                memory_map.insert(
                    instruction_array.get(1).cloned(),
                    instruction_array.get(2).cloned(),
                );
            }
            "GET" => match memory_map.get(instruction_array.get(1).cloned()) {
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
