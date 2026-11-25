//! Phiora first: the app's document asked of Phiora over a real socket,
//! whole or in pieces, and the app's own file where Phiora does not
//! answer with one.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

use appkit::service::PTable;
use serde_json::Value;

use super::exchange::line;
use crate::phiora::Phiora;
use crate::{Phiogy, Site, data, fact};

/// Something that answers every ask with those bytes, and where it is.
fn answering(answer: String) -> String {
    let door = TcpListener::bind("127.0.0.1:0").expect("a port");
    let at = door.local_addr().expect("where it is").to_string();
    _ = std::thread::Builder::new().spawn(move || {
        for wire in door.incoming().map_while(Result::ok) {
            let mut reader = BufReader::new(wire);
            let mut line = String::new();
            while reader.read_line(&mut line).is_ok_and(|read| read > 0) && !line.trim().is_empty() {
                line.clear();
            }
            _ = reader.get_mut().write_all(answer.as_bytes());
        }
    });
    at
}

/// A place nothing listens at: a port that was just let go.
fn nowhere() -> String {
    TcpListener::bind("127.0.0.1:0").and_then(|door| door.local_addr()).expect("a port").to_string()
}

/// An answer in pieces, as the Node Phiora sends its own.
fn pieced(status: &str, body: &str) -> String {
    let (first, rest) = body.split_at(body.len() / 2);
    format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\n{first}\r\n{:x}\r\n{rest}\r\n0\r\n\r\n", first.len(), rest.len())
}

#[test]
fn phiora_is_asked_as_a_page_asks_it_and_read_whole_or_in_pieces() {
    // The line answers an app's document as Phiora does, its length said.
    let line = line();
    let file: Value = serde_json::from_str(&std::fs::read_to_string(Phiogy::source()).expect("the web Phiogy's file")).expect("JSON");
    let phiora = Phiora::at(&line.at());
    assert_eq!(phiora.app("phiogy").and_then(|doc| serde_json::from_str::<Value>(&doc).ok()), Some(file));
    // One that is refused is no answer.
    assert_eq!(phiora.app("nope"), None);
    // In pieces, as Node sends it; and with nothing said of its length.
    assert_eq!(Phiora::at(&answering(pieced("200 OK", "hello, world"))).get("/any").as_deref(), Some("hello, world"));
    assert_eq!(Phiora::at(&answering("HTTP/1.1 200 OK\r\n\r\nto the end".to_string())).get("/any").as_deref(), Some("to the end"));
    assert_eq!(Phiora::at(&answering(pieced("404 Not Found", "{}"))).get("/any"), None);
    // Nobody there is no answer, and soon: the window does not wait.
    let began = Instant::now();
    assert_eq!(Phiora::at(&nowhere()).app("phiogy"), None);
    assert!(began.elapsed() < Duration::from_secs(3), "{:?}", began.elapsed());
}

#[test]
fn where_phiora_is_comes_from_the_command_line_first() {
    let at = |given: &str| Phiora::own(Some(given.to_string())).map(|phiora| phiora.at);
    // The place alone, or written as the web writes it; and the app's
    // server's, over TLS under its path. It is kept as it was given.
    assert_eq!(at("localhost:4320").as_deref(), Some("localhost:4320"));
    assert_eq!(at("http://localhost:4320/").as_deref(), Some("http://localhost:4320"));
    assert_eq!(at("https://phiegg.com/api/").as_deref(), Some("https://phiegg.com/api"));
    // The word for none has nobody asked.
    assert_eq!(at(PTable::read(data::PHIORA).get("none")), None);
}

#[test]
fn step_16_the_site_is_phiora_s_when_it_answers_and_the_app_s_file_when_not() {
    let text = PTable::read(data::TEXT);
    let path = Phiogy::source();
    let file = Site::open(&path);
    assert_eq!((file.asked.clone(), fact::read(&file, &text).as_str()), (None, text.get("fact.file")));
    // Phiora's document is the file's with a place called otherwise, as
    // one edited in Phiora is.
    let mut doc: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("the web Phiogy's file")).expect("JSON");
    let key = PTable::read(data::SITE);
    let (nav, label) = (key.get("nav"), key.get("label"));
    let was = doc[nav][0][label].as_str().expect("a first place").to_string();
    doc[nav][0][label] = Value::from("Named in Phiora");
    let phiora = Phiora::at(&answering(pieced("200 OK", &doc.to_string())));
    let site = Site::first(Some(&phiora), "phiogy", &path);
    assert_eq!(site.asked, Some((phiora.at.clone(), true)));
    assert_eq!((site.venue[0].label.as_str(), file.venue[0].label.as_str()), ("Named in Phiora", was.as_str()));
    assert_eq!(fact::read(&site, &text), text.fill("fact.phiora", "at", &phiora.at));
    // What the document names is read from beside the file either way:
    // the rooms, their people, the hosts.
    assert_eq!(site.venue[1..], file.venue[1..]);
    assert_eq!((site.venue[0].plate.clone(), site.face.len()), (file.venue[0].plate.clone(), file.face.len()));
    // One that answers with what is no document, one that refuses and
    // one that is not there leave the file, and the window says so.
    for at in [answering(pieced("200 OK", r#"{"error":"no_app"}"#)), answering(pieced("404 Not Found", "{}")), nowhere()] {
        let phiora = Phiora::at(&at);
        let site = Site::first(Some(&phiora), "phiogy", &path);
        assert_eq!((site.asked.clone(), &site.venue), (Some((at.clone(), false)), &file.venue));
        assert_eq!(fact::read(&site, &text), text.fill("fact.unanswered", "at", &at));
    }
}
