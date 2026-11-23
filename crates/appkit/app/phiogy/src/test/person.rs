//! What the line answers besides a room's call, asked over a real
//! socket as a page asks Phiora: an app's document, who the page
//! belongs to, and a person's own documents.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};

use super::board;
use super::exchange::line_on;
use crate::Phiogy;

/// Ask the line something: the status, the cookie it gave, and what it
/// answered.
fn ask(at: &str, method: &str, path: &str, cookie: &str, body: &str) -> (u16, String, Value) {
    let mut wire = TcpStream::connect(at).expect("the line is there");
    wire.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    let request = format!("{method} {path} HTTP/1.1\r\nHost: {at}\r\nCookie: {cookie}\r\nContent-Length: {}\r\n\r\n{body}", body.len());
    wire.write_all(request.as_bytes()).unwrap();
    let mut answer = String::new();
    wire.read_to_string(&mut answer).unwrap();
    let (head, said) = answer.split_once("\r\n\r\n").unwrap_or((answer.as_str(), ""));
    let status = head.split_whitespace().nth(1).and_then(|code| code.parse().ok()).unwrap_or_default();
    let given = head.lines().filter_map(|line| line.split_once(": ")).find(|(name, _)| *name == "set-cookie").map(|(_, value)| value.split(';').next().unwrap_or_default().to_string());
    (status, given.unwrap_or_default(), serde_json::from_str(said).unwrap_or_default())
}

#[test]
fn an_app_s_document_is_the_app_s_own_file() {
    let line = line_on(Arc::new(Mutex::new(board())));
    let at = line.at();
    // The web Phiogy's own file, as it is on disk now: one source.
    let file: Value = serde_json::from_str(&std::fs::read_to_string(Phiogy::source()).expect("the web Phiogy's file")).expect("JSON");
    let (status, cookie, document) = ask(&at, "GET", "/app/phiogy", "", "");
    assert_eq!((status, cookie.as_str()), (200, ""));
    assert_eq!(document, file);
    // The older spelling answers the same; an app there is not says so.
    assert_eq!(ask(&at, "GET", "/apps/phiogy", "", "").2, file);
    let (status, _, said) = ask(&at, "GET", "/app/nope", "", "");
    assert_eq!((status, said), (404, json!({ "error": "no_app", "message": "No app nope." })));
    assert_eq!(ask(&at, "GET", "/app/..%2Fphiogy", "", "").0, 404);
}

#[test]
fn a_page_is_told_who_it_is_and_stays_that_person() {
    let line = line_on(Arc::new(Mutex::new(board())));
    let at = line.at();
    let (status, cookie, me) = ask(&at, "GET", "/me", "", "");
    assert_eq!(status, 200);
    assert!(cookie.starts_with("phi_sid=") && me["id"].as_str().is_some_and(|id| id.starts_with("guest_")), "{cookie} {me}");
    assert_eq!((me["guest"].clone(), me["name"].clone(), me["handle"].clone(), me["email"].clone()), (json!(true), json!("Guest"), Value::Null, Value::Null));
    // With its cookie it is the same person, and is given no other.
    let (_, again, same) = ask(&at, "GET", "/me", &cookie, "");
    assert_eq!((again.as_str(), &same["id"]), ("", &me["id"]));
    // Another page is another person.
    assert_ne!(ask(&at, "GET", "/me", "", "").2["id"], me["id"]);
}

#[test]
fn each_person_keeps_their_own_documents() {
    let line = line_on(Arc::new(Mutex::new(board())));
    let at = line.at();
    let (status, a, _) = ask(&at, "PUT", "/state/follow", "", r#"{"c_opal":1}"#);
    assert_eq!(status, 204);
    assert_eq!(ask(&at, "GET", "/state/follow", &a, ""), (200, String::new(), json!({ "c_opal": 1 })));
    // What is kept again takes the place of what was.
    assert_eq!(ask(&at, "PUT", "/state/follow", &a, "[1,2]").0, 204);
    assert_eq!(ask(&at, "GET", "/state/follow", &a, "").2, json!([1, 2]));
    // Another person has none of it; nor has this one what was never kept.
    let (status, _, said) = ask(&at, "GET", "/state/follow", "", "");
    assert_eq!((status, said["error"].as_str()), (404, Some("unset")));
    assert_eq!(ask(&at, "GET", "/state/other", &a, "").0, 404);
}

#[test]
fn a_document_s_name_and_what_is_kept_are_checked() {
    let line = line_on(Arc::new(Mutex::new(board())));
    let at = line.at();
    let said = |method: &str, path: &str, body: &str| {
        let (status, _, said) = ask(&at, method, path, "", body);
        (status, said["error"].as_str().unwrap_or_default().to_string())
    };
    assert_eq!(said("GET", "/state/..%2Fetc", ""), (400, "bad_name".to_string()));
    assert_eq!(said("PUT", "/state/x", "{nope"), (400, "bad_json".to_string()));
    assert_eq!(said("PUT", "/state/x", ""), (400, "empty".to_string()));
    // More than the line reads is refused, and the one who sent it
    // hears so.
    let most = crate::comp::Limit::own().body;
    assert_eq!(said("PUT", "/state/x", &format!("\"{}\"", "x".repeat(most + 1))), (413, "too_large".to_string()));
    assert_eq!(said("POST", "/state/x", "{}"), (404, "not_found".to_string()));
}
