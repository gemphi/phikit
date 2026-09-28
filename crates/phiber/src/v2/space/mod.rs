//! Space — workspace tab.
//!
//! Every workspace owns a folder: `space/<name>/mod.rs` is the structure and
//! `space/<name>/set.rs` is the content (its areas and the hosts they offer).
//! `set.rs` here is only the index that puts them in `BSpaceType` order.

pub mod set;
pub mod animation;
pub mod compositing;
pub mod geometry_nodes;
pub mod layout;
pub mod modeling;
pub mod rendering;
pub mod scripting;
pub mod sculpting;
pub mod shading;
pub mod texture_paint;
pub mod uv_editing;
pub mod step;
pub mod switch;

use crate::v2::area::BAreaSpec;
use crate::v2::modal::BModalList;
use crate::v2::spec::BSpec;
use crate::v2::spec::BSpecKind;
use crate::v2::state::BState;
use crate::v2::state::CtxKey;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;
use crate::v2::zone::BZoneList;
use crate::v2::zone::BZoneType;
pub use switch::switch_space;

// `BSpaceType` — workspace tab tag. Mirrors Blender 5.2.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BSpaceType {
    Layout = 0,
    Modeling = 1,
    Sculpting = 2,
    UvEditing = 3,
    TexturePaint = 4,
    Shading = 5,
    Animation = 6,
    Rendering = 7,
    Compositing = 8,
    GeometryNodes = 9,
    Scripting = 10,
}

impl BSpaceType {
    /// Every space that exists, including the ones not offered as tabs.
    /// `from_slug` resolves against this, so a space can still be addressed
    /// by slug (tests, saved state, deep links) after being removed from the
    /// tab strip.
    pub const EVERY: &'static [BSpaceType] = &[
        BSpaceType::Layout,
        BSpaceType::Modeling,
        BSpaceType::Sculpting,
        BSpaceType::UvEditing,
        BSpaceType::TexturePaint,
        BSpaceType::Shading,
        BSpaceType::Animation,
        BSpaceType::Rendering,
        BSpaceType::Compositing,
        BSpaceType::GeometryNodes,
        BSpaceType::Scripting,
    ];

    /// Workspaces offered in the top tab strip.
    ///
    /// `Modeling` and `Sculpting` are intentionally absent: they exist and
    /// have specs, they are simply not offered as tabs. `from_slug` iterates
    /// this list, so a stale `ctx.space = "Modeling"` resolves to `None` and
    /// `BSpaceList::active` falls back to `Layout`.
    pub const ALL: &'static [BSpaceType] = &[
        BSpaceType::Layout,
        BSpaceType::UvEditing,
        BSpaceType::TexturePaint,
        BSpaceType::Shading,
        BSpaceType::Animation,
        BSpaceType::Rendering,
        BSpaceType::Compositing,
        BSpaceType::GeometryNodes,
        BSpaceType::Scripting,
    ];

    pub fn label(self) -> &'static str {
        self.spec().label
    }
    pub fn icon(self) -> &'static str {
        self.spec().icon
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::Layout => "Layout",
            Self::Modeling => "Modeling",
            Self::Sculpting => "Sculpting",
            Self::UvEditing => "UvEditing",
            Self::TexturePaint => "TexturePaint",
            Self::Shading => "Shading",
            Self::Animation => "Animation",
            Self::Rendering => "Rendering",
            // The SLUG is this space's stable identity: it is what
            // `ctx.space` persists, what the seed writes, and what
            // `load_ui::space_for` matches a Blender file's workspace name
            // against. Renaming the space to "ECS Wand" is a LABEL change —
            // renaming the slug too made `from_slug("Compositing")` return
            // None, so a .blend's Compositing workspace no longer resolved
            // and the welcome window's template lost its action.
            Self::Compositing => "Compositing",
            Self::GeometryNodes => "GeometryNodes",
            Self::Scripting => "Scripting",
        }
    }
    pub fn from_slug(s: &str) -> Option<Self> {
        Self::EVERY.iter().copied().find(|t| t.slug() == s)
    }

    /// Whether this space appears in the top tab strip.
    pub fn is_tab(self) -> bool {
        Self::ALL.contains(&self)
    }
}

// `BSpaceSpec` — one space with its area set inline.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BSpaceSpec {
    pub kind: BSpaceType,
    pub label: &'static str,
    pub icon: &'static str,
    pub areas: &'static [BAreaSpec],
}

impl BSpec for BSpaceSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Space(self.kind)
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

impl BSpaceSpec {
    /// Areas whose slot sits inside the given zone.
    pub fn areas_in(&self, zone: BZoneType) -> &[BAreaSpec] {
        crate::v2::zone::areas_in(self.areas, zone)
    }
}

// `BSpaceItem` — one space.

#[derive(Clone, Copy, Debug)]
pub struct BSpaceItem {
    pub space: BSpaceType,
}

impl BSpaceItem {
    pub fn new(space: BSpaceType) -> Self {
        Self { space }
    }
}

impl BViewItem for BSpaceItem {
    fn build(&self, state: &BState) -> BView {
        // Record the active space for the whole subtree — area slots are
        // keyed per space so one workspace's editor choice cannot leak into
        // another's (see `v2::area::current`).
        crate::v2::area::with_space(self.space, || BView::Space {
            kind: self.space,
            list: BZoneList::new(self.space.spec()).items(state),
        })
    }
}

// `BSpaceList` — the app root. Emits the active `BSpaceItem` and, if
// `ctx.modal` is set, the modal overlay as a sibling on top.

#[derive(Clone, Copy, Debug, Default)]
pub struct BSpaceList;

impl BSpaceList {
    pub fn new() -> Self {
        Self
    }
    pub fn active(state: &BState) -> BSpaceType {
        state
            .get(CtxKey::Space)
            .and_then(BSpaceType::from_slug)
            .unwrap_or(BSpaceType::Layout)
    }
}

impl BViewList for BSpaceList {
    fn items(&self, state: &BState) -> Vec<BView> {
        let mut out = vec![BSpaceItem::new(Self::active(state)).build(state)];
        out.extend(BModalList::new().items(state));
        out
    }
}

// Per-space content — one file per Blender workspace.

// ── Layout (was `view/render/space.rs`) ──
// Space-level layout: the app root. One space is always active; it stacks
// its zones vertically (Top strip → Middle band → Bottom strip → global
// Statusbar).
use crate::v2::view::layout::{BAxis, BSpaceLayout};

impl BSpaceLayout for BSpaceType {
    /// Space has no per-space layout choices — every space is a column of
    /// zones. Zone kinds control their own children's orientation.
    fn axis(&self) -> BAxis {
        BAxis::Column
    }
}
