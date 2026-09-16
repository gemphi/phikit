//! `BSpaceType` — workspace tab tag. Mirrors Blender 5.2.

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
    /// `Modeling` and `Sculpting` are intentionally absent: their specs stay
    /// in `SET` (the enum is `#[repr(u8)]` and `spec()` indexes `SET` by
    /// discriminant, so removing rows there would misindex every later
    /// space), they are simply not offered as tabs. `from_slug` iterates this
    /// list, so a stale `ctx.space = "Modeling"` resolves to `None` and
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

    pub fn spec(self) -> &'static super::BSpaceSpec {
        &super::SET[self as usize]
    }
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
