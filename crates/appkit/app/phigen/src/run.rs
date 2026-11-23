//! Open Phigen in a window: at a desk from `main`, in a browser from
//! `web`. The words are a command line's; a browser gives none.
//!
//! `--width 420 --height 760` opens it at another size than
//! `data/theme.tsv` gives: below the width appkit's `data/mode.tsv`
//! gives, it is laid out as a mobile app. `--say "a line"` types the line
//! and sends it before the window opens; it may be given more than once.
//! `--show search` opens it on that space, `--pick menu` with the menu
//! of that row of the bar shown, `--open profile` with that modal open,
//! and `--look dark` in that look, which is not kept.
//! `--forget` opens it without the chats and the choices kept from
//! before, and keeps none. With nothing said in any chat it opens on the
//! welcome, unless `--plain` is given.

use std::rc::Rc;

use appkit::pui::PApp;
use appkit::pui::window::PDeed;
use appkit::screen;
use appkit::service::{PBundle, PFolder};

use crate::Phigen;

/// The value after each `flag` among the words.
fn given(word: &[String], flag: &str) -> Vec<String> {
    word.windows(2).filter(|pair| pair[0] == flag).map(|pair| pair[1].clone()).collect()
}

/// Open Phigen and run until its window is closed.
pub fn run(word: &[String]) {
    let row = |flag: &str, token: &str| given(word, flag).last().map(|value| format!("size\t{token}\t{value}\n")).unwrap_or_default();
    let look = Rc::new(Phigen::look(&(row("--width", screen::WIDTH) + &row("--height", screen::HEIGHT))));
    let app = Phigen::open(look, !word.iter().any(|word| word == "--forget"));
    given(word, "--look").iter().for_each(|look| _ = app.desk().shade.turn(look));
    let mut stage = app.stage();
    given(word, "--say").iter().for_each(|line| Phigen::key(&mut stage, line));
    // A person's first time is welcomed.
    let welcome = (app.fresh() && !word.iter().any(|word| word == "--plain")).then(|| PDeed::Open("welcome".to_string()));
    let (show, pick) = (given(word, "--show").into_iter().map(PDeed::Show), given(word, "--pick").into_iter().map(PDeed::Pick));
    let deed = welcome.into_iter().chain(show).chain(pick).chain(given(word, "--open").into_iter().map(PDeed::Open));
    deed.for_each(|deed| _ = stage.window.apply(&deed));
    // Phigen's own assets, on disk, then the ones every app shares, which
    // are built into the program.
    let own = PFolder::new(&[concat!(env!("CARGO_MANIFEST_DIR"), "/asset")]);
    screen::run(stage, (own, PBundle));
}
