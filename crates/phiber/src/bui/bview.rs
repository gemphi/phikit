//! BView enum — the pure declarative view tree type and its fluent constructors.

use super::state::BKey;
use crate::store::BPanelRole;

use gpui::{App, Window};
use std::rc::Rc;

pub type ActionHandler = Rc<dyn Fn(&BAction, &mut Window, &mut App)>;

/// Actions triggered by clicking interactive elements.
/// Uses BKey for type-safe hierarchy-bound state addressing.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BAction {
    Set(BKey, String),
    Toggle(BKey),
    Resize(BKey, i32),
    /// Open a dropdown and close all others (menubar hover-to-switch).
    Open(BKey),
    /// Open a file from the given path (file browser → import).
    OpenFile(String),
    /// Viewport orbit: absolute mouse position while MMB-dragging (px).
    Orbit {
        x: i32,
        y: i32,
    },
    /// Viewport zoom: scroll-wheel delta (sign = direction).
    ZoomDelta(i32),
}

/// Color shades for universal Cards.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum CardShade {
    #[default]
    Panel,
    App,
    Activity,
    Header,
    Viewport,
    Accent,
}

/// Pure declarative view tree produced by state and rendered by GPUI.
#[derive(Clone, Debug)]
pub enum BView {
    // Universal Card & Composable Primitives
    Card {
        shade: CardShade,
        header: Option<String>,
        icon: Option<String>,
        badge: Option<String>,
        tool: Option<Box<BView>>,
        collapsible: bool,
        collapsed: bool,
        collapse_action: Option<BAction>,
        body: Option<Box<BView>>,
        footer: Option<String>,
        footer_action: Option<Box<BView>>,
        width: Option<f32>,
        height: Option<f32>,
    },
    List {
        gap: Option<f32>,
        items: Vec<BView>,
    },
    Item {
        label: String,
        icon: Option<String>,
        badge: Option<String>,
        active: bool,
        action: Option<BAction>,
    },

    // Standard Primitives
    Label {
        text: String,
        icon: Option<String>,
    },
    PropRow {
        label: String,
        value: String,
    },
    Separator {
        vertical: bool,
        action: Option<BAction>,
    },
    Empty,

    // Layout Containers
    Row {
        gap: Option<f32>,
        children: Vec<BView>,
    },
    Column {
        gap: Option<f32>,
        children: Vec<BView>,
        center: bool,
    },
    Panel {
        role: BPanelRole,
        title: String,
        children: Vec<BView>,
        width: Option<f32>,
        height: Option<f32>,
    },

    // Menus & Tabs
    Dropdown {
        label: String,
        icon: Option<String>,
        icon_only: bool,
        align_right: bool,
        open: bool,
        children: Vec<BView>,
        action: Option<BAction>,
        hover_action: Option<BAction>,
    },
    HTabBar {
        children: Vec<BView>,
    },
    VTabBar {
        width: Option<f32>,
        children: Vec<BView>,
    },
    Tab {
        label: String,
        icon: Option<String>,
        active: bool,
        icon_only: bool,
        action: Option<BAction>,
    },
    IconList {
        children: Vec<BView>,
    },
    IconButton {
        icon: String,
        label: Option<String>,
        tooltip: Option<String>,
        action: Option<BAction>,
        active: bool,
    },
    Section {
        label: String,
        icon: Option<String>,
        expanded: bool,
        children: Vec<BView>,
        action: Option<BAction>,
    },
    Scroll {
        children: Vec<BView>,
    },
    Spatial {
        mesh: Option<std::sync::Arc<crate::bui::state::WireframeMesh>>,
        filename: String,
        objects: usize,
        vertices: usize,
        camera: (f32, f32, f32),
    },
    Switcher {
        host: crate::store::hosts::BHostType,
        open: bool,
        preview_host: Option<crate::store::hosts::BHostType>,
        action: BAction,
    },
}

impl BView {
    // ── Universal Card Fluent Constructors ──

    pub fn card() -> Self {
        Self::Card {
            shade: CardShade::Panel,
            header: None,
            icon: None,
            badge: None,
            tool: None,
            collapsible: false,
            collapsed: false,
            collapse_action: None,
            body: None,
            footer: None,
            footer_action: None,
            width: None,
            height: None,
        }
    }

    pub fn with_shade(mut self, s: CardShade) -> Self {
        if let Self::Card { shade, .. } = &mut self {
            *shade = s;
        }
        self
    }

    pub fn with_header(mut self, title: impl Into<String>) -> Self {
        if let Self::Card { header, .. } = &mut self {
            *header = Some(title.into());
        }
        self
    }

    pub fn with_badge(mut self, b: impl Into<String>) -> Self {
        match &mut self {
            Self::Card { badge, .. } => *badge = Some(b.into()),
            Self::Item { badge, .. } => *badge = Some(b.into()),
            _ => {}
        }
        self
    }

    pub fn with_tool(mut self, t: BView) -> Self {
        if let Self::Card { tool, .. } = &mut self {
            *tool = Some(Box::new(t));
        }
        self
    }

