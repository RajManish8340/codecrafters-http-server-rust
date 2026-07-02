#[allow(unused_imports)]
use std::net::TcpListener;
use std::{
    env::{self},
    fs::{self, File},
    io::{BufRead, BufReader, Read, Write},
};

use flate2::{Compression, write::GzEncoder};

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
    println!("accepted new connection");
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                std::thread::spawn(move || {
                    let mut reader = BufReader::new(_stream.try_clone().unwrap());
                    loop {
                        let mut request_line = String::new();
                        reader.read_line(&mut request_line).unwrap();

                        let method = request_line.split_whitespace().nth(0).unwrap();
                        let path = request_line.split_whitespace().nth(1).unwrap();

                        let mut headers: Vec<String> = Vec::new();
                        loop {
                            let mut line = String::new();
                            reader.read_line(&mut line).unwrap();
                            let line = line.trim_end().to_string();
                            if line.is_empty() {
                                break;
                            }
                            headers.push(line);
                        }
                        let closed_header = headers
                            .iter()
                            .find(|c| c.starts_with("Connection"))
                            .and_then(|c| c.split(": ").nth(1));
                        let should_close = closed_header == Some("close");
                        let connection_header = if should_close {
                            "Connection: close\r\n"
                        } else {
                            ""
                        };

                        match path {
                            "/" => {
                                let response =
                                    format!("HTTP/1.1 200 OK\r\n{}\r\n", connection_header);
                                _stream.write_all(response.as_bytes()).unwrap();
                            }
                            p if p.starts_with("/echo") => {
                                let content = p.strip_prefix("/echo/").unwrap();
                                let mut content_bytes: Vec<u8> = content.bytes().collect();
                                let encodings = headers
                                    .iter()
                                    .find(|h| h.starts_with("Accept-Encoding"))
                                    .and_then(|h| h.split(": ").nth(1));

                                if encodings.is_some() {
                                    let mut vec_encoding = encodings.unwrap().split(", ");
                                    let gzip_encoding =
                                        vec_encoding.find(|g| g.starts_with("gzip"));

                                    if gzip_encoding == Some("gzip") {
                                        let mut encoder =
                                            GzEncoder::new(Vec::new(), Compression::default());
                                        encoder.write_all(content_bytes.as_slice()).unwrap();
                                        content_bytes = encoder.finish().unwrap();
                                        let response = format!(
                                            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n{}Content-Encoding: {}\r\nContent-Length: {}\r\n\r\n",
                                            connection_header,
                                            gzip_encoding.unwrap(),
                                            content_bytes.len(),
                                        );
                                        _stream.write_all(response.as_bytes()).unwrap();
                                        _stream.write_all(content_bytes.as_slice()).unwrap();
                                    } else {
                                        let response = format!(
                                            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n{}Content-Length: {}\r\n\r\n{}",
                                            connection_header,
                                            content.len(),
                                            content
                                        );
                                        _stream.write_all(response.as_bytes()).unwrap();
                                    }
                                } else {
                                    let response = format!(
                                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n{}Content-Length: {}\r\n\r\n{}",
                                        connection_header,
                                        content.len(),
                                        content
                                    );
                                    _stream.write_all(response.as_bytes()).unwrap();
                                }
                            }

                            p if p.starts_with("/user-agent") => {
                                let header = headers
                                    .iter()
                                    .find(|x| x.starts_with("User-Agent"))
                                    .unwrap();
                                let content = header.split(": ").nth(1).unwrap();
                                let response = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n{}Content-Length: {}\r\n\r\n{}",
                                    connection_header,
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

                                if method == "GET" {
                                    let file_content = fs::read(&dir);
                                    match file_content {
                                        Ok(fc) => {
                                            let header_response = format!(
                                                "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\n{}Content-Length: {}\r\n\r\n",
                                                connection_header,
                                                fc.len(),
                                            );
                                            _stream.write_all(header_response.as_bytes()).unwrap();
                                            _stream.write_all(&fc).unwrap()
                                        }
                                        Err(..) => _stream
                                            .write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
                                            .unwrap(),
                                    }
                                } else if method == "POST" {
                                    let file = File::create(&dir);
                                    let header = headers
                                        .iter()
                                        .find(|h| h.starts_with("Content-Length"))
                                        .unwrap();
                                    let content_length = header
                                        .split(": ")
                                        .nth(1)
                                        .unwrap()
                                        .parse::<usize>()
                                        .unwrap();
                                    match file {
                                        Ok(mut f) => {
                                            let mut body = vec![0u8; content_length];
                                            reader.read_exact(&mut body).unwrap();
                                            f.write_all(&body).unwrap();

                                            _stream
                                                .write_all(b"HTTP/1.1 201 Created \r\n\r\n")
                                                .unwrap();
                                        }
                                        Err(..) => {
                                            _stream
                                                .write_all(
                                                    b"HTTP/1.1 500 Internal Server Error\r\n\r\n",
                                                )
                                                .unwrap();
                                        }
                                    }
                                }
                            }

                            _ => {
                                _stream
                                    .write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
                                    .unwrap();
                            }
                        }

                        if should_close {
                            break;
                        };
                    }
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
