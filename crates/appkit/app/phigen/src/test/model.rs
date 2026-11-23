use std::io::{Read, Write};
use std::net::TcpListener;
use std::rc::Rc;
use std::thread::JoinHandle;
use std::time::Duration;

use appkit::pui::PApp;
use appkit::pui::core::{PEvent, PMode};
use appkit::pui::window::PStage;
use appkit::service::PTable;

use crate::test::{DESK, shown};
use crate::{History, Model, Phigen, Reply, Source, Turn, Who, data};

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

/// The model `data/model.tsv` names, on another server, as `data/text.tsv`
/// has it speak.
fn model(host: &str) -> Model {
    let table = format!("kind\tollama\nhost\t{host}\npath\t/api/chat\nname\ttest-model\n");
    Model::read(&PTable::read(&table), &PTable::read(data::TEXT)).expect("a server of a known kind")
}

/// All of an answer, once the last of it has arrived.
fn whole(mut source: Box<dyn Source>) -> String {
    let mut text = String::new();
    for _ in 0..500 {
        let (more, done) = source.more();
        text.push_str(&more);
        match done {
            true => return text,
            false => std::thread::sleep(Duration::from_millis(10)),
        }
    }
    panic!("the answer never ended");
}

/// Draw frames until nothing more is to come, as the screen does.
fn settle(stage: &mut PStage) {
    for _ in 0..500 {
        match stage.tick() {
            true => std::thread::sleep(Duration::from_millis(10)),
            false => return,
        }
    }
    panic!("the stage stayed busy");
}

const ANSWER: [&str; 3] = [
    "{\"message\":{\"role\":\"assistant\",\"content\":\"A kit \"},\"done\":false}\n",
    "{\"message\":{\"role\":\"assistant\",\"content\":\"of crates.\"},\"done\":false}\n",
    "{\"message\":{\"role\":\"assistant\",\"content\":\"\"},\"done\":true}\n",
];

#[test]
fn the_model_s_answer_is_its_pieces_in_order_and_it_is_sent_the_chat_without_the_greeting() {
    let (host, seen) = server("200 OK", ANSWER.to_vec());
    let past = [Turn::new(Who::Phigen, "Hello, I am Phigen."), Turn::new(Who::Person, "hi"), Turn::new(Who::Phigen, "hello")];
    assert_eq!(whole(model(&host).reply("what is phikit?", &past)), "A kit of crates.");
    let request = seen.join().expect("the server's thread");
    let body: serde_json::Value = serde_json::from_str(request.split("\r\n\r\n").nth(1).expect("a body")).expect("the body is JSON");
    assert_eq!((body["model"].as_str(), body["stream"].as_bool()), (Some("test-model"), Some(true)));
    let said: Vec<(&str, &str)> = body["messages"].as_array().expect("messages").iter().map(|message| (message["role"].as_str().unwrap_or_default(), message["content"].as_str().unwrap_or_default())).collect();
    // What it is told it is, then the chat from the person's first word.
    assert_eq!(said[0].0, "system");
    assert_eq!(said[1..], [("user", "hi"), ("assistant", "hello"), ("user", "what is phikit?")]);
}

#[test]
fn a_model_that_cannot_answer_says_why_in_the_words_the_text_table_gives() {
    let fail = PTable::read(data::TEXT).get("reply.fail").to_string();
    // The server knows no such model.
    let (host, _seen) = server("404 Not Found", vec!["{\"error\":\"model 'test-model' not found\"}\n"]);
    assert_eq!(whole(model(&host).reply("hi", &[])), fail.replace("{why}", "model 'test-model' not found"));
    // The server is not there at all.
    let free = TcpListener::bind("127.0.0.1:0").and_then(|listener| listener.local_addr()).expect("a port").to_string();
    let answer = whole(model(&free).reply("hi", &[]));
    assert!(answer.starts_with(fail.split("{why}").next().unwrap_or_default()) && answer.len() > fail.len() - 5, "{answer}");
}

#[test]
fn a_table_that_names_no_server_gives_no_model() {
    let text = PTable::read(data::TEXT);
    assert!(Model::read(&PTable::read("kind\tstand\n"), &text).is_none());
    assert!(Model::read(&PTable::read("kind\tollama\n"), &text).is_none());
    // The table Phigen ships names one, on this computer.
    assert_eq!(Model::read(&PTable::read(data::MODEL), &text).map(|model| model.name), Some("phi4-builder".to_string()));
}

