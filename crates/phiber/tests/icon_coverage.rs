//! Every icon the UI asks for must exist as an asset.
//!
//! A missing SVG does not fail loudly — GPUI just paints nothing, so a whole
//! header can quietly fall back to blank or repeated glyphs. This walks the
//! real view tree for every space and every modal, collects every icon name
//! the tree actually asks for, and checks each one against `assets/icons`.

use phiber::v2::state::CtxKey;
use phiber::v2::modal::{BModalItem, BModalType};
use phiber::v2::view::BViewItem as _;
use phiber::v2::space::{BSpaceList, BSpaceType};
use phiber::v2::state::{BState};
use phiber::v2::view::draw::brand;
use phiber::v2::view::{BView, BViewList};
use std::collections::BTreeSet;

/// Every icon name reachable from one view tree.
fn icons(v: &BView, out: &mut BTreeSet<String>) {
    let mut push = |name: &Option<String>| {
        if let Some(n) = name.as_deref().filter(|n| !n.is_empty()) {
            out.insert(n.to_string());
        }
    };
    match v {
        BView::Icon { name } if !name.is_empty() => {
            out.insert(name.clone());
        }
        BView::Button(ctl) | BView::MenuItem(ctl) | BView::Tab(ctl) | BView::MenuAnchor(ctl) => {
            push(&ctl.icon)
        }
        BView::Rail(rail) => rail.entries.iter().for_each(|e| push(&e.control.icon)),
        BView::Thumbnail(t) if !t.icon.is_empty() => {
            out.insert(t.icon.clone());
        }
        _ => {}
    }
    for child in children(v) {
        icons(child, out);
    }
}

fn children(v: &BView) -> Vec<&BView> {
    match v {
        BView::Space { list, .. }
        | BView::Zone { list, .. }
        | BView::Host { list, .. }
        | BView::Region { list, .. }
        | BView::Pane { list, .. }
        | BView::Row { list }
        | BView::ListRow { list, .. }
        | BView::Column { list }
        | BView::Grid { list } => list.iter().collect(),
        BView::Card(card) if matches!(card.kind, phiber::v2::view::prim::BCardKind::Column(_)) => card.body.iter().collect(),
        BView::Area { child, .. }
        | BView::Part { body: child, .. }
        | BView::Modal { body: child, .. } => {
            vec![child.as_ref()]
        }
        BView::Popover { anchor, body, .. } => {
            let mut out = vec![anchor.as_ref()];
            out.extend(body.iter());
            out
        }
        BView::TableRow { cell_list, .. } | BView::TableHeader { cell_list } => {
            cell_list.iter().map(|c| c.view.as_ref()).collect()
        }
        _ => Vec::new(),
    }
}

/// Icons asked for by every space and every modal, with menus opened so the
/// items inside them are reached too.
fn every_icon() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for space in BSpaceType::ALL.iter().copied() {
        let mut s = BState::new();
        s.set(CtxKey::Space, space.slug());
        for view in BSpaceList::new().items(&s) {
            icons(&view, &mut out);
        }
    }
    for kind in BModalType::ALL.iter().copied() {
        let s = BState::new();
        icons(&BModalItem::new(kind).build(&s), &mut out);
    }
    out
}

#[test]
fn every_icon_the_ui_asks_for_has_an_asset() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/icons");
    let missing: Vec<String> = every_icon()
        .into_iter()
        .filter(|name| !brand::is_brand(name))
        .filter(|name| {
            let file = name.replace('-', "_").replace(' ', "_").to_lowercase();
            !root.join(format!("{file}.svg")).exists()
        })
        .collect();
    assert!(
        missing.is_empty(),
        "{} icon(s) have no asset, so they paint blank:\n  {}",
        missing.len(),
        missing.join("\n  ")
    );
}

/// Icons named directly in a `svg().path("icons/<name>.svg")` call rather than
/// through a `BControl`. The gizmo builds its buttons that way, so the view
/// walk above never sees them — and two of them had no asset at all.
#[test]
fn every_icon_named_in_source_has_an_asset_too() {
    fn walk(dir: &std::path::Path, out: &mut Vec<(String, String)>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            for piece in text.split("icons/").skip(1) {
                let Some(name) = piece.split(".svg").next() else {
                    continue;
                };
                // Only literal names; a formatted one is resolved at runtime.
                if name.is_empty() || !name.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
                {
                    continue;
                }
                out.push((
                    name.to_string(),
                    path.display().to_string(),
                ));
            }
        }
    }
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut named = Vec::new();
    walk(&root.join("src"), &mut named);
    let icons = root.join("assets/icons");
    let missing: Vec<String> = named
        .into_iter()
        .filter(|(name, _)| !icons.join(format!("{name}.svg")).exists())
        .map(|(name, at)| format!("{name} (named in {at})"))
        .collect();
    assert!(
        missing.is_empty(),
        "{} icon(s) named in source have no asset:\n  {}",
        missing.len(),
        missing.join("\n  ")
    );
}

/// Every icon is drawn on the same canvas and inherits the text colour, so a
/// strip of them lines up and themes together.
#[test]
fn every_icon_shares_one_canvas_and_takes_the_theme_colour() {
    let icons = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/icons");
    let mut wrong = Vec::new();
    for entry in std::fs::read_dir(&icons).expect("icons").flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("svg") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        // One canvas for every icon: a 16-unit drawing is scaled into the
        // 24-unit box the rest use, so a strip of icons renders at one size.
        if !text.contains("viewBox=\"0 0 24 24\"") {
            wrong.push(format!("{name}: not on the shared 24-unit canvas"));
        }
        if !text.contains("currentColor") {
            wrong.push(format!("{name}: does not take the theme colour"));
        }
    }
    assert!(wrong.is_empty(), "{} icon(s) off-spec:\n  {}", wrong.len(), wrong.join("\n  "));
}

/// No icon the UI shows may be a placeholder.
///
/// Most of the set was stubbed out with one of two drawings — a bare circle
/// or a rounded square — so whole rows of the Properties editor and the
/// toolbars rendered as the same glyph. Genuine synonyms (two names for one
/// idea) are allowed in pairs; three or more names sharing a drawing is a
/// stub, not a synonym.
#[test]
fn no_icon_the_ui_shows_is_a_placeholder() {
    use std::collections::BTreeMap;
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut by_drawing: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for entry in std::fs::read_dir(root.join("assets/icons")).expect("icons").flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("svg") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let body = text
            .split_once('>')
            .map(|(_, rest)| rest.replace("</svg>", ""))
            .unwrap_or_default();
        let body = body.split_whitespace().collect::<Vec<_>>().join(" ");
        by_drawing
            .entry(body)
            .or_default()
            .push(path.file_stem().unwrap().to_string_lossy().to_string());
    }
    let shown = every_icon();
    let stubs: Vec<String> = by_drawing
        .into_values()
        .filter(|names| names.len() >= 3)
        .flat_map(|names| {
            names
                .into_iter()
                .filter(|name| shown.contains(name))
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(
        stubs.is_empty(),
        "{} icon(s) on screen are still the placeholder drawing:\n  {}",
        stubs.len(),
        stubs.join("\n  ")
    );
}
