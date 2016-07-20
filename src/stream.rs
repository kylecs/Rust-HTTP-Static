use std::net::{TcpStream};
extern crate time;
use request;
use response;
use std::fs::File;
use std::io::prelude::*;
use std::time::Duration;
use mime_guess;
use std::path::Path;
use std::ops::Add;

pub struct Stream {
    stream: TcpStream,
    directory: String,
}

impl Stream {
    pub fn new(stream: TcpStream, directory: String) -> Stream {
        stream.set_read_timeout(Some(Duration::new(5, 0))).unwrap();
        stream.set_write_timeout(Some(Duration::new(5, 0))).unwrap();
        Stream {stream: stream, directory: directory}
    }

    pub fn handle_client(&mut self) {
        let mut keepalive = true;
        //hold connection while client requests keep-alive or until timeout
        'main: while keepalive{
            let mut req;
            //try to handle request, if request fails or times out, the session will end
            match request::HttpRequest::receive_request(&mut self.stream.try_clone().unwrap()) {
                Some(request) => {
                    req = request;
                },
                None => {
                    break 'main;
                }
            }
            keepalive = req.keep_alive;

            //log request
            match req.request_type {
                request::RequestType::Get => {
                    println!("GET - {}", req.path);
                },
                request::RequestType::Head => {
                    println!("HEAD - {}", req.path);
                }
                _ => {
                    println!("UNKNOWN - {}", req.path);
                },
            }

            //create response object
            let mut res = response::HttpResponse::create_response(self.stream.try_clone().unwrap(),
                match req.request_type {
                    request::RequestType::Head => true,
                    _ => false,
                });

            //generate response
            self.handle_request(&mut req, &mut res);

            //handle session keepalive
            if req.keep_alive {
                res.add_header_str("Connection", "keep-alive");
            } else {
                res.add_header_str("Connection", "close");
            }

            //send data
            res.finalize();
        }
    }
    fn handle_request(&self, req: &mut request::HttpRequest, res: &mut response::HttpResponse){
        match req.request_type {
            request::RequestType::Get => {
                let (found, mime_type, file_opt) = access_file(self.directory.clone(), req.path.clone());
                if found {
                    match file_opt {
                        Some(mut file) => {
                            res.add_header_str("Content-Type", &mime_type.unwrap());
                            file.read_to_end(&mut res.buffer).unwrap();
                        },
                        None => {
                            res.set_status(response::HttpStatus::NotFound);
                        },
                    }
                }else {
                    res.set_status(response::HttpStatus::NotFound);
                }
            },
            request::RequestType::Head => {
                let (found, mime_type, file_opt) = access_file(self.directory.clone(), req.path.clone());
                if found {
                    match file_opt {
                        Some(file) => {
                            res.add_header_str("Content-Type", &mime_type.unwrap());
                            res.add_header_str("Content-Length", file.metadata().unwrap().len()
                                .to_string().as_str())
                        },
                        None => {
                            res.set_status(response::HttpStatus::NotFound);
                        },
                    }
                }else {
                    res.set_status(response::HttpStatus::NotFound);
                }
            },
            _ => {
                res.set_status(response::HttpStatus::NotImplemented);
            }
        }
    }
}

// access_file returns file_found, mime_guess, Option<File>
fn access_file(dir_path: String, mut req_path: String) -> (bool, Option<String>, Option<File>) {
    let found: bool;
    let mime_type: Option<String>;
    let file: Option<File>;
    let full_path: String;

    //forward / to /index.html
    if req_path == "/" {
        req_path = String::from("/index.html");
    }

    //disalow use of .. in path
    if req_path.contains("..") {
        return (false,None,None);
    }

    full_path = dir_path.add(req_path.as_str());

    let path = Path::new(&full_path);
    //get mime_type
    mime_type = match path.extension() {
        Some(ext) => {
            let ext = ext.to_str().unwrap();
            match mime_guess::get_mime_type_str(ext) {
                Some(mime_type) => Some(String::from(mime_type)),
                None => Some(String::from("application/octet-stream")),
            }
        },
        None => {
            Some(String::from("application/octet-stream"))
        }
    };

    file = File::open(&full_path).ok();
    found = file.is_some();

    (found, mime_type, file)
}
