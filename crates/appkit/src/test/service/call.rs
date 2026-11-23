use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::service::{PCall, PStream, STOPPED};

/// A server for one call: it takes a request, answers with these pieces
/// as HTTP chunks, and gives back the request it was sent.
fn server(status: &'static str, piece: Vec<&'static str>) -> (String, JoinHandle<String>) {
    slow(status, piece, 0)
}

/// The same, waiting that many milliseconds before each piece after the
/// first. A call that was let go while it waits is not an error here.
fn slow(status: &'static str, piece: Vec<&'static str>, pause: u64) -> (String, JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a port to listen on");
    let host = listener.local_addr().expect("its address").to_string();
    let serve = move || {
        let (mut wire, _) = listener.accept().expect("a call");
        // The request: its head, then as much body as the head says.
        let (mut request, mut byte) = (Vec::new(), [0u8; 1]);
        while !request.ends_with(b"\r\n\r\n") && wire.read(&mut byte).is_ok_and(|read| read == 1) {
            request.push(byte[0]);
        }
        let head = String::from_utf8_lossy(&request).to_string();
        let length = head.lines().find_map(|line| line.to_ascii_lowercase().strip_prefix("content-length:").and_then(|value| value.trim().parse::<usize>().ok())).unwrap_or(0);
        let mut body = vec![0; length];
        wire.read_exact(&mut body).expect("the request's body");
        wire.write_all(format!("HTTP/1.1 {status}\r\nTransfer-Encoding: chunked\r\n\r\n").as_bytes()).expect("the answer's head to be sent");
        for (index, piece) in piece.iter().enumerate() {
            std::thread::sleep(Duration::from_millis(pause * index.min(1) as u64));
            wire.write_all(format!("{:x}\r\n{piece}\r\n", piece.len()).as_bytes()).ok();
            wire.flush().ok();
        }
        wire.write_all(b"0\r\n\r\n").ok();
        head + &String::from_utf8_lossy(&body)
    };
    (host, std::thread::spawn(serve))
}

/// Every line of a stream, once it has ended, and how it ended.
fn whole(stream: &PStream) -> (Vec<String>, Result<(), String>) {
    let mut line = Vec::new();
    for _ in 0..500 {
        let end = stream.end();
        line.extend(stream.take());
        match end {
            Some(end) => return (line, end),
            None => std::thread::sleep(Duration::from_millis(10)),
        }
    }
    panic!("the stream never ended");
}

#[test]
fn the_lines_of_an_answer_arrive_in_order_however_the_server_cuts_them() {
    // Three lines in four pieces: the second line is cut in two, and the
    // last has no line break after it.
    let (host, seen) = server("200 OK", vec!["{\"n\":1}\n{\"n\"", ":2}\n", "\n", "{\"n\":3}"]);
    let stream = PCall::to(&host).post("/api/chat", "{\"model\":\"m\"}");
    assert_eq!(whole(&stream), (vec!["{\"n\":1}".to_string(), "{\"n\":2}".to_string(), "{\"n\":3}".to_string()], Ok(())));
    // What was sent is a post of the body to the path.
    let request = seen.join().expect("the server's thread");
    assert!(request.starts_with("POST /api/chat HTTP/1.1\r\n") && request.ends_with("{\"model\":\"m\"}"), "{request}");
}

#[test]
fn a_call_that_only_asks_sends_no_body_and_gets_the_server_s_lines() {
    let (host, seen) = server("200 OK", vec!["{\"models\":[]}\n"]);
    let stream = PCall::to(&host).get("/api/tags");
    assert_eq!(whole(&stream), (vec!["{\"models\":[]}".to_string()], Ok(())));
    let request = seen.join().expect("the server's thread");
    assert!(request.starts_with("GET /api/tags HTTP/1.1\r\n") && request.ends_with("\r\n\r\n"), "{request}");
}

#[test]
fn a_server_that_refuses_gives_its_lines_and_says_what_it_answered() {
    let (host, _seen) = server("404 Not Found", vec!["{\"error\":\"no such model\"}\n"]);
    let stream = PCall::to(&host).post("/api/chat", "{}");
    assert_eq!(whole(&stream), (vec!["{\"error\":\"no such model\"}".to_string()], Err("the server answered 404".to_string())));
}

#[test]
fn a_server_that_is_not_there_ends_the_stream_with_why() {
    // Nothing listens on a port that was just let go.
    let free = TcpListener::bind("127.0.0.1:0").and_then(|listener| listener.local_addr()).expect("a port").to_string();
    let (line, end) = whole(&PCall::to(&free).post("/", "{}"));
    assert!(line.is_empty() && end.is_err());
    assert_eq!(PStream::ended(Ok(())).end(), Some(Ok(())));
}

#[test]
fn a_call_that_is_stopped_reads_no_more_and_says_it_was_stopped() {
    // The first line comes at once; the others would come after a wait.
    let (host, _seen) = slow("200 OK", vec!["{\"n\":1}\n", "{\"n\":2}\n", "{\"n\":3}\n"], 300);
    let stream = PCall::to(&host).post("/api/chat", "{}");
    let mut line = Vec::new();
    for _ in 0..200 {
        line.extend(stream.take());
        match line.is_empty() {
            true => std::thread::sleep(Duration::from_millis(10)),
            false => break,
        }
    }
    stream.stop();
    assert_eq!(whole(&stream), (Vec::new(), Err(STOPPED.to_string())));
    assert_eq!(line, ["{\"n\":1}"]);
}
