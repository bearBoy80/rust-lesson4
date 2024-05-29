use std::{
    env, fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread::Thread,
};

fn main() {
    let server_addres = "127.0.0.1:8080";
    let tcp = TcpListener::bind(server_addres).unwrap();
    for stream in tcp.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }

    println!("server closed successfully");
}

fn handle_connection(mut stream: TcpStream) {
    let buff = BufReader::new(&mut stream);
    let http_req: Vec<_> = buff
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    let status_line = "HTTP/1.1 200 OK";
    let path = "examples/hello.html";
    println!("{:?}", env::current_dir());
    let contents = fs::read_to_string(path).unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
    println!("http request: {:#?}", http_req);
}
