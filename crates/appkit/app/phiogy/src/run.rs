//! Open Phiogy in a window: at a desk from `main`, in a browser from
//! `web`. The words are a command line's; a browser gives none.
//!
//! The window is on the app's server (`data/app.tsv`): Phiora there,
//! and the calls on the line there (`https://phiegg.com/wrt`), so
//! everybody who runs Phiogy, on any machine or in a browser, meets in
//! the same rooms. `--local` keeps to this machine: Phiora here, and
//! the window serving the line itself, where `data/line.tsv` says, so
//! pages that open a room are seen in it at once. `--port 4300` serves
//! it on another port (and is local), and 0 has the system choose;
//! `--quiet` opens the window with no line, and `--at localhost:4311`
//! on the line another serves there. `--line` serves the line alone,
//! with no window, and says where it is. `--remote` reads nothing of
//! this disk's source, as a copy on another machine does: the site and
//! its pictures come from the server.
//!
//! `--demo u_noor` has it be that member of Phiora's, where Phiora
//! lets anyone be one to try things, and `--signout` nobody again.
//! `--signin noor.hale` signs in with that handle and the password the
//! system's setting `PHIOGY_SECRET` holds; `--export` and a path
//! writes everything Phiora keeps about the person there.
//! `--bag s_phiogy/ID` puts that thing of that shop in the bag, and
//! `--pay` pays for the bag.
//! `--post` and a path posts that picture; `--mute tavi` mutes that person;
//! `--wear s_phiogy/theme_sphere` puts that theme on; `--host NAME` makes a host.
//! `--write` and some words writes them in the room it opens on;
//! `--find` opens that room searched and `--type` types into the field
//! that has the keys; `--pins` lists what is pinned in it; `--who lora`
//! opens it with that person beside it.
//!
//! `--wheel -400` turns the wheel that far over the modal that is
//! open, or over the window's middle.
//!
//! `--view 0` opens it on the first picture posted in its room, in the
//! viewer, and `--tool zoom` has a tool of the viewer pressed, by its
//! name in `data/tool.tsv`. `--join` has the window join the call of the
//! room it opens on, and `--say` has it say each second what that call
//! sees. `--camera` turns the camera on in it, `--share` shares a screen,
//! `--mute` mutes, `--deaf` turns the others' sound off, and `--then lobby`
//! opens that page once it is in; `--set device=stand` gives a row of
//! `data/peer.tsv` another value: a tone and a grey ramp for the devices.
//! `--caller phiogy/lora` is a caller with no window: it joins that
//! room's call on this program's own line, or on the line `--at` names,
//! and says each second what it hears and sees of the others, for as
//! many seconds as `--for` gives, or until it is stopped.
//!
//! `--width 420 --height 760` opens it at another size than
//! `data/theme.tsv` gives: below the width appkit's `data/mode.tsv`
//! gives, it is laid out as a mobile app. `--room lobby` opens it on
//! that room, `--show venue` on that space, `--pick menu` with the menu
//! of that row of the bar shown, `--open profile` with that modal open,
//! `--look dark` in that look, which is not kept, and `--press Calls`
//! pressed where those words are drawn (run/press.rs). `--forget` opens
//! it without what was chosen before, and keeps nothing.
//!
//! Its hosts and rooms are asked of Phiora first, where
//! `data/phiora.tsv` says, and read from the app's own file when Phiora
//! does not answer. `--phiora localhost:4320` asks another, and
//! `--phiora none` nobody; `--site` and a path reads that file, and
//! asks nobody. `--read` says what was read and from where, with what
//! was said in the room it is on, and opens no window.

mod caller;
mod own;
mod press;
mod read;

use std::rc::Rc;
use std::sync::{Arc, Mutex};

use appkit::pui::PApp;
use appkit::pui::core::{PMode, PSize};
use appkit::pui::window::PDeed;
use appkit::screen;
use appkit::service::{PBundle, PFolder, PStore, PTable};

use crate::board::Board;
use crate::kept::Kept;
use crate::phiora::Phiora;
use crate::site::Site;
use crate::{Phiogy, data};

/// The value after each `flag` among the words.
fn given(word: &[String], flag: &str) -> Vec<String> {
    word.windows(2).filter(|pair| pair[0] == flag).map(|pair| pair[1].clone()).collect()
}

/// The line, served on that board, and where it listens; or why there
/// is none. It is served for as long as what is returned is kept.
#[cfg(not(target_family = "wasm"))]
fn serve(word: &[String], board: &Arc<Mutex<Board>>, text: &PTable) -> (Option<crate::exchange::Exchange>, Result<String, String>) {
    use crate::exchange::{Exchange, Set};
    if word.iter().any(|word| word == "--quiet") {
        return (None, Err(text.get("status.off").to_string()));
    }
    // On a line another serves, none is served here: the one named, or
    // the app's server's.
    if let Some(place) = given(word, "--at").pop() {
        return (None, Ok(place));
    }
    if !local(word) {
        return (None, Ok(Phiogy::server("server.line")));
    }
    let row = |flag: &str, name: &str| given(word, flag).last().map(|value| format!("{name}\t{value}\n")).unwrap_or_default();
    match Exchange::open(Set::own(&(row("--port", "port") + &row("--host", "host"))), Arc::clone(board)) {
        Ok(line) => {
            let at = line.at();
            (Some(line), Ok(at))
        }
        Err(error) => (None, Err(error.to_string())),
    }
}

