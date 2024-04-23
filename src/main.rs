// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    //match listener.accept() {
    //  Ok((_socket, addr)) => println!("new client: {addr:?}"),
    //  Err(e) => println!("couldn't get client: {e:?}"),
    //}

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                thread::spawn(move || handle_client(_stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

pub fn handle_client(mut stream: TcpStream) {
    loop {
        let mut buf = [0; 512];

        let bytes_read = stream.read(&mut buf).expect("failed to read from client");

        if bytes_read == 0 {
            return;
        }

        let cmd = buf[2] as char;

        match cmd {
            'p' => {
                stream.write_all("+PONG\r\n".as_bytes()).unwrap();
            }
            _ => {
                stream.write_all("Invalid command\r\n".as_bytes()).unwrap();
            }
        }
    }
}