#[test]
fn step_20_a_model_answers_me_and_its_answer_arrives_as_it_is_written() {
    let (host, _seen) = server("200 OK", ANSWER.to_vec());
    let look = Rc::new(Phigen::look(""));
    let app = Phigen::holding(Rc::clone(&look), History::new(Box::new(model(&host)), "Hello."));
    let mut stage = app.stage();
    stage.lay(DESK, PMode::Desktop);
    Phigen::key(&mut stage, "what is phikit?");
    // My words are there at once, and the answer is awaited: nothing
    // more can be said until it has arrived.
    assert!(app.desk().history.borrow().waiting());
    Phigen::key(&mut stage, "too soon");
    assert_eq!(app.desk().history.borrow().current().turn.len(), 3);
    settle(&mut stage);
    assert!(!app.desk().history.borrow().waiting());
    assert!(shown(&stage, &look).contains(&"A kit of crates.".to_string()));
    // What was typed too soon is still in the field, and can be said now.
    assert!(shown(&stage, &look).contains(&"too soon".to_string()));
}

#[test]
fn step_21_i_can_stop_an_answer_while_it_is_being_written() {
    // The first piece comes at once; the rest would come after a wait.
    let (host, _seen) = slow("200 OK", ANSWER.to_vec(), 400);
    let look = Rc::new(Phigen::look(""));
    let app = Phigen::holding(Rc::clone(&look), History::new(Box::new(model(&host)), "Hello."));
    let mut stage = app.stage();
    stage.lay(DESK, PMode::Desktop);
    Phigen::key(&mut stage, "what is phikit?");
    let written = || app.desk().history.borrow().current().turn.last().map(|turn| turn.text.clone()).unwrap_or_default();
    for _ in 0..200 {
        stage.tick();
        match written().is_empty() {
            true => std::thread::sleep(Duration::from_millis(10)),
            false => break,
        }
    }
    // While it is written the button stops it.
    assert!(shown(&stage, &look).contains(&"Stop".to_string()) && !shown(&stage, &look).contains(&"Send".to_string()));
    assert!(stage.send(&PEvent::KeyDown(PEvent::ESCAPE)));
    assert!(!app.desk().history.borrow().waiting());
    // What had been written stays, no more comes, and the button sends
    // again.
    std::thread::sleep(Duration::from_millis(600));
    assert!(!stage.tick());
    assert_eq!(written(), "A kit ");
    assert!(shown(&stage, &look).contains(&"Send".to_string()));
    // With nothing being written Escape stops nothing.
    assert!(!stage.send(&PEvent::KeyDown(PEvent::ESCAPE)));
}

#[test]
fn the_server_says_which_models_it_has_the_first_time_anyone_asks() {
    let (host, seen) = server("200 OK", vec!["{\"models\":[{\"name\":\"test-model:latest\"},{\"name\":\"other:7b\"}]}\n"]);
    let table = format!("kind\tollama\nhost\t{host}\npath\t/api/chat\nname\ttest-model\nlist\t/api/tags\n");
    let mut model = Model::read(&PTable::read(&table), &PTable::read(data::TEXT)).expect("a server of a known kind");
    // Until the server has said, the model the table names is the only one.
    let mut known = model.list();
    assert!(known.starts_with(&["test-model".to_string()]));
    for _ in 0..500 {
        match known.len() {
            1 => {
                std::thread::sleep(Duration::from_millis(10));
                known = model.list();
            }
            _ => break,
        }
    }
    // The same model under its default tag is not listed twice, and once
    // the server has said there is nothing more to wait for.
    assert_eq!(known, ["test-model", "other:7b"]);
    assert!(!model.asking());
    assert!(seen.join().expect("the server's thread").starts_with("GET /api/tags HTTP/1.1\r\n"));
}

#[test]
fn the_model_that_was_chosen_is_the_one_the_server_is_asked_for() {
    let (host, seen) = server("200 OK", ANSWER.to_vec());
    let mut model = model(&host);
    // A table that names no place to ask has nothing to wait for.
    assert_eq!((model.list(), model.name(), model.asking()), (vec!["test-model".to_string()], "test-model", false));
    assert!(model.choose("other:7b") && !model.choose(""));
    assert_eq!((model.list(), model.name()), (vec!["test-model".to_string(), "other:7b".to_string()], "other:7b"));
    assert_eq!(whole(model.reply("hi", &[])), "A kit of crates.");
    assert!(seen.join().expect("the server's thread").contains("\"model\":\"other:7b\""));
}
