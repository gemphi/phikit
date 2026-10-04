//! A function that takes a view as its first argument is a method of that view.
//!
//! The convention is the codebase's own, written on `BCtrlView`
//! (`view/draw/ctrl/mod.rs`): *"Each kind is a method of this type rather than a
//! free function beside it, so the widget a kind draws as is reached through the
//! control, not from anywhere."*
//!
//! It had drifted in the one place where it costs most. `BTimeView` carried four
//! `impl` blocks while **nine** `paint*(view: &BTimeView, …)` functions sat at
//! module level across eight `set.rs` files — and two of those files had an
//! `impl BTimeView` in their own sibling `mod.rs`, so the type's behaviour was
//! split across the same folder by nothing but which file someone opened.
//! `BImageView` had four more. `BCtrlView::render` reached a free `number()` ten
//! lines under the sentence above.
//!
//! This guard is deliberately narrow. It checks the two canvas view types that
//! had the problem, by name, rather than inventing a rule for every type in the
//! tree — a receiver-first function is not wrong in general (`chrome::paint`
//! takes an element and wraps it; `track::draggable` does the same), and a guard
//! that could not tell those apart would be noise. What it can say precisely is:
//! these two types own their painting.

use std::collections::BTreeSet;

/// Every `.rs` under a folder, read at compile time is not possible, so walk it.
fn sources(root: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut stack = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(root)];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(text) = std::fs::read_to_string(&path) {
                    out.push((path.display().to_string(), text));
                }
            }
        }
    }
    out
}

/// Module-level `fn NAME(view: &TYPE, …)` — a method written outside its type.
fn loose_receivers(root: &str, ty: &str) -> Vec<String> {
    let needle = format!("(view: &{ty},");
    let mut found = Vec::new();
    for (path, text) in sources(root) {
        for line in text.lines() {
            // A method is indented inside its `impl`; a free function starts at
            // column zero. That is the whole distinction, and it is exact.
            if line.starts_with("pub fn ") || line.starts_with("fn ") || line.starts_with("pub(") {
                if line.contains(&needle) {
                    let name =
                        line.split("fn ").nth(1).and_then(|r| r.split('(').next()).unwrap_or(line);
                    found.push(format!("{path}: {name}"));
                }
            }
        }
    }
    found
}

#[test]
fn the_time_canvas_paints_through_its_view() {
    let loose = loose_receivers("src/v2/view/draw/canvas/time", "BTimeView");
    assert!(
        loose.is_empty(),
        "these take a `&BTimeView` first and are not methods of it:\n  {}\n\
         Move each into an `impl BTimeView` block in its own file.",
        loose.join("\n  "),
    );
}

#[test]
fn the_image_canvas_paints_through_its_view() {
    let loose = loose_receivers("src/v2/view/draw/canvas/image", "BImageView");
    assert!(loose.is_empty(), "not methods of `BImageView`:\n  {}", loose.join("\n  "));
}

#[test]
fn the_control_renderer_reaches_every_kind_through_itself() {
    // `BCtrlView::render` is a match whose every arm is `self.<kind>(ctx)`.
    // One arm used to call a free `number(self.0, ctx)`.
    let ctrl = include_str!("../src/v2/view/draw/ctrl/mod.rs");
    let from = ctrl.find("fn render(").expect("the control renderer");
    let body = &ctrl[from..];
    let body = &body[..body.find("\n    }\n").map_or(body.len(), |i| i)];
    let strays: Vec<&str> = body
        .lines()
        .filter(|l| l.contains(" => "))
        .filter(|l| !l.contains("self."))
        .collect();
    assert!(
        strays.is_empty(),
        "`BCtrlView::render` reaches these from outside itself:\n  {}",
        strays.join("\n  "),
    );
}

#[test]
fn the_two_canvases_still_have_the_methods_they_gained() {
    // The sweep is only worth guarding if the methods are actually there —
    // a guard that passes because the functions were deleted is no guard.
    let time: BTreeSet<String> = sources("src/v2/view/draw/canvas/time")
        .into_iter()
        .flat_map(|(_, text)| {
            text.lines()
                .filter(|l| l.trim_start().starts_with("pub fn paint"))
                .filter(|l| l.contains("&self"))
                .filter_map(|l| l.split("fn ").nth(1)?.split('(').next().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .collect();
    for wanted in [
        "paint",
        "paint_frame_lines",
        "paint_value_lines",
        "paint_markers",
        "paint_cursor",
        "paint_curves",
        "paint_handles",
        "paint_curve_line",
        "paint_nla_rows",
    ] {
        assert!(time.contains(wanted), "`BTimeView::{wanted}` is missing; found {time:?}");
    }
}
