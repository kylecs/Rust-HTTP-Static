use std::collections::HashMap;
use std::net::TcpStream;
use std::io::BufReader;
use std::io::prelude::*;

pub enum RequestType {
    Get,
    Head,
    Other,
} //pub enum RequestType

pub struct HttpRequest {
    pub same_spec: bool,
    pub request_type: RequestType,
    pub path: String,
    pub keep_alive: bool,
    pub headers: HashMap<String, String>,
} //pub struct HttpRequest

impl HttpRequest {
    pub fn receive_request(stream: &mut TcpStream) -> Option<HttpRequest> {
        let same_spec: bool;
        let request_type: RequestType;
        let path: String;
        let mut keep_alive: bool = true;
        let mut headers: HashMap<String,String> = HashMap::new();
        let mut got_host: bool = false;

        let mut reader = BufReader::new(stream.try_clone().unwrap());
        //Read / interpret request listener
        let req_line;
        match read_line(&mut reader) {
            Some(line) => {
                req_line = line;
            },
            None => {
                return None
            },
        }
        if req_line.len() <  10 {
            //header is missing something
            //stream.write(b"HTTP/1.1 400 Bad Request").unwrap();
            return None;
        }
        let mut req_iter = req_line.split_whitespace();
        same_spec = String::from(req_iter.next_back().unwrap()).to_lowercase() == "http/1.1";
        path = String::from(req_iter.next_back().unwrap());

        request_type = match String::from(req_iter.next_back().unwrap()).to_lowercase().as_str() {
            "get" => RequestType::Get,
            "head" => RequestType::Head,
            _ => RequestType::Other,
        };
        //in compliance with HTTP/1.1, 100 continue is sent to HTTP/1.1 clients
        //upon receiving the status line
        if same_spec {
            send_continue(&mut stream.try_clone().unwrap());
        }

        //Read all headers, things don't supports are put in the headers hashmap
        loop{
            let header_line = read_line(&mut reader).unwrap();
            if header_line == "" {
                break;
            }
            let mut header_iter = header_line.split(":");
            let header_name = String::from(header_iter.next().unwrap());
            let header_name = String::from(header_name.trim());

            let header_value = String::from(header_iter.next().unwrap());
            let header_value = String::from(header_value.trim());

            match header_name.to_lowercase().as_str() {
                "connection" => {
                    keep_alive = match header_value.to_lowercase().as_str() {
                        "keep-alive" => true,
                        _ => false,
                    };
                },
                "host" => {
                    got_host = true;
                }
                _ => {
                    headers.insert(header_name, header_value);
                }
            }
        }
        if got_host || !same_spec{
            Some(HttpRequest{request_type: request_type, same_spec: same_spec,
                keep_alive: keep_alive, headers: headers, path: path})
        }else {
            None
        }
    }
    pub fn dump_headers(&self) {
        println!("Header Dump:");
        for (name, value) in &self.headers {
            println!("{} : {}", name, value);
        }
        println!("End Header Dump\n");
    }
}

fn read_line(reader: &mut BufReader<TcpStream>) -> Option<String> {
    let mut buffer = String::new();
    match reader.read_line(&mut buffer) {
        Ok(_) => {
            Some(String::from(buffer.trim()))
        },
        Err(_) => {
            None
        }
    }
}

fn send_continue(stream: &mut TcpStream) {
    stream.write(b"HTTP/1.1 100 Continue\n\n").unwrap();
}
