//! The style budget: how much hand-written GPUI styling each renderer file
//! still carries, held so it can only go down.
//!
//! Layout and surface decisions belong in one place — `BFlex`, answered by
//! each level's `BFlexed` impl — not in `div().flex().flex_1().bg(…)` chains
//! written wherever a container happens to be built. A *raw* style call is a
//! GPUI style method given a pixel value, a colour or nothing; a call given a
//! `B…` token (`.gap(BSpace::GapXs)`) is `BFlex`'s own builder and is the point.
//!
//! Every file's count is recorded in `golden/style_budget.txt`. A file may
//! never exceed its budget and a new file starts at zero, so the rest of the
//! renderer moves onto `BFlex` one file at a time and cannot slide back.
//! When a migration lowers a count, record the new budget:
//!     UPDATE_STYLE_BUDGET=1 cargo test -p phiber --test style_budget
//!
//! The budget went *up* once, in five files, when `border_1` and
//! `rounded_full` were added to `STYLE`. Those had been invisible here, so a
//! hidden literal like `border_1()` was the cheap way past this budget while
//! the token-driven `.border(px(strokes.border))` cost a point. No code
//! regressed in that pass; the ruler got more accurate. Widening the ruler
//! again is the only reason a budget may rise, and it needs saying here.
//!
//! `popover/leave.rs` earned an entry when the popover's dismissal backdrop
//! moved into it from `popover/mod.rs` — the file that already owned the
//! pointer-strayed-away test now owns the layer that asks it. That is the
//! same calls in a different file, not new ones: `popover/mod.rs` fell from
//! 42 to 39 in the same pass.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const BUDGET: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/style_budget.txt");

/// GPUI's style builders. `BFlex::div` is the one place allowed to call them.
const STYLE: &[&str] = &[
    "flex", "flex_col", "flex_row", "flex_1", "flex_grow", "flex_grow_1", "flex_shrink",
    "flex_shrink_0", "flex_basis", "flex_wrap", "flex_none", "items_start", "items_end",
    "items_center", "items_stretch", "items_baseline", "justify_start", "justify_end",
    "justify_center", "justify_between", "justify_around", "self_start", "self_end",
    "self_center", "gap", "gap_x", "gap_y", "p", "px", "py", "pt", "pb", "pl", "pr", "m", "mx",
    "my", "mt", "mb", "ml", "mr", "bg", "border", "border_t", "border_b", "border_l", "border_r",
    "border_color", "rounded", "rounded_t", "rounded_b", "rounded_l", "rounded_r", "rounded_tl",
    "rounded_tr", "rounded_bl", "rounded_br", "w", "h", "w_full", "h_full", "size", "size_full",
    "min_w", "min_h", "min_w_0", "min_h_0", "max_w", "max_h", "overflow_hidden",
    "overflow_x_hidden", "overflow_y_hidden", "overflow_scroll", "overflow_y_scroll",
    "text_color", "text_size", "shadow", "shadow_sm", "shadow_md", "shadow_lg", "shadow_xl",
    "absolute", "relative", "inset_0", "top", "bottom", "left", "right", "opacity",
    // GPUI's numeric shorthands carry the literal in the method name, so they
    // read as token-free while spending a design decision. Counting them is
    // what stops `border_1()` from being the cheap way past this budget.
    "border_1", "rounded_full",
];

/// The ones GPUI calls without an argument. Every other name in `STYLE` takes
/// one in GPUI, so a bare `.size()` is some other type's method (a trait's
/// `size()`), not styling.
const NO_ARG: &[&str] = &[
    "flex", "flex_col", "flex_row", "flex_1", "flex_grow_1", "flex_shrink_0", "flex_wrap",
    "flex_none", "items_start", "items_end", "items_center", "items_stretch", "items_baseline",
    "justify_start", "justify_end", "justify_center", "justify_between", "justify_around",
    "self_start", "self_end", "self_center", "w_full", "h_full", "size_full", "min_w_0",
    "min_h_0", "overflow_hidden", "overflow_x_hidden", "overflow_y_hidden", "overflow_scroll",
    "overflow_y_scroll", "shadow", "shadow_sm", "shadow_md", "shadow_lg", "shadow_xl",
    "absolute", "relative", "inset_0", "rounded_t", "rounded_b", "border_1", "rounded_full",
];

