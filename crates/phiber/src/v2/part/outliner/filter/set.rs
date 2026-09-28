//! Header controls — `bl_ui`'s `OUTLINER_HT_header`.
//!
//! Blender ref: `bl_ui/space_outliner.py` and
//! `editors/space_outliner/`.


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BOutlineFilterSpec {
    pub icon: &'static str,
    pub label: &'static str,
    pub key: &'static str,
}

pub static FILTERS: &[BOutlineFilterSpec] = &[
    BOutlineFilterSpec {
        icon: "filter",
        label: "Search",
        key: "filter_text",
    },
    BOutlineFilterSpec {
        icon: "case_sensitive",
        label: "Case Sensitive",
        key: "use_filter_case_sensitive",
    },
    BOutlineFilterSpec {
        icon: "whole_word",
        label: "Exact Match",
        key: "use_filter_complete",
    },
    BOutlineFilterSpec {
        icon: "collection",
        label: "Collections",
        key: "use_filter_collection",
    },
    BOutlineFilterSpec {
        icon: "object",
        label: "Objects",
        key: "use_filter_object",
    },
    BOutlineFilterSpec {
        icon: "git_branch",
        label: "Object Children",
        key: "use_filter_children",
    },
];
