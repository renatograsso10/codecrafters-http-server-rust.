use std::net::TcpListener;
use std::io::{Read, Write};
use std::env;
use std::fs::File;

fn main() {
    let args: Vec<String> = env::args().collect();
    let directory = if args.len() >= 3 && args[1] == "--directory" {
        args[2].clone()
    } else {
        ".".to_string() // Default to current directory
    };

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let directory = directory.clone();
                std::thread::spawn(move || {
                    handle_connection(stream, &directory);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: std::net::TcpStream, directory: &str) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..]);

    let response = if request.starts_with("GET /files/") {
        let filename = &request[11..request.find("HTTP/1.1").unwrap() - 1];
        let filepath = format!("{}/{}", directory, filename);
        match File::open(filepath) {
            Ok(mut file) => {
                let mut contents = Vec::new();
                file.read_to_end(&mut contents).unwrap();
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
                    contents.len(),
                    String::from_utf8_lossy(&contents)
                )
            }
            Err(_) => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
        }
    } else if request.starts_with("GET /echo/") {
        let echo_str = &request[10..request.find("HTTP/1.1").unwrap() - 1];
        let user_agent = request.lines()
            .find(|line| line.to_lowercase().starts_with("accept-encoding:"))
            .map(|line| line[16..].trim())
            .unwrap_or("");
        let encodings: Vec<&str> = user_agent.split(',').map(|s| s.trim()).collect();
        let response = if encodings.contains(&"gzip") {
            format!(
                "HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                echo_str.len(),
                echo_str
            )
        } else {
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                echo_str.len(),
                echo_str
            )
        };
        response
    } else if request.starts_with("GET /user-agent") {
        let user_agent = request.lines()
            .find(|line| line.starts_with("User-Agent:"))
            .map(|line| line[12..].trim())
            .unwrap_or("");
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            user_agent.len(),
            user_agent
        )
    } else if request.starts_with("GET / ") {
        "HTTP/1.1 200 OK\r\n\r\n".to_string()
    } else if request.starts_with("POST /files/") {
        let filename = &request[12..request.find("HTTP/1.1").unwrap() - 1];
        let filepath = format!("{}/{}", directory, filename);
        let body = request.split("\r\n\r\n").nth(1).unwrap_or("");
        match File::create(filepath) {
            Ok(mut file) => {
                file.write_all(body.trim_end_matches('\0').as_bytes()).unwrap();
                "HTTP/1.1 201 Created\r\n\r\n".to_string()
            }
            Err(_) => "HTTP/1.1 500 Internal Server Error\r\n\r\n".to_string(),
        }
    } else {
        "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
    };

    stream.write_all(response.as_bytes()).unwrap();
}