    pub fn with_collapsible(mut self, c: bool) -> Self {
        if let Self::Card { collapsible, .. } = &mut self {
            *collapsible = c;
        }
        self
    }

    pub fn with_collapsed(mut self, c: bool) -> Self {
        if let Self::Card { collapsed, .. } = &mut self {
            *collapsed = c;
        }
        self
    }

    pub fn with_collapse_action(mut self, a: BAction) -> Self {
        if let Self::Card {
            collapse_action, ..
        } = &mut self
        {
            *collapse_action = Some(a);
        }
        self
    }

    pub fn with_body(mut self, b: BView) -> Self {
        if let Self::Card { body, .. } = &mut self {
            *body = Some(Box::new(b));
        }
        self
    }

    pub fn with_footer(mut self, f: impl Into<String>) -> Self {
        if let Self::Card { footer, .. } = &mut self {
            *footer = Some(f.into());
        }
        self
    }

    pub fn with_footer_action(mut self, fa: BView) -> Self {
        if let Self::Card { footer_action, .. } = &mut self {
            *footer_action = Some(Box::new(fa));
        }
        self
    }

    pub fn with_width(mut self, w: Option<f32>) -> Self {
        match &mut self {
            Self::Card { width, .. } => *width = w,
            Self::Panel { width, .. } => *width = w,
            _ => {}
        }
        self
    }

    pub fn with_height(mut self, h: Option<f32>) -> Self {
        match &mut self {
            Self::Card { height, .. } => *height = h,
            Self::Panel { height, .. } => *height = h,
            _ => {}
        }
        self
    }

    // ── List & Item Fluent Constructors ──

    pub fn list() -> Self {
        Self::List {
            gap: None,
            items: Vec::new(),
        }
    }

    pub fn with_gap(mut self, g: Option<f32>) -> Self {
        match &mut self {
            Self::Row { gap, .. } => *gap = g,
            Self::Column { gap, .. } => *gap = g,
            Self::List { gap, .. } => *gap = g,
            _ => {}
        }
        self
    }

    pub fn with_item(mut self, item: BView) -> Self {
        if let Self::List { items, .. } = &mut self {
            items.push(item);
        }
        self
    }

    pub fn with_items(mut self, new_items: Vec<BView>) -> Self {
        if let Self::List { items, .. } = &mut self {
            items.extend(new_items);
        }
        self
    }

    pub fn with_child(mut self, child: BView) -> Self {
        match &mut self {
            Self::Row { children, .. } => children.push(child),
            Self::Column { children, .. } => children.push(child),
            Self::List { items, .. } => items.push(child),
            _ => {}
        }
        self
    }

    pub fn item(label: impl Into<String>) -> Self {
        Self::Item {
            label: label.into(),
            icon: None,
            badge: None,
            active: false,
            action: None,
        }
    }

