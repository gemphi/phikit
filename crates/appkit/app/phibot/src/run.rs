//! Open the botanist in a window. `--width 420 --height 760` opens it at
//! another size than `data/theme.tsv` gives; `--show garden` opens it on
//! the greenhouse, which then loads at once; `--look dark` in that look.

use std::rc::Rc;

use appkit::pui::PApp;
use appkit::pui::window::PDeed;
use appkit::screen;
use appkit::service::{PBundle, PFolder};

use crate::Phibot;

/// The value after each `flag` among the words.
fn given(word: &[String], flag: &str) -> Vec<String> {
    word.windows(2).filter(|pair| pair[0] == flag).map(|pair| pair[1].clone()).collect()
}

/// Open the botanist and run until its window is closed.
pub fn run(word: &[String]) {
    let row = |flag: &str, token: &str| given(word, flag).last().map(|value| format!("size\t{token}\t{value}\n")).unwrap_or_default();
    let look = Rc::new(Phibot::look(&(row("--width", screen::WIDTH) + &row("--height", screen::HEIGHT))));
    let app = Phibot::new(look);
    given(word, "--look").iter().for_each(|look| _ = app.desk().shade.turn(look));
    let mut stage = app.stage();
    for space in given(word, "--show") {
        app.desk().plant(&space);
        stage.window.apply(&PDeed::Show(space));
    }
    let own = PFolder::new(&[concat!(env!("CARGO_MANIFEST_DIR"), "/asset")]);
    let film = app.desk().film.clone();
    screen::run(stage, (crate::part::Moving(film), (own, PBundle)));
}
