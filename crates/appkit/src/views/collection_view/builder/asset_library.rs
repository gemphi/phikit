//! Asset Library Assembly Logic
//!
//! this file implements the asset library panel for Reality Composer Pro.
//! it provides a searchable, categorized grid of 3D primitive shapes and effects.
//! mirrors Apple's modular pattern for content libraries and object pickers.

use super::data_source::GenericGridItemDelegate;
use crate::svg_icons::{self, IconId};
use crate::{
    NSAutoresizingMask, NSBezelStyle, NSButton, NSColor, NSLayoutAttribute, NSRect, NSSearchField,
    NSStackView, NSTextField, NSView,
};

/// Assembly helper for Asset Library panel.
/// creates a categorized browser for project assets and primitives.
pub fn build_rcp_asset_library(
    view: &mut NSView,
    rect: NSRect,
    search_field: &mut NSSearchField,
    collection_view: &mut crate::collection_view::NSCollectionView,
) {
    let search_height = 28.0;
    search_field
        .view_mut()
        .set_frame(NSRect::new(8.0, 8.0, rect.width() - 16.0, search_height));
    search_field.set_placeholder_string("Search Assets...");
    search_field.view_mut().set_autoresizing_mask(
        NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::MAX_Y_MARGIN,
    );
    view.add_subview(search_field.view_clone());

    let tabs_height = 32.0;
    let tabs_y = 8.0 + search_height + 8.0;
    let mut tabs_stack = NSStackView::horizontal();
    tabs_stack.view_mut().set_name("CategoryTabsStack");
    tabs_stack
        .view_mut()
        .set_frame(NSRect::new(8.0, tabs_y, rect.width() - 16.0, tabs_height));
    tabs_stack.set_spacing(4.0);
    tabs_stack.set_alignment(NSLayoutAttribute::Fill);
    tabs_stack.view_mut().set_autoresizing_mask(
        NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::MAX_Y_MARGIN,
    );

    let categories = ["Shapes", "Materials", "Physics", "Audio"];
    for cat in categories {
        let mut btn = NSButton::new(NSRect::zero()).with_title(cat);
        tabs_stack.add_arranged_subview(btn.view_clone());
    }
    view.add_subview(tabs_stack.view_clone());

    let grid_y = tabs_y + tabs_height + 8.0;
    let grid_rect = NSRect::new(
        8.0,
        grid_y,
        rect.width() - 16.0,
        (rect.height() - grid_y).max(0.0),
    );
    collection_view.view_mut().set_frame(grid_rect);
    collection_view.view_mut().set_autoresizing_mask(
        NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
    );
    view.add_subview(collection_view.view().clone());

    let shapes = vec![
        ("Capsule", IconId::Sphere),
        ("Cone", IconId::Cone),
        ("Cube", IconId::Cube),
        ("Cylinder", IconId::Cylinder),
        ("Sphere", IconId::Sphere),
        ("Plane", IconId::Plane),
        ("Torus", IconId::Torus),
    ];

    let mut icon_x = 16.0;
    let mut icon_y = grid_y + 16.0;
    for (name, icon_id) in shapes {
        let mut icon_btn = NSButton::new(NSRect::new(icon_x, icon_y, 64.0, 64.0));
        icon_btn.set_bezel_style(NSBezelStyle::Rounded);
        icon_btn
            .view_mut()
            .set_background_color(Some(NSColor::new(0.22, 0.22, 0.24, 1.0)));

        if let Some(icon_img) = svg_icons::get_sized(icon_id, 24.0, 24.0) {
            let mut icon_view = NSView::new(NSRect::new(20.0, 24.0, 24.0, 24.0));
            icon_view.set_delegate(GenericGridItemDelegate { image: icon_img });
            icon_btn.view_mut().add_subview(icon_view);
        }

        let mut label = NSTextField::label(name);
        label
            .control_mut()
            .view_mut()
            .set_frame(NSRect::new(4.0, 4.0, 56.0, 16.0));
        label.set_font_size(9.0);
        label.set_text_alignment(crate::NSTextAlignment::Center);
        label.set_text_color(NSColor::secondary_label());
        icon_btn.view_mut().add_subview(label.view_clone());

        view.add_subview(icon_btn.view_clone());

        icon_x += 80.0;
        if icon_x > rect.width() - 80.0 {
            icon_x = 16.0;
            icon_y += 80.0;
        }
    }
}
