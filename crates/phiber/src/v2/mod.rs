//! Phiber v2 — flat, singular, one folder per hierarchy level.
//!
//! Three view traits describe the coordinator layer:
//!   - `view::BViewItem` — emits exactly one `BView::<level>` variant.
//!   - `view::BViewList` — iterates a store collection into `Vec<BView>`.
//!
//! One store trait describes the static side:
//!   - `spec::BSpec`     — every static spec at every level.
//!
//! Static data lives ONLY in `set.rs` files. No inline arrays/tuples in any
//! `list.rs` or `item.rs`.

pub mod spec;
pub mod state;
pub mod theme;
pub mod view;

pub mod area;
pub mod host;
pub mod modal;
pub mod pane;
pub mod part;
pub mod region;
pub mod scene;
pub mod space;
pub mod zone;

pub mod app;

pub use spec::{BSpec, BSpecKind};
pub use state::{BPath, BState};
pub use theme::{
    BDimensions, BEffects, BMetric, BPalette, BPaletteName, BPreferenceControl,
    BPreferenceMetadata, BRadiusScale, BRadiusToken, BResizeStyle, BResolvedColors,
    BResolvedUiStyle, BRoleMap, BShadowStyle, BSpacing, BStrokes, BStyleChoice, BStyleDescriptor,
    BStyleParseError, BStyleParser, BStyleRole, BStyleValue, BTypography, BUiStyleDefaults,
    METRICS, UI_STYLE_DESCRIPTORS, palette_for, resolve_ui_style, ui_style_defaults,
};
pub use view::{BView, BViewItem, BViewList};
