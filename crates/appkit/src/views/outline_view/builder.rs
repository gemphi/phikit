//! Builder for RCP-style Scene Hierarchy / Outline View UI

use crate::{NSView, NSRect, NSColor, NSButton, NSTextField, NSAutoresizingMask, NSBezelStyle, NSImagePosition, NSApplication, NSLayoutConstants};
use crate::scroll_view::NSScrollView;
use crate::event::{NSEvent, NSEventType, NSDraggingInfo, NSDragOperation};
use crate::theme;
use crate::svg_icons::{self, IconId};
use crate::view::NSViewDelegate;
use std::sync::{Arc, Mutex};

/// A single node representation in the hierarchy
#[derive(Debug, Clone)]
pub struct HierarchyNode {
    pub id: u64,
    pub name: String,
    pub depth: usize,
    pub is_selected: bool,
    pub is_root: bool,
    pub is_collapsed: bool,
}

/// Builds the RCP-style hierarchy view
pub fn build_rcp_hierarchy(
    content: &mut NSView,
    container_w: f32,
    min_content_h: f32,
    nodes: &[HierarchyNode],
    on_select: Arc<dyn Fn(u64) + Send + Sync>,
    on_toggle_collapse: Arc<dyn Fn(u64) + Send + Sync>,
    on_file_dropped: Option<Arc<dyn Fn(std::path::PathBuf) + Send + Sync>>,
) {
    content.remove_all_subviews();

    if nodes.is_empty() {
        let mut empty = NSTextField::label("No entities. Use Insert menu.");
        empty.control_mut().view_mut().set_name("HierarchyEmptyLabel");
        empty.control_mut().view_mut().set_frame(NSRect::new(
            8.0,
            8.0,
            (container_w - 16.0).max(80.0),
            20.0,
        ));
        empty.set_text_color(theme::color(theme::CLR_TEXT_TERTIARY));
        content.add_subview(empty.view_clone());
        return;
    }

    let item_height = NSLayoutConstants::TREE_ITEM_HEIGHT;
    let computed_h = (nodes.len() as f32 * item_height + NSLayoutConstants::UI_SPACING).max(min_content_h);
    content.set_frame(NSRect::sized(container_w, computed_h));

    // Content drag and drop delegate
    let content_delegate = HierarchyContentDelegate {
        on_file_dropped: on_file_dropped.clone(),
    };
    content.set_delegate(content_delegate);

    let mut y = NSLayoutConstants::TREE_ROW_PADDING_Y;

    for node in nodes {
        let indent = node.depth as f32 * NSLayoutConstants::TREE_ICON_SIZE + NSLayoutConstants::UI_SPACING;

        // Row container view
        let mut row = NSView::new(NSRect::new(0.0, y, container_w, item_height));
        row.set_name(&format!("Row_{}", node.id));
        row.set_autoresizing_mask(NSAutoresizingMask::WIDTH_SIZABLE);

        let _ = node.is_selected.then(|| {
            let sel_bg = theme::color(theme::CLR_BG_SELECTED);
            row.set_background_color(Some(sel_bg));
        });

        let mut disc_x = indent;

        let _ = (node.is_root || node.depth == 1).then(|| {
            let tria_icon = match node.is_collapsed {
                true  => IconId::TriaRight,
                false => IconId::TriaDown,
            };

            let mut disc_btn = NSButton::new(NSRect::new(
                disc_x,
                NSLayoutConstants::TREE_ROW_PADDING_Y,
                NSLayoutConstants::TREE_DISCLOSURE_SIZE,
                NSLayoutConstants::TREE_DISCLOSURE_SIZE,
            ));
            match svg_icons::get_sized(tria_icon, NSLayoutConstants::ICON_BUTTON_INLINE, NSLayoutConstants::ICON_BUTTON_INLINE) {
                Some(icon) => { disc_btn.set_image(Some(icon)); }
                None => { disc_btn.set_title(match node.is_collapsed {
                    true  => "Ã¢â€“Â¶",
                    false => "Ã¢â€“Â¼",
                }); }
            }
            disc_btn.set_bezel_style(NSBezelStyle::Inline);

            let node_id = node.id;
            let on_toggle = on_toggle_collapse.clone();
            disc_btn.set_action(move || {
                on_toggle(node_id);
            });

            row.add_subview(disc_btn.view_clone());
            disc_x += NSLayoutConstants::TREE_DISCLOSURE_SIZE;
        });

        // Map node name to appropriate IconId for professional HIG feel
        let name_lower = node.name.to_lowercase();
        let icon_id = match name_lower {
            n if n.contains("camera") => IconId::Camera,
            n if n.contains("audio") || n.contains("sound") || n.contains("speaker") => IconId::Audio,
            n if n.contains("light") || n.contains("sun") || n.contains("directional") || n.contains("spot") => IconId::Light,
            n if n.contains("sphere") => IconId::Sphere,
            n if n.contains("cube") || n.contains("box") => IconId::Cube,
            n if n.contains("cylinder") => IconId::Cylinder,
            n if n.contains("cone") => IconId::Cone,
            n if n.contains("torus") => IconId::Torus,
            n if n.contains("plane") => IconId::Plane,
            _ if node.is_root => IconId::Scene,
            _ => IconId::Object,
        };

        // Entity type icon placement
        let mut icon_btn = NSButton::new(NSRect::new(
            disc_x + 2.0,
            NSLayoutConstants::TREE_ICON_PADDING_Y,
            NSLayoutConstants::TREE_ICON_SIZE,
            NSLayoutConstants::TREE_ICON_SIZE,
        ));
        icon_btn.set_bezel_style(NSBezelStyle::Inline);
        match svg_icons::get_sized(icon_id, NSLayoutConstants::ICON_BUTTON_INLINE, NSLayoutConstants::ICON_BUTTON_INLINE) {
            Some(icon) => { icon_btn.set_image(Some(icon)); }
            None => { icon_btn.set_title("Ã¢Å“Â¦"); }
        }
        row.add_subview(icon_btn.view_clone());

        // Name label
        let label_x = disc_x + NSLayoutConstants::ICON_BUTTON_SIZE;
        let mut label = NSTextField::label(&node.name);
        label.control_mut().view_mut().set_frame(NSRect::new(
            label_x,
            NSLayoutConstants::TREE_ROW_PADDING_Y,
            container_w - label_x - NSLayoutConstants::UI_SPACING,
            NSLayoutConstants::TREE_LABEL_HEIGHT,
        ));
        label.set_text_color(match node.is_selected {
            true  => theme::color(theme::CLR_TEXT_SELECTED),
            false => theme::color(theme::CLR_TEXT_LABEL),
        });
        label.set_font_size(12.0);
        label.control_mut().view_mut().set_name(&format!("EntityLabel_{}", node.id));
        label.control_mut().view_mut().set_autoresizing_mask(NSAutoresizingMask::WIDTH_SIZABLE);
        row.add_subview(label.view().clone());

        // Click and context-menu delegate for the row
        let delegate = EntityRowDelegate {
            entity_id: node.id,
            on_select: on_select.clone(),
        };
        row.set_delegate(delegate);

        content.add_subview(row);
        y += item_height;

        if y > computed_h {
            break;
        }
    }
}

