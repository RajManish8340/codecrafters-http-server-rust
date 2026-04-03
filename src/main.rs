use std::io::{BufRead, BufReader, Write};
#[allow(unused_imports)]
use std::net::TcpListener;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let reader = BufReader::new(&_stream);
                let request_line = reader.lines().next().unwrap().unwrap();
                let path = request_line.split_whitespace().nth(1);
                println!("accepted new connection");
                if path == Some("/") {
                    _stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
                } else {
                    _stream
                        .write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
                        .unwrap();
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