/// Whether the window keeps to this machine: asked to, or to serve the
/// line here.
fn local(word: &[String]) -> bool {
    word.iter().any(|word| word == "--local" || word == "--line") || !given(word, "--port").is_empty()
}

/// A browser cannot listen: there is no line.
#[cfg(target_family = "wasm")]
fn serve(_: &[String], _: &Arc<Mutex<Board>>, text: &PTable) -> (Option<()>, Result<String, String>) {
    (None, Err(text.get("status.browser").to_string()))
}

/// Open Phiogy and run until its window is closed; or, asked for the
/// line alone, serve it until the program is stopped.
pub fn run(word: &[String]) {
    let (text, own) = (PTable::read(data::TEXT), PTable::read(data::APP));
    let has = |flag: &str| word.iter().any(|word| word == flag);
    let board = Phiogy::board();
    // Asked what it reads, it serves no line.
    let (line, at) = match has("--read") {
        true => (None, Err(text.get("status.off").to_string())),
        false => serve(word, &board, &text),
    };
    if has("--line") {
        return alone(line, &at, &text);
    }
    if let Some(key) = given(word, "--caller").pop() {
        return caller::caller(word, &key, &at, &text);
    }
    // A file that was named is read as it is; else Phiora is asked first.
    let named = given(word, "--site").pop();
    let here = local(word).then(|| PTable::read(data::PHIORA).get("here").to_string());
    let phiora = named.is_none().then(|| Phiora::own(given(word, "--phiora").pop().or(here))).flatten();
    // `--remote`: as a copy on another machine is, with nothing of this
    // disk's source: the site and its pictures from the app's server.
    let remote = has("--remote");
    let source = || match remote {
        true => Phiogy::server("server.site"),
        false => Phiogy::source(),
    };
    let site = Site::first(phiora.as_ref(), own.get("key"), &named.unwrap_or_else(source));
    // What the person keeps is in their own folder and, where Phiora
    // gave the site, on Phiora too; asked to forget, it is nowhere.
    let keep = !has("--forget");
    let store = PStore::user(own.get("folder")).filter(|_| keep);
    let kept = Kept::open(store, phiora.filter(|_| keep && site.asked.as_ref().is_some_and(|(_, gave)| *gave)));
    let row = |flag: &str, token: &str| given(word, flag).last().map(|value| format!("size\t{token}\t{value}\n")).unwrap_or_default();
    let look = Rc::new(Phiogy::look(&(row("--width", screen::WIDTH) + &row("--height", screen::HEIGHT))));
    // On a line it does not serve, it listens to its rooms as a page does.
    let away = line.is_none() && at.is_ok();
    let app = Phiogy::open(look, site, board, at, keep).keeping(kept);
    let app = match away {
        true => app.away(),
        false => app,
    };
    // Where the real library is built in, it carries this window's calls.
    let film = app.desk().film();
    let (over, shown) = (caller::over(word), film.clone());
    let app = app.carried(Rc::new(move || caller::engine(&over, &shown)));
    given(word, "--look").iter().for_each(|look| _ = app.desk().shade.turn(look));
    // The page it opens on: its space is wished for, and shown once
    // the window stands.
    given(word, "--room").iter().for_each(|room| _ = app.desk().show(room));
    // What it was given to write is written in the room it is on. Asked
    // what it reads, it says all of it, and ends when Phiora has what
    // changed.
    // Be one of Phiora's members to try things, or nobody again.
    given(word, "--demo").iter().for_each(|id| _ = app.desk().kept.sign(Some(id)));
    if has("--signout") {
        app.desk().kept.sign(None);
    }
    // Sign in with a handle; the password is the system's setting of
    // that name, never a word of the command line.
    let refused = given(word, "--signin").pop().and_then(|handle| app.desk().kept.enter(&handle, &std::env::var(own.get("secret")).unwrap_or_default()).err());
    refused.iter().for_each(|why| println!("{}", text.fill("read.refused", "why", why)));
    // Everything Phiora keeps about the person, written to a file.
    given(word, "--export").iter().for_each(|file| _ = app.desk().kept.export().map(|all| std::fs::write(file, all)));
    given(word, "--write").iter().for_each(|said| _ = app.desk().write(said));
    given(word, "--post").iter().for_each(|path| _ = app.desk().post(path));
    given(word, "--mute").iter().filter(|id| !app.desk().muted(id)).for_each(|id| app.desk().mute(id));
    own::own(app.desk(), word);
    // Things put in the bag, each as its shop and its id; and the bag
    // paid for.
    for (shop, id) in given(word, "--bag").iter().filter_map(|place| place.split_once('/')) {
        let thing = app.desk().shop(shop).and_then(|shop| shop.product.iter().find(|product| product.id == id).cloned());
        thing.into_iter().for_each(|thing| app.desk().pick(shop, &thing));
    }
    if has("--pay") {
        app.desk().pay();
    }
    if has("--read") {
        read::read(app.desk(), &text).iter().for_each(|line| println!("{line}"));
        return app.desk().kept.settle();
    }
    caller::begin(app.desk(), word);
    // Asked to, the window says each second what its call sees, as the
    // caller with no window does.
    let watch = app.desk().call.borrow().as_ref().map(|call| (call.peer.clone(), call.key.clone(), call.watch())).filter(|_| has("--say"));
    if let Some((peer, key, watch)) = watch {
        println!("{}", text.fill("caller.begin", "peer", &peer).replace("{key}", &key).replace("{at}", app.desk().line.as_ref().as_ref().map_or("", String::as_str)));
        let words = text.clone();
        _ = std::thread::Builder::new().spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                let seen = crate::board::lock(&watch).clone();
                caller::said(&seen, &words).iter().for_each(|line| println!("{line}"));
            }
        });
    }
    let mut stage = app.stage();
    let (show, pick) = (given(word, "--show").into_iter().map(PDeed::Show), given(word, "--pick").into_iter().map(PDeed::Pick));
    // `--view 0` opens the first picture of the room in the viewer.
    let view = given(word, "--view").pop().and_then(|number| number.parse::<usize>().ok());
    app.desk().view.set(view);
    *app.desk().now.album.borrow_mut() = app.desk().shot();
    let viewed = view.map(|_| PDeed::Open(crate::part::VIEW.to_string()));
    let deed = show.chain(pick).chain(given(word, "--open").into_iter().map(PDeed::Open)).chain(viewed);
    deed.for_each(|deed| _ = stage.window.apply(&deed));
    // A tool is told to the viewer as a word: the stage does what was
    // wished for after an event, and what that wished for after another.
    app.desk().wish.borrow_mut().extend(given(word, "--tool").into_iter().map(PDeed::Tell));
    (0..2).for_each(|_| _ = stage.send(&appkit::pui::core::PEvent::KeyUp(' ')));
    // A room opened searched, with its pinned listed, with a person
    // beside it; and keys typed into whichever field has them.
    app.desk().now.finding.set(has("--find"));
    app.desk().now.pins.set(has("--pins"));
    *app.desk().now.who.borrow_mut() = given(word, "--who").pop();
    // Keys go to the parts that are shown: the window is laid out
    // first, at the size it opens in.
    let size = PSize { width: stage.theme().size(screen::WIDTH).unwrap_or_default(), height: stage.theme().size(screen::HEIGHT).unwrap_or_default() };
    stage.lay(size, PMode::of_width(size.width));
    given(word, "--type").iter().flat_map(|text| text.chars()).for_each(|key| _ = stage.send(&appkit::pui::core::PEvent::KeyDown(key)));
    let pressed = press::press(&mut stage, word);
    // The wheel turned over the modal that is open, or where `--press`
    // pressed last, or over the middle of the window: what is far down a
    // drawer or a column is brought into sight.
    let middle = stage.panel().map_or(pressed.unwrap_or(appkit::pui::core::PPoint::new(size.width / 2.0, size.height / 2.0)), |panel| panel.body.center());
    let turn = given(word, "--wheel").into_iter().filter_map(|by| by.parse::<f32>().ok());
    turn.for_each(|by| _ = stage.send(&appkit::pui::core::PEvent::Scroll(middle, appkit::pui::core::PPoint::new(0.0, by))));
    // Phiogy's own assets, on disk, then the ones every app shares, which
    // are built into the program.
    let asset = Phiogy::asset();
    let own = PFolder::new(&[asset.to_string_lossy().as_ref()]);
    // The line is served until the window is closed: `line` is kept
    // to the end of this.
    // The pictures the site names are the web's own files, by their keys.
    let hosted = match remote {
        true => crate::hosted::Hosted::own().far_only(),
        false => crate::hosted::Hosted::own(),
    };
    screen::run(stage, (crate::hosted::Moving(film), (hosted, (own, PBundle))));
}

/// The line alone: say where it is, and stay until the program is
/// stopped. One that could not open says why, and the program ends
/// saying it failed.
#[cfg(not(target_family = "wasm"))]
fn alone(line: Option<crate::exchange::Exchange>, at: &Result<String, String>, text: &PTable) {
    match (line, at) {
        (Some(line), Ok(at)) => {
            println!("{}", text.fill("line.open", "at", at));
            line.wait();
        }
        (_, Err(why)) | (None, Ok(why)) => {
            eprintln!("{}", text.fill("line.fail", "why", why));
            std::process::exit(1);
        }
    }
}

#[cfg(target_family = "wasm")]
fn alone(_: Option<()>, _: &Result<String, String>, _: &PTable) {}
