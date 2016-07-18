use std::io::BufWriter;
use std::io::prelude::*;
use std::collections::HashMap;
use std::net::TcpStream;
use time;

pub struct HttpResponse {
    pub status: HttpStatus,
    headers: HashMap<String, String>,
    writer: BufWriter<TcpStream>,
    pub buffer: Vec<u8>,
    head_req: bool,
}

pub enum HttpStatus {
    Ok,
    NotFound,
    NotImplemented,
    BadRequest,
}

impl HttpResponse {
    pub fn create_response(stream: TcpStream, head_req: bool) -> HttpResponse {
        HttpResponse{status: HttpStatus::Ok, headers: HashMap::new(),
            writer: BufWriter::new(stream), buffer: Vec::new(), head_req: head_req}
    }

    fn write_body(&mut self) {
        self.writer.write(self.buffer.as_slice()).expect("Couldn't write buffer!");
    }

    fn write_headers(&mut self) {
        for (name, value) in &self.headers {
            self.writer.write(name.as_bytes()).unwrap() as i32;
            self.writer.write(b": ").unwrap() as i32;
            self.writer.write(value.as_bytes()).unwrap() as i32;
            self.writer.write(b"\n").unwrap() as i32;
        }
    }

    fn write_status(&mut self) {
        let status = String::from(match self.status {
            HttpStatus::Ok => {
                "HTTP/1.1 200 OK\n"
            },
            HttpStatus::NotFound => {
                "HTTP/1.1 404 Not Found\n"
            },
            HttpStatus::NotImplemented => {
                "HTTP/1.1 501 Not Implemented\n"
            },
            HttpStatus::BadRequest => {
                "HTTP/1.1 400 Bad Request\n"
            }
        });
        self.writer.write(status.as_bytes()).unwrap();
    }

    pub fn append_string(&mut self, string: String) {
        self.buffer.append(&mut Vec::from(string.as_bytes()));
    }

    pub fn finalize(&mut self){
        match self.status {
            HttpStatus::NotFound => {
                self.append_string(String::from("404 Not Found"));
            },
            HttpStatus::NotImplemented => {
                self.append_string(String::from("501 Not Implemented"));
            },
            HttpStatus::BadRequest => {
                self.append_string(String::from("400 Bad Request"));
            },
            _ => {}
        }
        //final headers
        if !self.head_req{
            let body_length = self.buffer.len();
            self.add_header_str("Content-Length", body_length.to_string().as_str());
        }

        //in compliance with HTTP/1.1 spec, all responses must include time in UTC/GMT
        let now_str: String = time::strftime("%a, %d %b %Y %T %Z", &time::now_utc()).unwrap();
        self.add_header(String::from("Date"), now_str);

        self.write_status();
        self.write_headers();

        //line break to start body
        self.writer.write(b"\n").unwrap() as i32;
        self.write_body();

        //flush the buffer to ensure all data is sent
        self.writer.flush().unwrap();
    }

    pub fn set_status(&mut self, status: HttpStatus) {
        self.status = status;
    }

    pub fn add_header(&mut self, name: String, value: String) {
        self.headers.insert(name, value);
    }

    pub fn add_header_str(&mut self, name: &str, value: &str) {
        self.headers.insert(String::from(name), String::from(value));
    }
}