/// The layout element itself, where the style calls belong: `BFlex::div`,
/// the two stateful containers built on it, and the one function that turns
/// an elevation into a GPUI shadow rung.
const THE_ELEMENT: &[&str] =
    &["flex/apply.rs", "flex/place.rs", "flex/stateful.rs", "flex/paint.rs"];

fn raw_calls(src: &str) -> usize {
    let src = match src.find("#[cfg(test)]") {
        Some(at) => &src[..at],
        None => src,
    };
    let mut n = 0;
    for line in src.lines() {
        let line = line.trim_start();
        if line.starts_with("//") {
            continue;
        }
        let bytes = line.as_bytes();
        let mut i = 0;
        while let Some(dot) = line[i..].find('.') {
            let start = i + dot + 1;
            let name_end = line[start..]
                .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
                .map_or(line.len(), |e| start + e);
            i = name_end;
            if name_end >= line.len() || bytes[name_end] != b'(' {
                continue;
            }
            let name = &line[start..name_end];
            if !STYLE.contains(&name) {
                continue;
            }
            // `BFlex`'s builder takes a token (`BSpace::…`); the trait method
            // `.flex(ctx)` takes the context. Neither is raw styling.
            let args = &line[name_end + 1..];
            if args.starts_with('B') || args.starts_with("c)") || args.starts_with("ctx)") || args.starts_with("_c)") {
                continue;
            }
            if args.starts_with(')') && !NO_ARG.contains(&name) {
                continue;
            }
            n += 1;
        }
    }
    n
}

fn walk(dir: &Path, root: &Path, out: &mut BTreeMap<String, usize>) {
    for entry in std::fs::read_dir(dir).expect("renderer dir").flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, root, out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let rel = path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
        if THE_ELEMENT.contains(&rel.as_str()) || rel.ends_with("tests.rs") || rel.contains("/tests/") {
            continue;
        }
        let n = raw_calls(&std::fs::read_to_string(&path).expect("source"));
        if n > 0 {
            out.insert(rel, n);
        }
    }
}

fn measured() -> BTreeMap<String, usize> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/v2/view/draw");
    let mut out = BTreeMap::new();
    walk(&root, &root, &mut out);
    out
}

#[test]
fn no_renderer_file_writes_more_raw_style_than_its_budget() {
    let now = measured();
    let total: usize = now.values().sum();
    if std::env::var("UPDATE_STYLE_BUDGET").is_ok() {
        let text: String = now.iter().map(|(f, n)| format!("{n:4} {f}\n")).collect();
        std::fs::write(BUDGET, text).expect("write budget");
        eprintln!("recorded budget: {total} raw style calls in {} files", now.len());
        return;
    }
    let budget: BTreeMap<String, usize> = std::fs::read_to_string(BUDGET)
        .expect("no budget — record one with UPDATE_STYLE_BUDGET=1")
        .lines()
        .filter_map(|l| {
            let mut it = l.split_whitespace();
            Some((it.nth(1)?.to_string(), l.split_whitespace().next()?.parse().ok()?))
        })
        .collect();
    let over: Vec<String> = now
        .iter()
        .filter(|(f, n)| **n > budget.get(*f).copied().unwrap_or(0))
        .map(|(f, n)| format!("{f}: {n} raw style calls (budget {})", budget.get(f).copied().unwrap_or(0)))
        .collect();
    assert!(
        over.is_empty(),
        "raw GPUI styling grew — describe the container with BFlex instead:\n  {}",
        over.join("\n  ")
    );
}

/// The levels the refactor moved onto `BFlex` carry no raw styling of their
/// own. What is left in them is part-level controls (a seam's line, an icon's
/// size), which move with the parts.
#[test]
fn the_migrated_levels_describe_their_containers_with_bflex() {
    let now = measured();
    for file in [
        "space/mod.rs", "zone/mod.rs", "area/mod.rs", "host/mod.rs", "region/mod.rs", "pane/mod.rs", "modal/mod.rs",
        "modal/card.rs", "modal/fullscreen.rs", "modal/welcome.rs",
        // The popover joined them: it was 39 raw calls, a hand-built panel
        // that drifted from the cards beside it every time either moved. It
        // is a `BPopoverPart` now, so a menu, a filter popover and an
        // accordion are three settings of one thing.
        "popover/mod.rs",
    ] {
        assert_eq!(now.get(file).copied().unwrap_or(0), 0, "{file} writes raw GPUI style");
    }
}
