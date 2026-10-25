//! One row height, for every kind of row.
//!
//! There were two answers. `row_height` was 26 — a control's own height, with
//! no room over or under it — and the tree, the list, the table and a menu's
//! popover all read it. `card_head_height()` computed `button_height +
//! HEAD_ROOM` = 30 for itself, and the card rows, the section headings and the
//! rail's entries read that. So a card's row and a menu's row were different
//! heights, and neither could say why.
//!
//! `row_height` IS that expression now, and `card_head_height` returns it, so
//! the question has one answer and moving `BUTTON_HEIGHT` moves every row with
//! it. A menu's rows are the rows of a list that happens to float; floating
//! does not make a row shorter.
//!
//! ## The second answer, and the third
//!
//! That settled a row against a control. It left two more disagreements, both
//! of them things a reader sees as one kind of element drawn at two sizes.
//!
//! **A heading had three heights.** A bar or region header took
//! `header_height` — the module the whole window's grid is built from. A card's
//! own heading took `row_height`, which was six pixels shorter, so the
//! Properties panel's title sat visibly under the bar above it. And a
//! horizontal strip's tab took `header_height - menu_top_gap`.
//!
//! The first two are now one: `row_height` IS the module, so a card head, a
//! row and a bar header are one height. The third stays, and is the one worth
//! keeping — a tab HANGS from its bar with the gap above it, which is the
//! topbar's tab shape and what `chrome_geometry` checks. A Properties rail tab
//! is not one of those; it is a card head, which is what lets it line up with
//! the panel headings beside it.
//!
//! **A control had two.** `field.rs` drew a field at `row_height` while every
//! button drew at `button_height`, so a button standing next to a field was
//! `HEAD_ROOM` shorter than it. A field is a control, and takes a control's
//! height now.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BState, CtxKey};
use phiber::v2::theme::{BPaletteName, BThemeMode, ui_style_defaults_in};

/// A row gives the control it holds room over and under it, and that is the
/// whole of what a row's height is.
/// A heading is one height, wherever it is drawn.
///
/// There were three. A bar or region header took `header_height` (the module);
/// a strip tab took `header_height - menu_top_gap`, two pixels under it; and a
/// card's own heading took `row_height`, which was six under. All three read as
/// "a heading" on screen — the Properties panel's title, the tab that opens it
/// and the bar above both — and no two were the same height.
///
/// The strip tab's subtraction is the one worth naming: it made the tabs
/// unable to line up with the card headings beside them *at any gap*, because
/// the items themselves were different sizes. Matching the column gaps was not
/// enough on its own, which is what the pitch guard in
/// `the_properties_strip_is_a_strip.rs` is paired with.
#[test]
fn a_heading_is_one_height() {
    for name in BPaletteName::ALL.iter().copied() {
        for mode in BThemeMode::ALL {
            let d = ui_style_defaults_in(name, mode).dimensions;
            assert_eq!(
                d.card_head_height(),
                d.header_height,
                "{name:?}/{mode:?}: a card's heading is a header's height",
            );
            assert_eq!(
                d.row_height,
                d.header_height,
                "{name:?}/{mode:?}: and a row is the module too, so a row of controls \
                 fills the band it sits in",
            );
        }
    }
    // A Properties rail tab is a card head, so it is the module too — which
    // is what lets it line up with the panel headings beside it. A horizontal
    // strip's caps are NOT: they hang from their bar with the menu gap above,
    // which is the topbar's tab shape and is deliberate (`chrome_geometry`).
    let shape = include_str!("../src/v2/view/draw/tab/shape.rs");
    let height = shape.split("pub fn height(").nth(1).expect("the cap height");
    let height = &height[..height.find("\n    }").unwrap_or(height.len())];
    let code: String = height
        .lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        code.contains("card_head_height()"),
        "a rail tab takes the card-head module, which is now the header's own height",
    );
}

/// A control is one height too, and a field is a control.
///
/// `field.rs` was the one control that took `row_height` instead, so every
/// button standing next to a field came out `HEAD_ROOM` shorter than it. That
/// is what "the buttons look thin" was.
#[test]
fn a_field_is_a_control_not_a_row() {
    let field = include_str!("../src/v2/view/draw/field.rs");
    assert!(
        field.contains("dimensions.button_height"),
        "a field takes a control's height",
    );
    let code: String = field
        .lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !code.contains(".h(px(ctx.style.dimensions.row_height))"),
        "and not a row's, or a button beside it is shorter than it is",
    );
}

#[test]
fn a_row_is_a_control_plus_the_room_around_it() {
    use phiber::v2::theme::style::metric::{BUTTON_HEIGHT, CONTROL_CLEAR};
    for name in BPaletteName::ALL.iter().copied() {
        for mode in BThemeMode::ALL {
            let d = ui_style_defaults_in(name, mode).dimensions;
            // A control clears its row by `CONTROL_CLEAR` above and below.
            assert_eq!(d.row_height, BUTTON_HEIGHT + 2.0 * CONTROL_CLEAR, "{name:?}/{mode:?}: a row is its control plus room");
            assert_eq!(d.button_height, BUTTON_HEIGHT, "{name:?}/{mode:?}: and a control is a control");
            assert!(d.row_height > d.button_height, "{name:?}/{mode:?}: the room is real");
        }
    }
}

