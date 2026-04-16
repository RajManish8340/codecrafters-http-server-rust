#[allow(unused_imports)]
use std::net::TcpListener;
use std::{
    env::{self},
    fs,
    io::{BufRead, BufReader, Write},
};

fn get_dir_arg() -> Option<String> {
    let args: Vec<String> = env::args().collect();
    for i in 0..=args.len() - 1 {
        if args[i] == "--directory" && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
    }
    return None;
}
fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                std::thread::spawn(move || {
                    let reader = BufReader::new(&_stream);
                    let mut lines = reader.lines();
                    let request_line = lines.next().unwrap().unwrap();
                    let path = request_line.split_whitespace().nth(1).unwrap();
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
                            let headers = lines
                                .filter_map(|x| x.ok())
                                .find(|x| x.starts_with("User-Agent"))
                                .unwrap();
                            let content = headers.split(": ").nth(1).unwrap();
                            let response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                content.len(),
                                content
                            );
                            _stream.write_all(response.as_bytes()).unwrap()
                        }

                        p if p.starts_with("/files/") => {
                            let base_dir = get_dir_arg().unwrap();
                            let file_name = p.strip_prefix("/files/").unwrap();
                            let mut dir = base_dir.clone();
                            dir.push_str(file_name);
                            let file_content = fs::read(&dir);

                            match file_content {
                                Ok(fc) => {
                                    let header_response = format!(
                                        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
                                        fc.len(),
                                    );
                                    _stream.write_all(header_response.as_bytes()).unwrap();
                                    _stream.write_all(&fc).unwrap()
                                }
                                Err(..) => _stream
                                    .write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
                                    .unwrap(),
                            }
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
