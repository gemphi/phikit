//! Signing in with a handle and a password, the sessions, and taking
//! one's data out: asked of something that answers as Phiora does,
//! and either takes the sign-in or refuses it. No account is made and
//! no password of anybody's is used: what is typed here is nonsense.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use appkit::pui::PApp;
use appkit::pui::core::{PEvent, PMark, PMode, PPoint};
use appkit::pui::window::{PDeed, PStage};
use appkit::theme::{PLook, PTheme};

use super::tools::typed;
use super::{AT, DESK, drawn, shown, site};
use crate::Phiogy;
use crate::kept::Kept;
use crate::phiora::Phiora;

const GUEST: &str = r#"{"id":"guest_1","name":"Guest","handle":null,"guest":true,"canDemo":false}"#;
const NOOR: &str = r#"{"id":"u_noor","name":"Noor Hale","handle":"noor.hale","guest":false,"canDemo":false}"#;
const WRONG: &str = r#"{"error":"bad_credentials","message":"Handle or password is wrong."}"#;
const SESSION: &str = r#"[{"id":"a","current":true,"browser":"Phiogy on Windows"},{"id":"b","current":false,"browser":"Chrome on Windows"}]"#;

/// Something that answers as Phiora does: who is asking, a sign-in it
/// takes or refuses, the sessions, and an export. Where it listens.
fn phiora(grant: bool) -> String {
    let door = TcpListener::bind("127.0.0.1:0").expect("a port");
    let at = door.local_addr().expect("where it is").to_string();
    let signed = Arc::new(AtomicBool::new(false));
    _ = std::thread::Builder::new().spawn(move || {
        for wire in door.incoming().map_while(Result::ok) {
            let mut reader = BufReader::new(wire);
            let (mut first, mut length) = (String::new(), 0usize);
            _ = reader.read_line(&mut first);
            let mut line = String::new();
            while reader.read_line(&mut line).is_ok_and(|read| read > 0) && !line.trim().is_empty() {
                length = line.to_lowercase().strip_prefix("content-length:").and_then(|rest| rest.trim().parse().ok()).unwrap_or(length);
                line.clear();
            }
            _ = reader.read_exact(&mut vec![0; length]);
            let path = first.split_whitespace().nth(1).unwrap_or_default();
            let me = match signed.load(Ordering::SeqCst) {
                true => NOOR,
                false => GUEST,
            };
            let (status, body) = match (path, grant) {
                ("/me", _) => ("200 OK", me),
                ("/auth/signin", true) => {
                    signed.store(true, Ordering::SeqCst);
                    ("200 OK", NOOR)
                }
                ("/auth/signin", false) => ("401 Unauthorized", WRONG),
                ("/auth/session", _) => ("200 OK", SESSION),
                ("/auth/session/revoke", _) => ("200 OK", r#"{"ok":true,"revoked":1}"#),
                ("/me/export", _) => ("200 OK", NOOR),
                _ => ("404 Not Found", "{}"),
            };
            let answer = format!("HTTP/1.1 {status}\r\nSet-Cookie: phi_sid=test; Path=/\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
            _ = reader.get_mut().write_all(answer.as_bytes());
        }
    });
    at
}

/// The account open in a window whose documents are that Phiora's.
fn account(at: &str) -> (Phiogy, PStage, Rc<PLook>) {
    let look = Rc::new(Phiogy::look(""));
    let app = Phiogy::holding(Rc::clone(&look), site(), Phiogy::board(), Ok(AT.to_string()), None).keeping(Kept::open(None, Some(Phiora::at(at))));
    let mut stage = app.stage();
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert!(stage.window.apply(&PDeed::Open("account".to_string())));
    stage.lay(DESK, PMode::of_width(DESK.width));
    foot(&mut stage, &look);
    (app, stage, look)
}

/// Turn the wheel until the account's foot is in sight: being
/// somebody is the last thing in it.
fn foot(stage: &mut PStage, look: &PLook) {
    let body = stage.panel().expect("the account is open").body;
    _ = shown(stage, look);
    stage.send(&PEvent::Scroll(body.center(), PPoint::new(0.0, -4000.0)));
}

/// A press on the last line of words that reads so.
fn press(stage: &mut PStage, look: &PLook, wanted: &str) -> bool {
    let at = drawn(stage, look).iter().rev().find_map(|mark| match mark {
        PMark::Text { at, text, .. } if text == wanted => Some(at.plus(PPoint::new(2.0, 2.0))),
        _ => None,
    });
    stage.send(&PEvent::PointerDown(at.unwrap_or_else(|| panic!("{wanted} in the account"))))
}

#[test]
fn step_36_a_password_is_never_drawn_and_what_phiora_refuses_is_said_in_its_words() {
    let (app, mut stage, look) = account(&phiora(false));
    let desk = app.desk();
    let word = |name: &str| desk.text.get(name).to_string();
    let text = shown(&stage, &look);
    assert!(text.contains(&word("account.handle")) && text.contains(&word("account.secret")), "{text:?}");
    // The handle is typed first; a press on the other field gives it
    // the keys.
    typed(&mut stage, "noor.hale");
    assert!(press(&mut stage, &look, &word("account.secret")));
    typed(&mut stage, "not-it");
    let text = shown(&stage, &look);
    let mark = look.word("control.mask").unwrap_or_default().repeat(6);
    assert!(text.contains(&"noor.hale".to_string()) && text.contains(&mark) && !text.iter().any(|line| line.contains("not-it")), "{text:?}");
    // Refused: Phiora's own words; the password is gone from the
    // field, and the person is who they were.
    assert!(press(&mut stage, &look, &word("account.signIn")));
    let text = shown(&stage, &look);
    assert!(text.contains(&"Handle or password is wrong.".to_string()) && !text.contains(&mark) && desk.kept.guest(), "{text:?}");
}

#[test]
fn step_37_signed_in_i_see_my_sessions_sign_the_others_out_and_take_my_data_out() {
    let (app, mut stage, look) = account(&phiora(true));
    let desk = app.desk();
    let word = |name: &str| desk.text.get(name).to_string();
    typed(&mut stage, "noor.hale");
    assert!(press(&mut stage, &look, &word("account.secret")));
    typed(&mut stage, "not-it");
    // Enter signs in, as the control does.
    assert!(stage.send(&PEvent::KeyDown(PEvent::ENTER)) && !desk.kept.guest());
    foot(&mut stage, &look);
    let text = shown(&stage, &look);
    let wanted = ["Noor Hale".to_string(), "@noor.hale".to_string(), word("account.signOut"), word("account.session"), "Phiogy on Windows".to_string(), word("account.this"), "Chrome on Windows".to_string(), word("account.other"), word("account.alone"), word("account.data")];
    assert!(wanted.iter().all(|line| super::said(&text, line)) && !text.contains(&word("account.handle")), "{text:?}");
    assert!(press(&mut stage, &look, &word("account.alone")));
    // Everything Phiora keeps about me is written to a file, and the
    // system is asked where a copy is to be kept.
    assert!(press(&mut stage, &look, &word("account.export")));
    let asked = stage.errand();
    let [PDeed::Save(file)] = asked.as_slice() else { panic!("{asked:?}") };
    assert!(std::fs::read_to_string(file).is_ok_and(|all| all.contains("noor.hale")), "{file}");
}
