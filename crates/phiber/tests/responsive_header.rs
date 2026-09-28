//! A header with no room folds its right-hand controls behind one icon, and a
//! modal that elevates a host does not draw a second header under its title.

use phiber::v2::state::{AppKey};
use phiber::v2::host::BHostType;
use phiber::v2::part::filter::{BFilter, is_narrow};
use phiber::v2::state::{BState};
use phiber::v2::view::{BView, BViewItem};

fn at_width(width: f32) -> BState {
    let mut s = BState::new();
    s.set(AppKey::WindowWidth, width.to_string());
    s
}

#[test]
fn a_wide_window_shows_the_controls_inline() {
    let s = at_width(1600.0);
    assert!(!is_narrow(&s));
    let view = BFilter::new(BHostType::View3D).build(&s);
    assert!(
        !matches!(view, BView::Popover { .. }),
        "with room, the shading modes stay inline"
    );
}

#[test]
fn a_narrow_window_folds_them_behind_one_icon() {
    let s = at_width(900.0);
    assert!(is_narrow(&s));
    let view = BFilter::new(BHostType::View3D).build(&s);
    let BView::Popover { anchor, body, .. } = view else {
        panic!("a narrow header folds its right-hand controls");
    };
    assert!(
        matches!(anchor.as_ref(), BView::Button(ctl) if ctl.icon.as_deref() == Some("filter")),
        "the fold is one filter icon"
    );
    assert_eq!(body.len(), 1, "and it holds the controls it replaced");
}

#[test]
fn an_unmeasured_window_is_not_treated_as_narrow() {
    // Before the first paint there is no width; the header must not fold on
    // the strength of a missing value.
    assert!(!is_narrow(&BState::new()));
}

#[test]
fn a_modal_hosting_an_editor_drops_that_editors_own_header() {
    let card = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/v2/view/draw/modal/card.rs"),
    )
    .expect("modal card source");
    assert!(
        card.contains("without_own_header(self.body)"),
        "the dialog's title bar is the only header"
    );
    assert!(
        card.contains("BRegionType::Header"),
        "and only the header region is dropped — every other region stays"
    );
}
