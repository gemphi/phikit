//! How long Phiogy's own part of a frame takes: laying the window out
//! and drawing it, with no screen behind it. Not run with the others
//! (it measures, it does not check): run it alone and read what it
//! prints.
//!
//!     cargo test -p phiogy pace -- --ignored --nocapture

use std::time::Instant;

use appkit::pui::core::{PMode, PSheet};

use super::{DESK, staged};

#[test]
#[ignore = "a measure, read by a person"]
fn pace_of_a_frame_laid_out_and_drawn() {
    let (_app, mut stage, look) = staged(DESK);
    let turn = 200;
    let began = Instant::now();
    (0..turn).for_each(|_| stage.lay(DESK, PMode::of_width(DESK.width)));
    let laid = began.elapsed().as_secs_f64() * 1000.0 / turn as f64;
    let began = Instant::now();
    let mark: usize = (0..turn).map(|_| {
        let mut sheet = PSheet::new(look.as_ref());
        stage.draw(&mut sheet);
        sheet.mark.len()
    }).sum();
    let drawn = began.elapsed().as_secs_f64() * 1000.0 / turn as f64;
    let began = Instant::now();
    (0..turn).for_each(|_| _ = stage.tick());
    let ticked = began.elapsed().as_secs_f64() * 1000.0 / turn as f64;
    println!("a frame of the lobby: laid out in {laid:.2} ms, drawn in {drawn:.2} ms ({} marks), a tick {ticked:.3} ms", mark / turn);
}

#[test]
fn a_move_that_changes_nothing_under_the_pointer_draws_nothing_again() {
    use appkit::pui::core::{PEvent, PPoint};
    let (_app, mut stage, look) = staged(DESK);
    stage.draw(&mut PSheet::new(look.as_ref()));
    // Down the middle of the messages and down the rooms list, a pixel
    // at a time: the window is drawn again only where what is under the
    // pointer changed, a few times, not once a pixel.
    let mut said = Vec::new();
    for (x, from) in [(700.0, 150.0), (150.0, 250.0)] {
        let turned = (0..120).filter(|step| {
            let moved = stage.send(&PEvent::PointerMove(PPoint::new(x, from + *step as f32)));
            moved.then(|| stage.draw(&mut PSheet::new(look.as_ref())));
            moved
        }).count();
        said.push(turned);
    }
    assert!(said.iter().all(|turned| *turned < 20), "{said:?}");
}