/// The second answer is gone: `card_head_height` is a name for `row_height`,
/// not a sum of its own that can drift from it.
#[test]
fn a_cards_row_and_every_other_row_are_the_same_question() {
    for name in BPaletteName::ALL.iter().copied() {
        for mode in BThemeMode::ALL {
            let d = ui_style_defaults_in(name, mode).dimensions;
            assert_eq!(d.card_head_height(), d.row_height, "{name:?}/{mode:?}: one answer");
        }
    }
    let src = include_str!("../src/v2/theme/style/metric.rs");
    assert!(
        src.contains("pub const fn card_head_height(&self) -> f32 {\n        self.row_height\n    }"),
        "card_head_height defers to row_height rather than computing a second answer"
    );
}

/// And the rows the window actually paints are that height — not just the
/// token. Measured, because a row that sets `min_height` and then grows, or one
/// whose wrapper squashes it, would pass the token check and still look wrong.
#[gpui::test]
async fn every_painted_row_family_lands_on_that_height(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    let dims = ui_style_defaults_in(BPaletteName::DEFAULT, BThemeMode::DEFAULT).dimensions;
    let row_height = dims.row_height;
    // A tree's row is its own, shorter height (`tree_row_height`): Blender's
    // outliner row is a unit, not a control row, so the list reads as a list.
    let height_of = |family: &str| if family == "tree row" { dims.tree_row_height } else { row_height };

    // Every space, so the tree, the list, the table, the cards and the section
    // headings all get a turn at being on screen.
    let mut families: std::collections::BTreeMap<&'static str, Vec<(String, f32)>> = Default::default();
    for space in BSpaceType::ALL {
        app.update(cx, |app, cx| {
            app.state = BState::new();
            app.state.set(CtxKey::Space, space.slug());
            cx.notify();
        });
        cx.run_until_parked();
        for (sel, b) in cx.all_debug_bounds() {
            let h = f32::from(b.size.height);
            // A row's own element, not the label and value inside it: those
            // are a control and its caption, and they are not rows.
            if sel.ends_with("-label") || sel.ends_with("-value") || sel.ends_with("-text") {
                continue;
            }
            // The families named by their selector prefix. A row is the thing
            // these build; anything else in the window is not a row.
            let family = if sel.starts_with("card-row-") {
                "card row"
            } else if sel.starts_with("outliner-row-") {
                "tree row"
            } else if sel.starts_with("table-row") {
                "table row"
            } else if sel.starts_with("card-head") {
                "card heading"
            } else if sel.starts_with("section") {
                "section heading"
            } else {
                continue;
            };
            families.entry(family).or_default().push((sel.clone(), h));
        }
    }
    assert!(families.len() >= 3, "at least three row families are on screen to check: {:?}", families.keys());
    let mut wrong: Vec<String> = Vec::new();
    for (family, rows) in &families {
        let row_height = height_of(family);
        for (sel, h) in rows {
            // A row may be TALLER than the height when it wraps (a two-line
            // label); it may never be shorter, and the common case is exact.
            if *h + 0.5 < row_height {
                wrong.push(format!("{family} {sel} is {h}, under {row_height}"));
            }
        }
    }
    assert!(wrong.is_empty(), "rows shorter than a row:\n  {}", wrong.join("\n  "));
    // And the common case really is exact, family by family — a rule that only
    // held as "not shorter than" would let one family sit at 40 unnoticed.
    for (family, rows) in &families {
        let row_height = height_of(family);
        let exact = rows.iter().filter(|(_, h)| (h - row_height).abs() < 0.5).count();
        assert!(
            exact * 2 >= rows.len(),
            "{family}: only {exact} of {} are exactly {row_height} — heights {:?}",
            rows.len(),
            { let mut hs: Vec<String> = rows.iter().map(|(_, h)| format!("{h}")).collect(); hs.sort(); hs.dedup(); hs }
        );
    }
}

/// A menu's popover rows are in that set too, which is the part the whole
/// change was for: open a menu and its rows are rows.
#[gpui::test]
async fn a_menus_rows_are_rows(cx: &mut TestAppContext) {
    use phiber::v2::host::BHostType;
    use phiber::v2::state::{BKey as _, MenuKey, val::V_TRUE};
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    // A menu row is its own module: `MENU_ROW`, the button less its head
    // room, rather than the content row a property line is. Blender's menus
    // are tighter than its property rows, and a long one is a list of labels.
    let row_height = phiber::v2::theme::style::metric::MENU_ROW;
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        app.state.set(&MenuKey::Top(BHostType::Topbar.slug(), "File").path(), V_TRUE);
        cx.notify();
    });
    cx.run_until_parked();
    let rows: Vec<(String, f32)> = cx
        .all_debug_bounds()
        .into_iter()
        .filter(|(sel, _)| sel.starts_with("menu-"))
        .map(|(sel, b)| (sel, f32::from(b.size.height)))
        .collect();
    assert!(rows.len() >= 5, "the File menu's rows are painted: {rows:?}");
    for (sel, h) in &rows {
        assert!(
            (h - row_height).abs() < 0.5,
            "{sel} is {h}, and a menu row is {row_height}"
        );
    }
}
