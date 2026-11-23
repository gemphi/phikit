//! Builder for RCP-style 3D Viewport UI Chrome

use crate::{NSView, NSRect, NSColor, NSButton, NSStackView, NSBezelStyle, NSAutoresizingMask, NSTextField, NSTextAlignment};
use crate::gpu::NS3DViewportGPU;
use crate::svg_icons::{self, IconId};
use crate::theme::{self, keys};
use crate::NSLayoutConstants;

/// Builds high-fidelity viewport controls and overlays
pub fn build_rcp_viewport(
    view: &mut NSView,
    rect: NSRect,
    viewport: &mut NS3DViewportGPU,
) {
    let panel_radius = theme::f32_or(keys::RADIUS_PANEL, 12.0);
    
    viewport.view_mut().set_hidden(false);
    // Push viewport down by the radius so the rounded top corners of the panel
    // don't show the gray GPU background peeking through
    viewport.view_mut().set_frame(NSRect::new(0.0, panel_radius, rect.width(), rect.height() - panel_radius));
    viewport.view_mut().set_autoresizing_mask(NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE);
    view.add_subview(viewport.view().clone());

    // Viewport chrome sizing — derived from NSLayoutConstants constants
    let toolbar_btn_size = NSLayoutConstants::ICON_BUTTON_SIZE + 4.0;
    let toolbar_icon_size = NSLayoutConstants::ICON_BUTTON_SIZE - 4.0;
    let ui_spacing = NSLayoutConstants::UI_SPACING / 2.0;
    let nav_bar_h = NSLayoutConstants::MENU_BAR_HEIGHT;
    let mut toolbar_left = NSStackView::horizontal();
    toolbar_left.view_mut().set_name("ViewportToolbarLeft");
    toolbar_left.view_mut().set_frame(NSRect::new(
        10.0,
        nav_bar_h + 6.0,
        (toolbar_btn_size + ui_spacing) * 8.0,
        32.0,
    ));
    toolbar_left.view_mut().set_autoresizing_mask(NSAutoresizingMask::MAX_X_MARGIN | NSAutoresizingMask::MAX_Y_MARGIN);
    toolbar_left.set_spacing(ui_spacing);

    let left_tools = vec![
        (IconId::Select, "Select"),
        (IconId::Move, "Move"),
        (IconId::Rotate, "Rotate"),
        (IconId::Scale, "Scale"),
        (IconId::ViewPan, "Pan"),
        (IconId::ViewZoom, "Zoom"),
        (IconId::ViewCamera, "Reset Camera"),
        (IconId::World, "Environment"),
    ];

    for (icon_id, label) in left_tools {
        let mut btn = NSButton::new(NSRect::sized(toolbar_btn_size, toolbar_btn_size));
        if let Some(icon) = svg_icons::get_sized(icon_id.clone(), toolbar_icon_size, toolbar_icon_size) {
            btn.set_image(Some(icon));
        }
        btn.set_bezel_style(NSBezelStyle::Inline);
        btn.view_mut().set_name(label);
        
        let label_str = label.to_string();
        btn.set_action(move || {
            println!("[Viewport] Tool clicked: {}", label_str);
        });
        
        toolbar_left.add_arranged_subview(btn.view_clone());
    }
    view.add_subview(toolbar_left.view_clone());

    // Top-Right Toolbar (display options)
    let right_toolbar_w = (toolbar_btn_size + ui_spacing) * 4.0;
    let mut toolbar_right = NSStackView::horizontal();
    toolbar_right.view_mut().set_name("ViewportToolbarRight");
    toolbar_right.view_mut().set_frame(NSRect::new(
        rect.width() - right_toolbar_w - 10.0,
        nav_bar_h + 6.0,
        right_toolbar_w,
        32.0,
    ));
    toolbar_right.view_mut().set_autoresizing_mask(NSAutoresizingMask::MIN_X_MARGIN | NSAutoresizingMask::MAX_Y_MARGIN);
    toolbar_right.set_spacing(ui_spacing);

    let right_tools = vec![
        (IconId::Grid, "Grid"),
        (IconId::SnapOn, "Snap"),
        (IconId::Overlay, "Overlays"),
        (IconId::ShadingSolid, "Shading"),
    ];

    for (icon_id, label) in right_tools {
        let mut btn = NSButton::new(NSRect::sized(toolbar_btn_size, toolbar_btn_size));
        if let Some(icon) = svg_icons::get_sized(icon_id.clone(), toolbar_icon_size, toolbar_icon_size) {
            btn.set_image(Some(icon));
        }
        btn.set_bezel_style(NSBezelStyle::Inline);
        btn.view_mut().set_name(label);
        
        let label_str = label.to_string();
        btn.set_action(move || {
            println!("[Viewport] Display option clicked: {}", label_str);
        });
        
        toolbar_right.add_arranged_subview(btn.view_clone());
    }
    view.add_subview(toolbar_right.view_clone());

    // Navigation Bar (Top)
    let nav_bar_h = 28.0;
    let panel_radius = crate::theme::f32_or(crate::theme::keys::RADIUS_PANEL, 12.0);
    let mut nav_bar = NSView::new(NSRect::new(rect.width(), nav_bar_h));
    nav_bar.set_name("ViewportNavBar");
    nav_bar.set_background_color(Some(crate::theme::color(crate::theme::keys::CLR_BG_WINDOW)));
    nav_bar.set_corner_radius(panel_radius);
    nav_bar.set_clips_to_bounds(true);
    nav_bar.set_autoresizing_mask(NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::MAX_Y_MARGIN);

    let nav_tools = vec![
        (IconId::ViewCamera, "Perspective"),
        (IconId::Select, "Selection"),
        (IconId::ViewPan, "Pan"),
        (IconId::ViewZoom, "Dolly"),
        (IconId::ZoomAll, "Focus"),
    ];

    let mut nx = 8.0;
    for (icon_id, tooltip) in nav_tools {
        let mut btn = NSButton::new(NSRect::new(nx, 4.0, 24.0, 24.0));
        if let Some(icon) = svg_icons::get_sized(icon_id.clone(), 14.0, 14.0) {
            btn.set_image(Some(icon));
        }
        btn.set_bezel_style(NSBezelStyle::Inline);
        btn.view_mut().set_name(tooltip);
        
        let tooltip_str = tooltip.to_string();
        btn.set_action(move || {
            println!("[Viewport] Nav clicked: {}", tooltip_str);
        });
        
        nav_bar.add_subview(btn.view_clone());
        nx += 24.0;
    }

    let mut mode_label = NSTextField::label("Perspective");
    mode_label.control_mut().view_mut().set_frame(NSRect::new(
        rect.width() - 120.0,
        4.0,
        110.0,
        18.0,
    ));
    mode_label.set_text_color(NSColor::secondary_label());
    mode_label.set_font_size(11.0);
    mode_label.control_mut().view_mut().set_autoresizing_mask(NSAutoresizingMask::MIN_X_MARGIN);
    nav_bar.add_subview(mode_label.view_clone());

    view.add_subview(nav_bar);
}
