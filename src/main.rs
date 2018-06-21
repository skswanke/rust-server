extern crate hello;

use hello::ThreadPool;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs::File;
use std::thread;
use std::time::Duration;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();
        println!("Connection established!");
        pool.execute(|| {
            handle_connection(stream);
        });
    }
    println!("Shutting down");
}

struct Status {
    ok: &'static str,
    not_found: &'static str 
}

impl Status {
    fn new() -> Status {
        Status {
            ok: "HTTP/1.1 200 OK\r\n\r\n",
            not_found: "HTTP/1.1 404 NOT FOUND\r\n\r\n"
        }
    }
}


fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    let http_status = Status::new();

    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let response = if buffer.starts_with(get) {
        get_response(http_status.ok, "hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        get_response(http_status.not_found, "hello.html")
    } else {
        get_response(http_status.ok, "404.html")
    };

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn get_response(status_line: &str, filename: &str) -> String {
        let mut file = File::open(filename).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let response = format!("{}{}", status_line, contents);
    
        response
}
