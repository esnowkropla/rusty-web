extern crate regex;

use regex::Regex;
use std::collections::HashMap;
use std::io::{Read};
use std::net::{TcpStream, TcpListener};

#[derive(Debug)]
pub struct Request<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub version: &'a str,
    pub headers: HashMap<&'a str, &'a str>,
    pub body: Option<&'a str>,
}

impl<'a> Request<'a> {
    fn new(buff: &'a str) -> Request {
        let request_line = Request::parse_request_line(buff);
        Request {
            method: request_line.0,
            path: request_line.1,
            version: request_line.2,
            headers: Request::parse_headers(buff),
            body: Request::parse_body(buff)
        }
    }

    fn parse_request_line(request: &'a str) -> (&'a str, &'a str, &'a str) {
        let re = Regex::new("(.*) (.*) (.*)\r\n").unwrap();
        let captures = re.captures(request).unwrap();
        (captures.get(1).unwrap().as_str(),
         captures.get(2).unwrap().as_str(),
         captures.get(3).unwrap().as_str())
    }

    fn parse_headers(request: &'a str) -> HashMap<&'a str, &'a str> {
        let mut headers = HashMap::new();
        let re = Regex::new(r"(.*): +(.*)").unwrap();
        for line in request.lines() {
            if let Some(capture) = re.captures(line) {
                headers.insert(capture.get(1).unwrap().as_str(),
                               capture.get(2).unwrap().as_str());
            }
        }
        headers
    }

    fn parse_body(request: &'a str) -> Option<&'a str> {
        let re = Regex::new("\r\n\r\n(.*)").unwrap();
        match re.captures(request) {
            Some(capture) => capture.get(1).map(|x| x.as_str()),
            None => None
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buff = String::new();
    stream.read_to_string(&mut buff).unwrap();
    let request = Request::new(&buff);
    println!("{:?}", buff);
    println!("{:?}", request);
}

pub fn server() {
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();

    for stream in listener.incoming() {
        handle_connection(stream.unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    use std::io::Write;

    #[test]
    fn test_server() {
        thread::spawn(move || {server();});
        for _ in 1..5 {
            if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8000") {
                stream.write(b"Testing").unwrap();
            }
            else
            {
                thread::sleep(Duration::from_secs(1));
            }
        }
    }

    #[test]
    fn test_make_request() {
        let buff = "GET / HTTP/1.1\r\nHost: localhost:8000\r\nUser-Agent: curl/7.58.0\r\nAccept: */*\r\n\r\n";
        let request = Request::new(buff);

        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/");
        assert_eq!(request.version, "HTTP/1.1");
    }

    #[test]
    fn test_parse_request_line() {
        let request = "GET / HTTP/1.1\r\nHost: localhost:8000\r\nUser-Agent: curl/7.58.0\r\nAccept: */*\r\n\r\n";
        assert_eq!(Request::parse_request_line(request), ("GET", "/", "HTTP/1.1"));
    }

    #[test]
    fn test_read_headers() {
        let request = "GET / HTTP/1.1\r\nHost: localhost:8000\r\nUser-Agent: curl/7.58.0\r\nAccept: */*\r\n\r\n";
        let mut headers = HashMap::new();
        headers.insert("Host", "localhost:8000");
        headers.insert("User-Agent", "curl/7.58.0");
        headers.insert("Accept", "*/*");
        let parsed = Request::parse_headers(request);
        assert_eq!(parsed.get("Host"), headers.get("Host"));
        assert_eq!(parsed.get("User-Agent"), headers.get("User-Agent"));
        assert_eq!(parsed.get("Accept"), headers.get("Accept"));
        assert_eq!(parsed.len(), headers.len());
    }

    #[test]
    fn test_parse_body() {
        let request = "POST / HTTP/1.1\r\nHost: localhost:8000\r\nUser-Agent: curl/7.58.0\r\nAccept: */*\r\nContent-Length: 7\r\nContent-Type: application/x-www-form-urlencoded\r\n\r\nFoo=Bar";
        let body = Request::parse_body(request);
        assert_eq!(body, Some("Foo=Bar"));
    }
}
