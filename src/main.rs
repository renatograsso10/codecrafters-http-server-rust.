use std::net::TcpListener;
use std::io::{Read, Write};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                
                let mut buffer = [0; 512];
                stream.read(&mut buffer).unwrap();
                let request = String::from_utf8_lossy(&buffer[..]);
                
                let response = if request.starts_with("GET /echo/") {
                    let echo_str = &request[10..request.find("HTTP/1.1").unwrap() - 1];
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                        echo_str.len(),
                        echo_str
                    )
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
                } else {
                    "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
                };
                
                stream.write_all(response.as_bytes()).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