struct HierarchyContentDelegate {
    on_file_dropped: Option<Arc<dyn Fn(std::path::PathBuf) + Send + Sync>>,
}

impl NSViewDelegate for HierarchyContentDelegate {
    fn dragging_entered(&mut self, _view: &mut NSView, _info: &NSDraggingInfo) -> NSDragOperation {
        NSDragOperation::Copy
    }

    fn dragging_updated(&mut self, _view: &mut NSView, _info: &NSDraggingInfo) -> NSDragOperation {
        NSDragOperation::Copy
    }

    fn dragging_exited(&mut self, _view: &mut NSView, _info: &NSDraggingInfo) {}

    fn perform_drag_operation(&mut self, view: &mut NSView, info: &NSDraggingInfo) -> bool {
        let payload = &info.dragging_pasteboard_string;
        if payload.starts_with("asset:") {
            let asset_name = &payload[6..];
            if let Some(cb) = &self.on_file_dropped {
                cb(std::path::PathBuf::from(asset_name));
            }
            view.set_needs_display();
            return true;
        }
        false
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

struct EntityRowDelegate {
    entity_id: u64,
    on_select: Arc<dyn Fn(u64) + Send + Sync>,
}

impl NSViewDelegate for EntityRowDelegate {
    fn mouse_down(&mut self, _view: &mut NSView, event: &NSEvent) {
        if event.event_type() == NSEventType::LeftMouseDown {
            (self.on_select)(self.entity_id);
        }
    }

    fn right_mouse_down(&mut self, _view: &mut NSView, event: &NSEvent) {
        (self.on_select)(self.entity_id);

        let mut menu = crate::menu::Menu::new("Context");
        menu.add("Duplicate", move || {
            // Placeholder: dupe logic can be routed via standard events/state updates
        });
        menu.add("Delete", move || {
            // Placeholder: delete logic
        });

        if let Ok(mut app) = NSApplication::shared().lock() {
            let menu_items = menu.items.clone();
            let x = event.location_in_window().x;
            let y = event.location_in_window().y;
            app.pending_actions.push(Box::new(move |root_view: &mut NSView| {
                let _ = (menu_items, x, y, root_view);
            }));
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Assembly helper for the Hierarchy panel tabs, scroll bars, and toolbars
pub fn build_rcp_hierarchy_panel(
    view: &mut NSView,
    rect: NSRect,
    tab_bar: &mut NSView,
    scroll: &mut NSScrollView,
    footer: &mut NSView,
) {
    let tabs_h = NSLayoutConstants::TAB_BAR_HEIGHT;
    tab_bar.set_frame(NSRect::new(rect.width(), tabs_h));
    tab_bar.set_background_color(Some(theme::color(theme::CLR_BG_TAB_BAR)));
    tab_bar.set_autoresizing_mask(NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::MAX_Y_MARGIN);

    let mut scene_btn = NSButton::new(NSRect::new(4.0, 2.0, 54.0, 22.0)).with_title("Scene");
    scene_btn.set_accent_color(Some(theme::color(theme::CLR_BG_TAB_ACTIVE)));
    scene_btn.set_bezel_style(NSBezelStyle::Rounded);
    tab_bar.add_subview(scene_btn.view_clone());
    view.add_subview(tab_bar.clone());

    let footer_h = NSLayoutConstants::FIELD_ROW_HEIGHT;
    scroll.view_mut().set_frame(NSRect::new(0.0, tabs_h, rect.width(), rect.height() - tabs_h - footer_h));
    scroll.view_mut().set_autoresizing_mask(NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE);
    view.add_subview(scroll.view_clone());

    footer.set_frame(NSRect::new(0.0, rect.height() - footer_h, rect.width(), footer_h));
    footer.set_background_color(Some(theme::color(theme::CLR_BG_HEADER)));
    footer.set_autoresizing_mask(NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::MIN_Y_MARGIN);

    let mut btn_plus = NSButton::new(NSRect::new(6.0, 4.0, 24.0, 24.0))
        .with_image(svg_icons::get(IconId::Add));
    btn_plus.set_bezel_style(NSBezelStyle::Rounded);
    footer.add_subview(btn_plus.view_clone());

    let mut btn_gear = NSButton::new(NSRect::new(34.0, 4.0, 24.0, 24.0))
        .with_image(svg_icons::get(IconId::Settings));
    btn_gear.set_bezel_style(NSBezelStyle::Inline);
    footer.add_subview(btn_gear.view_clone());

    let mut search_field = NSTextField::new(NSRect::new(64.0, 6.0, rect.width() - 74.0, 20.0));
    search_field.set_placeholder("Filter");
    search_field.view_mut().set_name("HierarchySearch");
    search_field.view_mut().set_autoresizing_mask(NSAutoresizingMask::WIDTH_SIZABLE);
    footer.add_subview(search_field.view_clone());

    view.add_subview(footer.clone());
}

