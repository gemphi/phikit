//! Project Browser Assembly Logic
//!
//! this file implements the high-fidelity project browser layout for RCP.
//! it coordinates split views, navigators, and asset grid displays.
//! mirrors Apple's modular pattern for complex data-driven workspace panels.

use super::constants::*;
use crate::svg_icons::{self, IconId};
use crate::theme;
use crate::{
    NSAutoresizingMask, NSBezelStyle, NSButton, NSCollectionView, NSColor, NSLayoutAttribute,
    NSRect, NSScrollView, NSSlider, NSSplitView, NSSplitViewDividerStyle, NSTextField, NSView,
};

/// Assembly helper for Project Browser panel.
/// constructs a modular multi-pane interface with search and navigation tools.
pub fn build_rcp_project_browser(
    view: &mut NSView,
    rect: NSRect,
    collection: &mut NSCollectionView,
    import_btn: &mut NSButton,
    placeholder: &mut NSTextField,
) {
    let mut split = NSSplitView::vertical();
    let toolbar_h = 32.0;
    split.view_mut().set_frame(NSRect::new(
        0.0,
        toolbar_h,
        rect.width(),
        rect.height() - toolbar_h,
    ));
    split.view_mut().set_autoresizing_mask(
        NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
    );
    split.set_divider_style(NSSplitViewDividerStyle::Thin);

    // Toolbar
    let mut toolbar =
        NSView::new(NSRect::sized(rect.width(), toolbar_h)).with_name("ProjectToolbar");
    toolbar.set_background_color(Some(theme::color(theme::CLR_BG_TOOLBAR)));
    toolbar.set_autoresizing_mask(NSAutoresizingMask::WIDTH_SIZABLE);

    import_btn
        .view_mut()
        .set_frame(NSRect::new(PANEL_PADDING, 5.0, 80.0, 22.0));
    toolbar.add_subview(import_btn.view_clone());

    let icon_x = PANEL_PADDING + 80.0 + UI_SPACING;
    let icon_tools = [
        (IconId::Grid, "Grid View"),
        (IconId::MenuPanel, "List View"),
        (IconId::ZoomIn, "Zoom In"),
        (IconId::ZoomOut, "Zoom Out"),
    ];
    let mut tx = icon_x;
    for (icon_id, label) in icon_tools {
        let mut btn = NSButton::new(NSRect::new(tx, 5.0, 20.0, 20.0));
        if let Some(icon) = svg_icons::get_sized(icon_id, 14.0, 14.0) {
            btn.set_image(Some(icon));
        }
        btn.set_bezel_style(NSBezelStyle::Inline);

        let label_str = label.to_string();
        btn.set_action(move || {
            println!("[BottomPanel] Tool clicked: {}", label_str);
            match label_str.as_str() {
                "Grid View" => { /* Logic to switch to grid layout */ }
                "List View" => { /* Logic to switch to list layout */ }
                "Zoom In" => { /* Logic to increase icon size */ }
                "Zoom Out" => { /* Logic to decrease icon size */ }
                _ => {}
            }
        });

        toolbar.add_subview(btn.view_clone());
        tx += 24.0;
    }

    let mut slider = NSSlider::new(NSRect::new(
        rect.width() - 120.0 - 180.0 - MARGIN * 3.0,
        5.0,
        120.0,
        20.0,
    ));
    slider
        .view_mut()
        .set_autoresizing_mask(NSAutoresizingMask::MIN_X_MARGIN);
    toolbar.add_subview(slider.view_clone());

    let mut search = NSTextField::new(NSRect::new(rect.width() - 180.0 - MARGIN, 5.0, 180.0, 20.0));
    search
        .view_mut()
        .set_autoresizing_mask(NSAutoresizingMask::MIN_X_MARGIN);
    search.set_placeholder("Filter");
    toolbar.add_subview(search.view_clone());

    // Navigator (Left)
    let mut navigator = NSView::new(NSRect::sized(NAVIGATOR_WIDTH, rect.height()))
        .with_name("ProjectNavigator");
    navigator.set_autoresizing_mask(NSAutoresizingMask::HEIGHT_SIZABLE);
    navigator.set_background_color(Some(theme::color(theme::CLR_BG_CONTENT)));

    let mut collections_hdr = NSTextField::label("Collections");
    collections_hdr
        .control_mut()
        .view_mut()
        .set_frame(NSRect::new(
            MARGIN,
            10.0,
            NAVIGATOR_WIDTH - MARGIN * 2.0,
            20.0,
        ));
    collections_hdr.set_text_color(NSColor::secondary_label());
    collections_hdr.set_font_size(11.0);
    navigator.add_subview(collections_hdr.view().clone());

    let mut all_row = NSView::new(NSRect::new(0.0, 32.0, NAVIGATOR_WIDTH, 24.0));
    all_row.set_background_color(Some(NSColor::new(0.2, 0.45, 0.8, 0.7)));

    let mut all_icon = NSButton::new(NSRect::new(MARGIN + 4.0, 4.0, 16.0, 16.0));
    if let Some(icon) = svg_icons::get_sized(IconId::Folder, 16.0, 16.0) {
        all_icon.set_image(Some(icon));
    }
    all_icon.set_bezel_style(NSBezelStyle::Inline);
    all_row.add_subview(all_icon.view_clone());

    let mut all_files = NSTextField::label("All Files");
    all_files.control_mut().view_mut().set_frame(NSRect::new(
        MARGIN + 26.0,
        3.0,
        NAVIGATOR_WIDTH - 40.0,
        18.0,
    ));
    all_files.set_text_color(NSColor::white());
    all_row.add_subview(all_files.view_clone());
    navigator.add_subview(all_row);

    split.add_arranged_subview(navigator);

    // File View (Center)
    let center_width = (rect.width() - NAVIGATOR_WIDTH - PROJECT_INSPECTOR_WIDTH).max(100.0);
    let mut file_view_container = NSView::new(NSRect::new(
        NAVIGATOR_WIDTH,
        0.0,
        center_width,
        rect.height(),
    ))
    .with_name("FileViewContainer");
    file_view_container.set_autoresizing_mask(
        NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
    );

    let mut scroll = NSScrollView::new(NSRect::sized(center_width, rect.height()));
    scroll.view_mut().set_name("ProjectAssetsScroll");
    scroll.view_mut().set_autoresizing_mask(
        NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
    );

    collection
        .view_mut()
        .set_frame(NSRect::sized(center_width, rect.height()));
    collection
        .view_mut()
        .set_autoresizing_mask(NSAutoresizingMask::WIDTH_SIZABLE);

    scroll.set_document_view(collection.view().clone());
    file_view_container.add_subview(scroll.view().clone());

    placeholder.control_mut().view_mut().set_frame(NSRect::new(
        center_width / 2.0 - 100.0,
        rect.height() / 2.0,
        300.0,
        24.0,
    ));
    placeholder.control_mut().view_mut().set_autoresizing_mask(
        NSAutoresizingMask::MIN_X_MARGIN
            | NSAutoresizingMask::MAX_X_MARGIN
            | NSAutoresizingMask::MIN_Y_MARGIN
            | NSAutoresizingMask::MAX_Y_MARGIN,
    );
    placeholder.set_text_color(NSColor::secondary_label());
    file_view_container.add_subview(placeholder.view_clone());

    split.add_arranged_subview(file_view_container);

    // Inspector (Right)
    let mut inspector = NSView::new(NSRect::new(
        0.0,
        0.0,
        PROJECT_INSPECTOR_WIDTH,
        rect.height(),
    ))
    .with_name("ProjectInspector");
    inspector.set_autoresizing_mask(
        NSAutoresizingMask::MIN_X_MARGIN | NSAutoresizingMask::HEIGHT_SIZABLE,
    );
    inspector.set_background_color(Some(theme::color(theme::CLR_BG_CONTENT)));

    let mut ins_tabs = crate::NSStackView::horizontal();
    ins_tabs
        .view_mut()
        .set_frame(NSRect::sized(PROJECT_INSPECTOR_WIDTH, 28.0));
    ins_tabs.set_spacing(0.0);
    ins_tabs.set_alignment(NSLayoutAttribute::Fill);

    let mut tab_hier = NSButton::new(NSRect::zero()).with_title("Hierarchy");
    tab_hier.set_bezel_style(NSBezelStyle::TexturedRounded);
    ins_tabs.add_arranged_subview(tab_hier.view_clone());

    let mut tab_info = NSButton::new(NSRect::zero()).with_title("Information");
    tab_info.set_bezel_style(NSBezelStyle::Rounded);
    ins_tabs.add_arranged_subview(tab_info.view_clone());
    inspector.add_subview(ins_tabs.view_clone());

    let mut ins_content = NSView::new(NSRect::new(
        0.0,
        28.0,
        PROJECT_INSPECTOR_WIDTH,
        rect.height() - 28.0,
    ));
    let mut ins_label = NSTextField::label("Select a file to view its\ndetails.");
    ins_label.control_mut().view_mut().set_frame(NSRect::new(
        MARGIN,
        20.0,
        PROJECT_INSPECTOR_WIDTH - MARGIN * 2.0,
        40.0,
    ));
    ins_label.set_text_color(NSColor::secondary_label());
    ins_label.set_font_size(12.0);
    ins_content.add_subview(ins_label.view().clone());
    inspector.add_subview(ins_content);

    split.add_arranged_subview(inspector);

    split.set_divider_at_index(0, NAVIGATOR_WIDTH as f64);
    split.set_divider_at_index(1, (NAVIGATOR_WIDTH + center_width) as f64);

    view.add_subview(toolbar);
    view.add_subview(split.view().clone());
}
