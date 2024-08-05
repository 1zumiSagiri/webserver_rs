use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

use webserver_rs::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6660").unwrap();

    let pool = ThreadPool::new(10);

    // get first two incoming connections and end the listener
    for stream in listener.incoming().take(4) {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

// HTTP request format:
// Method Request-URI HTTP-Version
// headers CRLF
// message-body
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let method_line = buf_reader.lines().next().unwrap().unwrap();

    if method_line == "GET / HTTP/1.1"{
        let status_line = "HTTP/1.1 200 OK";
        let contents = fs::read_to_string("hello.html").unwrap();
        let len = contents.len();

        let response = format!(
            "{status_line}\r\nContent-Length: {len}\r\n\r\n{contents}",
        );

        stream.write_all(response.as_bytes()).unwrap();
    } else {
        let status_line = "HTTP/1.1 404 NOT FOUND";
        let contents = fs::read_to_string("404.html").unwrap();
        let len = contents.len();

        let response = format!(
            "{status_line}\r\nContent-Length: {len}\r\n\r\n{contents}",
        );

        stream.write_all(response.as_bytes()).unwrap();
    }
}
