use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        thread::spawn(move || match stream {
            Ok(mut _stream) => {
                handle_connection(_stream);
            }
            Err(e) => {
                println!("error {}", e);
            }
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    loop {
        let buf = &mut [0; 512];
        let bytes_read = stream.read(buf);

        let recieved_string = String::from_utf8_lossy(buf);
        println!("the string recieved is {}", recieved_string);

        match bytes_read {
            Ok(n) => {
                if n == 0 {
                    break;
                }

                let res = stream.write_all("+PONG\r\n".as_bytes());
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
        }
    }
}
