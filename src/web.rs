use crate::Asset;
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

pub fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);

    let first_line = match buf_reader.lines().next() {
        Some(Ok(line)) => line,
        _ => return,
    };

    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }

    let raw_path = parts[1];

    let path = if raw_path == "/" {
        "index.html"
    } else {
        &raw_path[1..]
    };

    if let Some(file) = Asset::get(path) {
        let content_type = match path.split('.').last() {
            Some("js") => "application/javascript",
            Some("wasm") => "application/wasm",
            Some("css") => "text/css",
            Some("png") => "image/png",
            _ => "text/html",
        };

        let response_headers = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            content_type,
            file.data.len()
        );

        stream.write_all(response_headers.as_bytes()).unwrap();
        stream.write_all(&file.data).unwrap();
        stream.flush().unwrap();
    } else {
        let response = "HTTP/1.1 404 NOT FOUND\r\nContent-Length: 0\r\n\r\n";
        stream.write_all(response.as_bytes()).unwrap();
    }
}
