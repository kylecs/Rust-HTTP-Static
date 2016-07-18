#![allow(dead_code)]
use std::net::{TcpListener};
use std::thread;
use argparse::{ArgumentParser,Store};
use std::env;

extern crate time;
extern crate mime_guess;
extern crate argparse;

pub mod request;
pub mod response;
pub mod stream;

fn main() {
    let mut dir = env::current_dir().unwrap().to_str().unwrap().to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Serve static files at the provided directory over HTTP");
        ap.refer(&mut dir)
            .add_option(&["-d", "--directory"], Store, "Directory to serve over HTTP");
        ap.parse_args_or_exit();
    }

    let listener = TcpListener::bind("0.0.0.0:80").unwrap();
    println!("Listening on port 80\n");
    for stream in listener.incoming() {
        let cdir = dir.clone();
        //spawn thread to handle client
        thread::spawn(move || {
            let mut client = stream::Stream::new(stream.unwrap(), String::from(cdir));
            client.handle_client();
        });
    }
}
