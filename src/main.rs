use single_threaded_web_server::ThreadPool;
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();

    let pool = ThreadPool::new(100);

    let mut count = 0;

    for stream in listener.incoming(){
        let stream = stream.unwrap();
        count = count + 1;
        pool.execute(move || {
            handle_connection(stream, count);
        });
    }
}

fn handle_connection(mut stream: TcpStream, count: i64) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    // for header in BufReader::new(&mut stream).lines() {
    //     let header = header.unwrap();
    //     if header == "\r" { break }
    // }

    // let get = b"GET / HTTP/1.1\r\n";
    // let sleep = b"GET /sleep HTTP/1.1\r\n";

    if (count % 10) == 0 {
        println!("Adding delay. Count: {}", count);
        thread::sleep(Duration::from_secs(2));
    }

    // let (status_line, filename) = if buffer.starts_with(get) {
    //     ("HTTP/1.1 200 OK", "hello.html")}
    // // } else if buffer.starts_with(sleep) {
    // //     thread::sleep(Duration::from_secs(5));
    // //     ("HTTP/1.1 200 OK", "hello.html")
    // // } else
    //     else {
    //     ("HTTP/1.1 404 NOT FOUND", "404.html")
    // };
    let header = "
HTTP/1.0 200 OK
Connection: keep-alive
Content-Length: 174
Content-Type: text/html; charset=utf-8
    ";
    let contents = fs::read_to_string("hello.html").unwrap();

    let response = format!("{}\r\n\r\n{}", header, contents);
    stream.write(response.as_bytes()).unwrap();
}
