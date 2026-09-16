//! Hierarchy-bound state keys matching the store structure.
//!
//! Level mapping:
//!   Space  -> CtxSpace
//!   Zone   -> Zone(BZoneType)
//!   Area   -> AreaHost(BHostType)
//!   Host   -> CtxHost
//!   Region -> CtxRegion
//!   Pane   -> Tab { host, part }, Expanded { host, part }
//!   Part   -> Dropdown { host, part, key }, MenuItem { host, part, label }, Section(label)

use crate::store::hosts::BHostType;
use crate::store::parts::BPartType;
use crate::store::zones::BZoneType;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BKey {
    // ── Context Keys (ctx::*) ──
    CtxSpace,
    CtxZone,
    CtxHost,
    CtxRegion,
    CtxMode,
    CtxTool,
    CtxObject,
    CtxScene,
    CtxViewLayer,
    CtxFrame,
    CtxPlaying,
    CtxOrientation,
    CtxShading,

    // ── Switcher preview host (ui::switcher::preview_host) ──
    SwitcherPreviewHost,
    SwitcherApplyHost,

    // ── Activity Bar & Social (ui::activity::*) ──
    ActivityTab,
    ActivityDrawer,
    ActivityExpanded,
    ActiveChatUser,
    ActiveChannel,

    // ── Universal Card (ui::card::*) ──
    CardCollapsed(String),

    // ── UI Hierarchy Keys (ui::*) ──
    Theme,
    UseGradients,
    Modal,
    Zone(BZoneType),
    AreaHost(BHostType),
    Tab {
        host: BHostType,
        part: BPartType,
    },
    Expanded {
        host: BHostType,
        part: BPartType,
    },
    Dropdown {
        host: BHostType,
        part: BPartType,
        key: String,
    },
    MenuItem {
        host: BHostType,
        part: BPartType,
        label: String,
    },
    Section(String),
    PanelExpanded,

    // ── Dimensions (ui::dim::*) ──
    LeftWidth,
    RightWidth,
    RightTopHeight,
    BottomHeight,

    // ── File browser (ui::file_browser::*) ──
    FilePath,
    FileSelected,

    // ── Viewport camera (ui::camera::*) ──
    CamYaw,
    CamPitch,
    CamZoom,
    CamDragX,
    CamDragY,
}

impl BKey {
    /// Convert to hierarchical string key.
    pub fn key(&self) -> String {
        match self {
            Self::CtxSpace => "ctx::space".to_string(),
            Self::CtxZone => "ctx::zone".to_string(),
            Self::CtxHost => "ctx::host".to_string(),
            Self::CtxRegion => "ctx::region".to_string(),
            Self::CtxMode => "ctx::mode".to_string(),
            Self::CtxTool => "ctx::tool".to_string(),
            Self::CtxObject => "ctx::object".to_string(),
            Self::CtxScene => "ctx::scene".to_string(),
            Self::CtxViewLayer => "ctx::view_layer".to_string(),
            Self::CtxFrame => "ctx::frame".to_string(),
            Self::CtxPlaying => "ctx::playing".to_string(),
            Self::CtxOrientation => "ctx::orientation".to_string(),
            Self::CtxShading => "ctx::shading".to_string(),
            Self::SwitcherPreviewHost => "ui::switcher::preview_host".to_string(),
            Self::SwitcherApplyHost => "ui::switcher::apply_host".to_string(),
            Self::ActivityTab => "ui::activity::tab".to_string(),
            Self::ActivityDrawer => "ui::activity::drawer".to_string(),
            Self::ActivityExpanded => "ui::activity::expanded".to_string(),
            Self::ActiveChatUser => "ui::activity::chat_user".to_string(),
            Self::ActiveChannel => "ui::activity::channel".to_string(),
            Self::CardCollapsed(id) => format!("ui::card::{id}::collapsed"),
            Self::Theme => "ui::theme".to_string(),
            Self::UseGradients => "ui::use_gradients".to_string(),
            Self::Modal => "ui::modal".to_string(),
            Self::Zone(z) => format!("ui::zone::{:?}", z),
            Self::AreaHost(h) => format!("ui::area_host::{:?}", h),
            Self::Tab { host, part } => format!("ui::host::{:?}::part::{:?}::tab", host, part),
            Self::Expanded { host, part } => {
                format!("ui::host::{:?}::part::{:?}::expanded", host, part)
            }
            Self::Dropdown { host, part, key } => {
                format!("ui::host::{:?}::part::{:?}::dropdown::{}", host, part, key)
            }
            Self::MenuItem { host, part, label } => format!(
                "ui::host::{:?}::part::{:?}::menu_item::{}",
                host, part, label
            ),
            Self::Section(label) => format!("ui::part::Panel::section::{}", label),
            Self::PanelExpanded => "ui::part::Panel::expanded".to_string(),
            Self::LeftWidth => "ui::dim::left_width".to_string(),
            Self::RightWidth => "ui::dim::right_width".to_string(),
            Self::RightTopHeight => "ui::dim::right_top_height".to_string(),
            Self::BottomHeight => "ui::dim::bottom_height".to_string(),
            Self::FilePath => "ui::file_browser::path".to_string(),
            Self::FileSelected => "ui::file_browser::selected".to_string(),
            Self::CamYaw => "ui::camera::yaw".to_string(),
            Self::CamPitch => "ui::camera::pitch".to_string(),
            Self::CamZoom => "ui::camera::zoom".to_string(),
            Self::CamDragX => "ui::camera::drag_x".to_string(),
            Self::CamDragY => "ui::camera::drag_y".to_string(),
        }
    }
}
