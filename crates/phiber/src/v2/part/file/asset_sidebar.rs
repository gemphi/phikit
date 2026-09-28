//! Asset-browser tools region: library selector + "Catalogs" panel
//! (All / Unassigned), Blender's `asset_catalog_tree_view`.

use super::sidebar::panel;
use crate::v2::state::{BPath, BState};
use crate::v2::view::prim::{BAction, BCard, BColumn, BControl, with_peers};
use crate::v2::view::BView;

/// `ctx.asset.catalog` — the active catalog filter ("all" / "unassigned" / path).
pub fn k_catalog() -> BPath {
    BPath::owned(vec!["asset", "catalog"])
}

/// `ctx.asset.library` — the active asset library name.
pub fn k_library() -> BPath {
    BPath::owned(vec!["asset", "library"])
}

/// Blender's built-in library choices (`ASSET_LIBRARY_*` in `asset_library.hh`).
pub const LIBRARIES: &[&str] = &["Current File", "Essentials", "All"];

fn catalog_button(s: &BState, icon: &'static str, label: &'static str, value: &'static str) -> BView {
    let active = s.get(&k_catalog()).unwrap_or("all") == value;
    BView::Button(
        BControl::new()
            .with_icon(if active { "check" } else { icon })
            .with_label(label)
            .with_action(BAction::Set(k_catalog(), value.to_string())),
    )
}

/// Asset mode — library selector + "Catalogs" panel (All / Unassigned).
pub fn asset_view(s: &BState) -> BView {
    let current = s.get(&k_library()).unwrap_or(LIBRARIES[0]);
    let library_rows: Vec<BView> = LIBRARIES
        .iter()
        .map(|lib| {
            BView::Button(
                BControl::new()
                    .with_icon(if *lib == current { "check" } else { "asset" })
                    .with_label(*lib)
                    .with_action(BAction::Set(k_library(), lib.to_string())),
            )
        })
        .collect();
    BView::from(BCard::column(BColumn::Sidebar, with_peers(vec![
            panel(s, "Library", library_rows),
            panel(
                s,
                "Catalogs",
                vec![
                    catalog_button(s, "asset", "All", "all"),
                    catalog_button(s, "help_circle", "Unassigned", "unassigned"),
                ],
            ),
        ]),))
}
