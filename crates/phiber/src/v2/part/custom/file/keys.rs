//! File-browser state keys, one `pub fn` per `FileSelectParams` field.
//!
//! The crate convention is `pub const BPath` in `state/key.rs` for global
//! keys; everything under `ctx.file.*` is local to this widget, so it lives
//! here instead of leaking a dozen names into the global key table. Every
//! call site goes through these — previously ten sites spelled
//! `BPath::owned(vec!["file", "path"])` by hand and two of them disagreed
//! about what `selected` contained.

use crate::v2::state::BPath;

/// Current directory (`FileSelectParams::dir`).
pub fn k_path() -> BPath {
    BPath::owned(vec!["file", "path"])
}
/// Absolute path of the highlighted row (`FileSelectParams::file`, resolved).
/// ALWAYS a full path — the importer opens exactly this string.
pub fn k_selected() -> BPath {
    BPath::owned(vec!["file", "selected"])
}
/// Text in the filename field.
pub fn k_filename() -> BPath {
    BPath::owned(vec!["file", "filename"])
}
/// Search glob (`FileSelectParams::filter_search`).
pub fn k_search() -> BPath {
    BPath::owned(vec!["file", "search"])
}
/// Display type (`eFileDisplayType`).
pub fn k_display() -> BPath {
    BPath::owned(vec!["file", "display"])
}
/// Sort column (`eFileSortType`).
pub fn k_sort() -> BPath {
    BPath::owned(vec!["file", "sort"])
}
/// `FILE_SORT_INVERT`.
pub fn k_sort_invert() -> BPath {
    BPath::owned(vec!["file", "sort_invert"])
}
/// `FILE_HIDE_DOT` (inverted — this key is "show hidden").
pub fn k_show_hidden() -> BPath {
    BPath::owned(vec!["file", "show_hidden"])
}
/// Master filter toggle (`FILE_FILTER`).
pub fn k_use_filter() -> BPath {
    BPath::owned(vec!["file", "use_filter"])
}
/// One per-type filter flag, e.g. `ctx.file.filter_blender`.
pub fn k_filter(flag: &'static str) -> BPath {
    BPath::owned(vec!["file", flag])
}
/// Pending navigation operator (`file.previous` / `next` / `parent` /
/// `refresh`), consumed by the reducer.
pub fn k_nav() -> BPath {
    BPath::owned(vec!["file", "nav"])
}
/// Back stack, newline-separated, most recent last.
pub fn k_hist_back() -> BPath {
    BPath::owned(vec!["file", "hist_back"])
}
/// Forward stack, newline-separated, most recent last.
pub fn k_hist_fwd() -> BPath {
    BPath::owned(vec!["file", "hist_fwd"])
}
/// What the browser was opened for — drives the execute button's label.
pub fn k_purpose() -> BPath {
    BPath::owned(vec!["file", "purpose"])
}
/// Last enumeration error, if any.
pub fn k_error() -> BPath {
    BPath::owned(vec!["file", "error"])
}

/// Accordion expansion state for a sidebar category section.
/// `ctx.file.sidebar.<category>.expanded`
/// Returns the encoded key string directly since `category` is a runtime
/// value; callers use `state.get_encoded()` / `state.set_encoded()`.
pub fn k_sidebar_section(category: &str) -> String {
    format!("ctx.file.sidebar.{category}.expanded")
}

/// Open/closed state of the type-filter popover (`FILEBROWSER_PT_filter`).
pub fn k_filter_open() -> BPath {
    BPath::owned(vec!["file", "filter_open"])
}