    pub fn tab(label: impl Into<String>) -> Self {
        Self::Tab {
            label: label.into(),
            icon: None,
            active: false,
            icon_only: false,
            action: None,
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        let i = icon.into();
        match &mut self {
            Self::Card { icon, .. } => *icon = Some(i),
            Self::Item { icon, .. } => *icon = Some(i),
            Self::IconButton { icon, .. } => *icon = i,
            Self::Tab { icon, .. } => *icon = Some(i),
            Self::Label { icon, .. } => *icon = Some(i),
            Self::Section { icon, .. } => *icon = Some(i),
            _ => {}
        }
        self
    }

    pub fn with_label(mut self, l: impl Into<String>) -> Self {
        let text = l.into();
        match &mut self {
            Self::IconButton { label, .. } => *label = Some(text),
            Self::Item { label, .. } => *label = text,
            Self::Tab { label, .. } => *label = text,
            Self::Label { text: t, .. } => *t = text,
            _ => {}
        }
        self
    }

    pub fn with_tooltip(mut self, tip: impl Into<String>) -> Self {
        if let Self::IconButton { tooltip, .. } = &mut self {
            *tooltip = Some(tip.into());
        }
        self
    }

    pub fn with_icon_only(mut self, only: bool) -> Self {
        match &mut self {
            Self::Dropdown { icon_only, .. } => *icon_only = only,
            Self::Tab { icon_only, .. } => *icon_only = only,
            _ => {}
        }
        self
    }

    pub fn with_active(mut self, a: bool) -> Self {
        match &mut self {
            Self::IconButton { active, .. } => *active = a,
            Self::Tab { active, .. } => *active = a,
            Self::Item { active, .. } => *active = a,
            _ => {}
        }
        self
    }

    pub fn with_action(mut self, a: BAction) -> Self {
        match &mut self {
            Self::IconButton { action, .. } => *action = Some(a),
            Self::Tab { action, .. } => *action = Some(a),
            Self::Item { action, .. } => *action = Some(a),
            Self::Separator { action, .. } => *action = Some(a),
            _ => {}
        }
        self
    }

    // ── Standard Constructors ──

    pub fn empty() -> Self {
        Self::Empty
    }

    pub fn button() -> Self {
        Self::IconButton {
            icon: String::new(),
            label: None,
            tooltip: None,
            action: None,
            active: false,
        }
    }

    pub fn switcher(
        host: crate::store::hosts::BHostType,
        open: bool,
        preview_host: Option<crate::store::hosts::BHostType>,
        action: BAction,
    ) -> Self {
        Self::Switcher {
            host,
            open,
            preview_host,
            action,
        }
    }

    pub fn spatial(
        mesh: Option<std::sync::Arc<crate::bui::state::WireframeMesh>>,
        objects: usize,
        vertices: usize,
        filename: impl Into<String>,
    ) -> Self {
        // Blender default view: yaw 45°, pitch 30°, zoom 1.0.
        Self::Spatial {
            mesh,
            filename: filename.into(),
            objects,
            vertices,
            camera: (45.0, 30.0, 1.0),
        }
    }

    pub fn with_camera(mut self, yaw: f32, pitch: f32, zoom: f32) -> Self {
        if let Self::Spatial { camera, .. } = &mut self {
            *camera = (yaw, pitch, zoom);
        }
        self
    }

    pub fn label(text: impl Into<String>) -> Self {
        Self::Label {
            text: text.into(),
            icon: None,
        }
    }

    pub fn label_icon(text: impl Into<String>, icon: impl Into<String>) -> Self {
        Self::Label {
            text: text.into(),
            icon: Some(icon.into()),
        }
    }

    pub fn prop_row(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self::PropRow {
            label: label.into(),
            value: value.into(),
        }
    }

    pub fn row(gap: Option<f32>, children: Vec<BView>) -> Self {
        Self::Row { gap, children }
    }

    pub fn column(gap: Option<f32>, children: Vec<BView>) -> Self {
        Self::Column {
            gap,
            children,
            center: false,
        }
    }

    /// Column that centers its children on the cross axis (e.g. icon rails).
    pub fn column_center(gap: Option<f32>, children: Vec<BView>) -> Self {
        Self::Column {
            gap,
            children,
            center: true,
        }
    }

    pub fn separator() -> Self {
        Self::Separator {
            vertical: false,
            action: None,
        }
    }

    pub fn vseparator() -> Self {
        Self::Separator {
            vertical: true,
            action: None,
        }
    }

    pub fn panel(role: BPanelRole, title: impl Into<String>, children: Vec<BView>) -> Self {
        Self::Panel {
            role,
            title: title.into(),
            children,
            width: None,
            height: None,
        }
    }

    pub fn panel_sized(
        role: BPanelRole,
        title: impl Into<String>,
        children: Vec<BView>,
        width: Option<f32>,
        height: Option<f32>,
    ) -> Self {
        Self::Panel {
            role,
            title: title.into(),
            children,
            width,
            height,
        }
    }

    pub fn dropdown(label: impl Into<String>, icon: Option<String>, children: Vec<BView>) -> Self {
        Self::Dropdown {
            label: label.into(),
            icon,
            icon_only: false,
            align_right: false,
            open: false,
            children,
            action: None,
            hover_action: None,
        }
    }

    pub fn dropdown_action(
        label: impl Into<String>,
        icon: Option<String>,
        children: Vec<BView>,
        open: bool,
        action: BAction,
    ) -> Self {
        Self::Dropdown {
            label: label.into(),
            icon,
            icon_only: false,
            align_right: false,
            open,
            children,
            action: Some(action),
            hover_action: None,
        }
    }

    pub fn with_hover_action(mut self, ha: BAction) -> Self {
        if let Self::Dropdown { hover_action, .. } = &mut self {
            *hover_action = Some(ha);
        }
        self
    }

    pub fn with_align_right(mut self) -> Self {
        if let Self::Dropdown { align_right, .. } = &mut self {
            *align_right = true;
        }
        self
    }

    pub fn htab_bar(children: Vec<BView>) -> Self {
        Self::HTabBar { children }
    }

    pub fn vtab_bar(children: Vec<BView>) -> Self {
        Self::VTabBar {
            width: None,
            children,
        }
    }

    pub fn vtab_bar_sized(width: Option<f32>, children: Vec<BView>) -> Self {
        Self::VTabBar { width, children }
    }

    pub fn icon_list(children: Vec<BView>) -> Self {
        Self::IconList { children }
    }

    pub fn section(
        label: impl Into<String>,
        icon: Option<String>,
        expanded: bool,
        children: Vec<BView>,
    ) -> Self {
        Self::Section {
            label: label.into(),
            icon,
            expanded,
            children,
            action: None,
        }
    }

    pub fn section_action(
        label: impl Into<String>,
        icon: Option<String>,
        expanded: bool,
        children: Vec<BView>,
        action: BAction,
    ) -> Self {
        Self::Section {
            label: label.into(),
            icon,
            expanded,
            children,
            action: Some(action),
        }
    }

    pub fn scroll(children: Vec<BView>) -> Self {
        Self::Scroll { children }
    }
}
