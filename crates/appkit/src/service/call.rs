//! A call: a request to a server that answers over time, a line at a
//! time, as a model does when it writes. The lines arrive on a stream the
//! app looks at once a frame (`PView::tick`); a thread does the waiting.
//!
//! It speaks plain HTTP, which is what a server on the same machine
//! speaks. In a browser there is no thread to wait on: the stream ends at
//! once and says so.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

/// How a stream that was stopped says it ended.
pub const STOPPED: &str = "stopped";

/// What has arrived of an answer, and how it ended, if it has.
#[derive(Debug, Default)]
struct Heard {
    line: Vec<String>,
    end: Option<Result<(), String>>,
    /// Whether the app has asked for the answer to stop.
    stop: bool,
}

/// The answer to a call, as it arrives.
#[derive(Debug, Clone, Default)]
pub struct PStream {
    heard: Arc<Mutex<Heard>>,
}

impl PStream {
    /// A stream that has ended already, that way.
    pub fn ended(end: Result<(), String>) -> PStream {
        let stream = PStream::default();
        stream.with(|heard| heard.end = Some(end));
        stream
    }

    /// The lines that have arrived since this was last asked, in order.
    pub fn take(&self) -> Vec<String> {
        self.with(|heard| std::mem::take(&mut heard.line))
    }

    /// How the answer ended: `None` while it is still arriving, `Ok` when
    /// all of it has, and why when it could not. Every line is on the
    /// stream before its end is.
    pub fn end(&self) -> Option<Result<(), String>> {
        self.with(|heard| heard.end.clone())
    }

    /// Stop the answer: no more of it is read, and the server is let go,
    /// which is how it learns to stop writing. The stream ends saying it
    /// was stopped.
    pub fn stop(&self) {
        self.with(|heard| heard.stop = true);
    }

    fn with<T>(&self, change: impl FnOnce(&mut Heard) -> T) -> T {
        change(&mut self.heard.lock().unwrap_or_else(|poisoned| poisoned.into_inner()))
    }
}

/// A server, by its address: `127.0.0.1:11434`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PCall {
    host: String,
}

impl PCall {
    pub fn to(host: &str) -> PCall {
        PCall { host: host.to_string() }
    }

    /// Post a JSON body to a path of the server. The answer's lines
    /// arrive on the stream that is returned.
    pub fn post(&self, path: &str, body: &str) -> PStream {
        self.send(format!("POST {path} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", self.host, body.len()))
    }

    /// Ask for a path of the server, sending nothing: what it has, such
    /// as which models. The answer's lines arrive on the stream.
    pub fn get(&self, path: &str) -> PStream {
        self.send(format!("GET {path} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n", self.host))
    }

    /// Send a request on a thread of its own.
    fn send(&self, request: String) -> PStream {
        let (stream, host) = (PStream::default(), self.host.clone());
        let out = stream.clone();
        let work = move || {
            let end = talk(&host, &request, &out);
            out.with(|heard| heard.end = Some(end));
        };
        // Where no thread can be had the call cannot be made.
        let begun = std::thread::Builder::new().spawn(work);
        begun.err().into_iter().for_each(|error| stream.with(|heard| heard.end = Some(Err(error.to_string()))));
        stream
    }
}

/// Put the whole lines among `bytes` on the stream; keep what is left of
/// an unfinished one in `rest`.
fn feed(rest: &mut Vec<u8>, bytes: &[u8], out: &PStream) {
    rest.extend_from_slice(bytes);
    while let Some(end) = rest.iter().position(|byte| *byte == b'\n') {
        let line: Vec<u8> = rest.drain(..=end).collect();
        let text = String::from_utf8_lossy(&line).trim().to_string();
        if !text.is_empty() {
            out.with(|heard| heard.line.push(text));
        }
    }
}

/// Send a request and read the answer, a line at a time, onto the stream.
fn talk(host: &str, request: &str, out: &PStream) -> Result<(), String> {
    let said = |error: std::io::Error| error.to_string();
    let wire = TcpStream::connect(host).map_err(said)?;
    (&wire).write_all(request.as_bytes()).map_err(said)?;
    let mut reader = BufReader::new(wire);
    let line = |reader: &mut BufReader<TcpStream>| {
        let mut text = String::new();
        reader.read_line(&mut text).map(|_| text.trim().to_string()).map_err(said)
    };
    let status = line(&mut reader)?;
    let code = status.split_whitespace().nth(1).unwrap_or_default().to_string();
    // The headers say how the body is sent: in chunks, by its length, or
    // until the server closes.
    let (mut chunked, mut length) = (false, None);
    loop {
        let header = line(&mut reader)?.to_ascii_lowercase();
        if header.is_empty() {
            break;
        }
        chunked |= header.starts_with("transfer-encoding:") && header.contains("chunked");
        length = header.strip_prefix("content-length:").and_then(|value| value.trim().parse::<usize>().ok()).or(length);
    }
    let mut rest = Vec::new();
    match (chunked, length) {
        (true, _) => loop {
            let size = usize::from_str_radix(line(&mut reader)?.split(';').next().unwrap_or_default().trim(), 16).map_err(|error| error.to_string())?;
            if size == 0 {
                break;
            }
            if out.with(|heard| heard.stop) {
                return Err(STOPPED.to_string());
            }
            let mut chunk = vec![0; size];
            reader.read_exact(&mut chunk).map_err(said)?;
            feed(&mut rest, &chunk, out);
            line(&mut reader)?;
        },
        (false, Some(length)) => {
            let mut body = vec![0; length];
            reader.read_exact(&mut body).map_err(said)?;
            feed(&mut rest, &body, out);
        }
        (false, None) => {
            let mut body = Vec::new();
            reader.read_to_end(&mut body).map_err(said)?;
            feed(&mut rest, &body, out);
        }
    }
    // A last line the server did not end.
    feed(&mut rest, b"\n", out);
    match code == "200" {
        true => Ok(()),
        false => Err(format!("the server answered {code}")),
    }
}
