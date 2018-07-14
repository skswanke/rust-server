extern crate server;

use server::ThreadPool;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs::File;
use std::thread;
use std::time::Duration;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Listening at: http://127.0.0.1:7878");
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

enum Status {
    Ok,
    NotFound 
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let response = if buffer.starts_with(get) {
        get_response(Status::Ok, "hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        get_response(Status::NotFound, "hello.html")
    } else {
        get_response(Status::Ok, "404.html")
    };

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn get_response(status: Status, filename: &str) -> String {
        let mut file = File::open(filename).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let status_line = match status {
            Status::Ok => "HTTP/1.1 200 OK\r\n\r\n",
            Status::NotFound => "HTTP/1.1 404 NOT FOUND\r\n\r\n",
        };

        let response = format!("{}{}", status_line, contents);
    
        response
}
