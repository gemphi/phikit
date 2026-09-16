//! Outliner data model.
//!
//! Feature: phiber-blender-space-parity
//! Requirements: 14.1, 15.1, 18.1, 18.2

#![allow(dead_code)]

/// Blender's `SpaceOutliner.display_mode` — six variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BOutlineMode {
    ViewLayer,
    Scenes,
    Sequence,
    LibraryOverrides,
    DataApi,
    OrphanData,
}

impl BOutlineMode {
    pub const ALL: &'static [BOutlineMode] = &[
        Self::ViewLayer,
        Self::Scenes,
        Self::Sequence,
        Self::LibraryOverrides,
        Self::DataApi,
        Self::OrphanData,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Self::ViewLayer => "View_Layer",
            Self::Scenes => "Scenes",
            Self::Sequence => "Sequence",
            Self::LibraryOverrides => "Library_Overrides",
            Self::DataApi => "Data_Api",
            Self::OrphanData => "Orphan_Data",
        }
    }

    pub fn from_slug(s: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|m| m.slug() == s)
    }
}

/// Row kind — coarse categorization used by filter and by icon selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BOutlineRowKind {
    SceneCollection,
    Collection,
    Object,
    Mesh,
    Curve,
    Armature,
    Bone,
    Material,
    Texture,
    Light,
    Camera,
    Empty,
    Modifier,
    Constraint,
    Datablock,
    Other,
}

/// Filter popover state.
#[derive(Debug, Clone, PartialEq)]
pub struct BOutlineFilter {
    pub search: String,
    pub case_sensitive: bool,
    pub exact_match: bool,
    pub show_collections: bool,
    pub show_objects: bool,
    pub show_object_children: bool,
}

impl Default for BOutlineFilter {
    fn default() -> Self {
        Self {
            search: String::new(),
            case_sensitive: false,
            exact_match: false,
            show_collections: true,
            show_objects: true,
            show_object_children: true,
        }
    }
}

/// A single row in the outliner tree.
#[derive(Debug, Clone, PartialEq)]
pub struct BOutlineRow {
    pub id: String,
    pub kind: BOutlineRowKind,
    pub label: String,
    pub children: Vec<BOutlineRow>,
    pub hide: bool,
    pub disable_selection: bool,
    pub disable_in_viewports: bool,
    pub disable_in_renders: bool,
    pub holdout: bool,
    pub indirect_only: bool,
}

impl BOutlineRow {
    pub fn leaf(id: impl Into<String>, kind: BOutlineRowKind, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind,
            label: label.into(),
            children: Vec::new(),
            hide: false,
            disable_selection: false,
            disable_in_viewports: false,
            disable_in_renders: false,
            holdout: false,
            indirect_only: false,
        }
    }

    pub fn with_children(mut self, children: Vec<BOutlineRow>) -> Self {
        self.children = children;
        self
    }
}

/// A tree — a root row plus the current display mode.
#[derive(Debug, Clone, PartialEq)]
pub struct BOutlineTree {
    pub root: BOutlineRow,
    pub mode: BOutlineMode,
}

impl BOutlineTree {
    pub fn new(root: BOutlineRow, mode: BOutlineMode) -> Self {
        Self { root, mode }
    }

    /// Look up a row by id via a depth-first scan.
    pub fn find(&self, id: &str) -> Option<&BOutlineRow> {
        find_in(&self.root, id)
    }

    /// Look up a row by id, returning a mutable reference.
    pub fn find_mut(&mut self, id: &str) -> Option<&mut BOutlineRow> {
        find_in_mut(&mut self.root, id)
    }
}

fn find_in<'a>(r: &'a BOutlineRow, id: &str) -> Option<&'a BOutlineRow> {
    if r.id == id {
        return Some(r);
    }
    for c in &r.children {
        if let Some(f) = find_in(c, id) {
            return Some(f);
        }
    }
    None
}

fn find_in_mut<'a>(r: &'a mut BOutlineRow, id: &str) -> Option<&'a mut BOutlineRow> {
    if r.id == id {
        return Some(r);
    }
    for c in &mut r.children {
        if let Some(f) = find_in_mut(c, id) {
            return Some(f);
        }
    }
    None
}

/// Drop target for reparent — nested into a collection, or sibling of a row.
#[derive(Debug, Clone, PartialEq)]
pub enum BDropTarget {
    Into(String),
    Sibling { of: String, after: bool },
}

/// Error returned by `reparent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BReparentError {
    DescendantOfSelf,
    RootImmutable,
    UnknownId,
}

/// Error returned by `rename`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BRenameError {
    RootImmutable,
    UnknownId,
    Empty,
}
