use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

fn main() {
    // localhost 7878 포트에서 수신되는 TCP 연결을 listen하기
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // 영원히 블록하면서, 이 IP 주소로 들어오는 요청을 처리
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    // 처음 1024 바이트의 데이터를 스트림으로부터 읽어들임
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";

    // 요청 안의 데이터에 따라 환영인사로 응답하거나 404 에러
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };
    let contents = fs::read_to_string(filename).unwrap();

    // 다시 스트림에 응답을 씀
    // 클라이언트에게 응답이 전송될 수 있게 스트림을 플러시함
    let response = format!("{}{}", status_line, contents);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
