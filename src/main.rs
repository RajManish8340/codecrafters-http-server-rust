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
                std::thread::spawn(move || {
                    let reader = BufReader::new(&_stream);
                    let mut lines = reader.lines();
                    let request_line = lines.next().unwrap().unwrap();
                    let path = request_line.split_whitespace().nth(1).unwrap();
                    let headers = lines
                        .filter_map(|x| x.ok())
                        .find(|x| x.starts_with("User-Agent"))
                        .unwrap();
                    println!("accepted new connection");

                    match path {
                        "/" => _stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap(),
                        p if p.starts_with("/echo") => {
                            let content = p.strip_prefix("/echo/").unwrap();
                            let response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                content.len(),
                                content
                            );
                            _stream.write_all(response.as_bytes()).unwrap();
                        }

                        p if p.starts_with("/user-agent") => {
                            let content = headers.split(": ").nth(1).unwrap();
                            let response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                content.len(),
                                content
                            );
                            _stream.write_all(response.as_bytes()).unwrap()
                        }

                        _ => {
                            _stream
                                .write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
                                .unwrap();
                        }
                    }
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
