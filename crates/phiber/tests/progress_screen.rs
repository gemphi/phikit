//! Opening a file shows what is being read, over whatever is on screen, and
//! gets out of the way when the scene settles.

use phiber::v2::state::{ModalKey};
use phiber::v2::modal::{BModalList, BModalType};
use phiber::v2::scene::{load_progress_key, load_stage_key, selected_key};
use phiber::v2::state::{BState};
use phiber::v2::view::{BView, BViewList};

/// The labels a built view carries, flattened.
fn labels(v: &BView, out: &mut Vec<String>) {
    match v {
        BView::Label { text } | BView::Field { text } => out.push(text.clone()),
        BView::ProgressBar { label, .. } => out.push(label.clone()),
        BView::Modal { body, .. } => labels(body, out),
        BView::Column { list }
        | BView::Row { list }
        | BView::ListRow { list, .. } => list.iter().for_each(|c| labels(c, out)),
        BView::Card(card) if matches!(card.kind, phiber::v2::view::prim::BCardKind::Column(_)) => card.body.iter().for_each(|c| labels(c, out)),
        _ => {}
    }
}

#[test]
fn reading_a_file_takes_the_screen_and_then_gives_it_back() {
    let mut s = BState::new();
    // Nothing is loading: no progress screen.
    assert!(BModalList::new().items(&s).is_empty());

    // An import starts.
    s.set(&selected_key(), "C:/files/THYMUS-BOLDROID-WAVE-003.blend");
    s.set(&load_progress_key(), "0.4");
    s.set(&load_stage_key(), "Reading meshes");
    let shown = BModalList::new().items(&s);
    assert_eq!(shown.len(), 1, "the progress screen is up");
    let mut found = Vec::new();
    labels(&shown[0], &mut found);
    assert!(
        found.iter().any(|l| l.contains("THYMUS")),
        "it names the file being opened: {found:?}"
    );
    assert!(
        found.iter().any(|l| l == "Reading meshes"),
        "and the stage it is on: {found:?}"
    );

    // It outranks whatever modal was open …
    s.set(ModalKey::Open, BModalType::FileBrowser.slug());
    let shown = BModalList::new().items(&s);
    assert!(matches!(&shown[0], BView::Modal { kind, .. } if *kind == BModalType::Progress));

    // … and hands the screen back once the scene settles.
    s.remove(&load_progress_key());
    let shown = BModalList::new().items(&s);
    assert!(matches!(&shown[0], BView::Modal { kind, .. } if *kind == BModalType::FileBrowser));
}
