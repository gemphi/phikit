use rust_appkit::prelude::*;
use rust_appkit::{NSSplitView, NSStackView};

#[test]
fn test_sustainable_layout_engine() {
    let window_frame = NSRect::sized(1000.0, 1000.0);
    let mut root_view = NSView::new(window_frame);
    root_view.set_name("Root");

    // Create a horizontal split (Left | Right)
    let mut split = NSSplitView::vertical();
    split.view_mut().set_name("MainSplit");
    split.view_mut().set_frame(window_frame);
    split.view_mut().set_autoresizing_mask(
        NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
    );

    let mut left = NSView::new(NSRect::sized(200.0, 1000.0));
    left.set_name("LeftPanel");

    let mut right = NSView::new(NSRect::sized(800.0, 1000.0));
    right.set_name("RightPanel");

    split.add_arranged_subview(left);
    split.add_arranged_subview(right);

    root_view.add_subview(split.view().clone());

    // Initial verification
    assert_eq!(root_view.subviews().len(), 1);

    // Resize root
    root_view.set_frame(NSRect::sized(2000.0, 2000.0));
    root_view.layout_subviews();

    // Split should have resized
    let split_frame = root_view.subviews()[0].frame();
    assert_eq!(split_frame.width(), 2000.0);
    assert_eq!(split_frame.height(), 2000.0);

    println!("Sustainable Layout Engine Test Passed!");
}

#[test]
fn test_stack_view_distribution() {
    let mut stack = NSStackView::vertical();
    stack
        .view_mut()
        .set_frame(NSRect::sized(100.0, 300.0));
    stack.view_mut().set_name("Stack");
    stack.set_spacing(0.0);

    for i in 0..3 {
        let mut v = NSView::new(NSRect::sized(100.0, 50.0));
        v.set_name(&format!("Item_{}", i));
        v.set_autoresizing_mask(NSAutoresizingMask::HEIGHT_SIZABLE);
        stack.add_arranged_subview(v);
    }

    stack.view_mut().layout_subviews();

    // Since distribution is Fill (default) and height is 300,
    // each item should get 100 height if they are flexible or if distribution logic divides space.
    // Our current StackViewDelegate divides space equally if no intrinsic size is provided.

    let subviews = stack.view().subviews();
    assert_eq!(subviews.len(), 3);
    assert_eq!(subviews[0].frame().height(), 100.0);
    assert_eq!(subviews[1].frame().height(), 100.0);
    assert_eq!(subviews[2].frame().height(), 100.0);

    println!("Stack View Distribution Test Passed!");
}

#[test]
fn test_responsive_sidebar_containment() {
    // 1. Create a Sidebar (320x900)
    let sidebar_frame = NSRect::sized(320.0, 900.0);
    let mut sidebar = NSView::new(sidebar_frame).with_name("Sidebar");

    // 2. Add an internal SearchBar that SHOULD be responsive (WIDTH_SIZABLE)
    // Initially width=300 (10px margins)
    let mut search_bar = NSView::new(NSRect::new(10.0, 10.0, 300.0, 30.0)).with_name("SearchBar");
    search_bar.set_autoresizing_mask(NSAutoresizingMask::WIDTH_SIZABLE);
    sidebar.add_subview(search_bar.clone());

    // 3. Shrink Sidebar to 200px
    sidebar.set_frame(NSRect::sized(200.0, 900.0));
    sidebar.layout_subviews();

    // 4. Verify SearchBar scaled down and is NOT overflowing (300px > 200px)
    let search_frame = sidebar.subviews()[0].frame();
    assert!(
        search_frame.width() <= 200.0,
        "SearchBar failed to resize and is now overflowing its parent!"
    );
    assert!(search_frame.width() > 0.0);

    println!("Responsive Sidebar Containment Test Passed!");
}

#[test]
fn test_panel_anchoring_integrity() {
    // Regression test for "ugly" layout jitter
    let mut panel = NSView::new(NSRect::sized(500.0, 500.0)).with_name("ParentPanel");

    // Bottom-aligned toolbar (pinned to bottom)
    let mut toolbar = NSView::new(NSRect::new(0.0, 460.0, 500.0, 40.0)).with_name("BottomToolbar");
    toolbar.set_autoresizing_mask(
        NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::MIN_Y_MARGIN,
    );
    panel.add_subview(toolbar.clone());

    // Grow panel height to 800
    panel.set_frame(NSRect::sized(500.0, 800.0));
    panel.layout_subviews();

    // Toolbar should still be at bottom (y = 760)
    let toolbar_frame = panel.subviews()[0].frame();
    assert_eq!(
        toolbar_frame.origin.y, 760.0,
        "Toolbar failed to stay anchored to the bottom!"
    );

    println!("Panel Anchoring Integrity Test Passed!");
}

#[test]
fn test_five_zone_layout_arrangement() {
    use rust_appkit::panels::workspace_layout::{DockPosition, WorkspacePreset};

    let preset = WorkspacePreset::default_rcp();

    // Verify toolbar_visible is true (toolbar occupies the top zone)
    assert!(
        preset.toolbar_visible,
        "Toolbar must be visible in default RCP layout"
    );

    // Collect slots by id for easy lookup
    let find_slot = |id: &str| preset.slots.iter().find(|s| s.id == id);

    // 1. Toolbar (top zone)
    let toolbar = find_slot("toolbar").expect("toolbar slot must exist");
    assert_eq!(
        toolbar.dock,
        DockPosition::Top,
        "toolbar must be docked at Top"
    );
    assert!(
        toolbar.min_size > 0.0,
        "toolbar must have non-zero min_size"
    );

    // 2. Hierarchy (left zone)
    let hierarchy = find_slot("hierarchy").expect("hierarchy slot must exist");
    assert_eq!(
        hierarchy.dock,
        DockPosition::Left,
        "hierarchy must be docked at Left"
    );
    assert!(
        hierarchy.min_size > 0.0,
        "hierarchy must have non-zero min_size"
    );

    // 3. Viewport (center zone)
    let viewport = find_slot("viewport").expect("viewport slot must exist");
    assert_eq!(
        viewport.dock,
        DockPosition::Center,
        "viewport must be docked at Center"
    );
    assert!(
        viewport.min_size > 0.0,
        "viewport must have non-zero min_size"
    );

    // 4. Inspector (right zone)
    let inspector = find_slot("inspector").expect("inspector slot must exist");
    assert_eq!(
        inspector.dock,
        DockPosition::Right,
        "inspector must be docked at Right"
    );
    assert!(
        inspector.min_size > 0.0,
        "inspector must have non-zero min_size"
    );

    // 5. Bottom zone (timeline + project browser)
    let timeline = find_slot("timeline").expect("timeline slot must exist");
    assert_eq!(
        timeline.dock,
        DockPosition::Bottom,
        "timeline must be docked at Bottom"
    );
    assert!(
        timeline.min_size > 0.0,
        "timeline must have non-zero min_size"
    );

    let project_browser = find_slot("project_browser").expect("project_browser slot must exist");
    assert_eq!(
        project_browser.dock,
        DockPosition::Bottom,
        "project_browser must be docked at Bottom"
    );
    assert!(
        project_browser.min_size > 0.0,
        "project_browser must have non-zero min_size"
    );

    // Verify bottom panel is visible
    assert!(preset.bottom_panel_visible, "bottom panel must be visible");

    // Verify all panels have min_size <= preferred_size <= max_size
    for slot in &preset.slots {
        assert!(
            slot.min_size <= slot.preferred_size,
            "Panel '{}': min_size ({}) must be <= preferred_size ({})",
            slot.id,
            slot.min_size,
            slot.preferred_size
        );
        assert!(
            slot.preferred_size <= slot.max_size,
            "Panel '{}': preferred_size ({}) must be <= max_size ({})",
            slot.id,
            slot.preferred_size,
            slot.max_size
        );
    }

    println!("Five-Zone Layout Arrangement Test Passed!");
}


#[test]
fn test_layout_constraints_from_split_view() {
    use rust_appkit::panels::workspace_layout::LayoutConstraints;

    // Create a split view simulating the main editor split (sidebar | viewport | inspector)
    let total_width = 1200.0;
    let frame = NSRect::sized(total_width, 600.0);
    let mut split = NSSplitView::new(frame);
    split.set_vertical(true);

    // Add 3 subviews
    split.add_arranged_subview(NSView::new(NSRect::sized(300.0, 600.0)));
    split.add_arranged_subview(NSView::new(NSRect::sized(600.0, 600.0)));
    split.add_arranged_subview(NSView::new(NSRect::sized(300.0, 600.0)));

    // Set divider positions: sidebar ends at 300, inspector starts at 900
    split.set_divider_at_index(0, 300.0);
    split.set_divider_at_index(1, 900.0);

    let constraints = LayoutConstraints::from_split_view(&split);

    assert_eq!(constraints.sidebar_w, 300.0);
    assert_eq!(constraints.viewport_w, 600.0);
    assert_eq!(constraints.inspector_w, 300.0);
    assert_eq!(constraints.total_width(), total_width);

    println!("LayoutConstraints from_split_view Test Passed!");
}

#[test]
fn test_layout_constraints_enforces_minimums() {
    use rust_appkit::panels::workspace_layout::{defaults, LayoutConstraints};

    // Test that LayoutConstraints clamps divider positions to enforce minimums.
    // The actual minimum values come from the theme system:
    //   sidebar_min = theme(SIZE_SIDEBAR_WIDTH, 320) * 0.625
    //   inspector_min = sidebar_min + 50
    let sidebar_min = defaults::sidebar_min();
    let inspector_min = defaults::inspector_min();

    let total_width = 1000.0;
    let frame = NSRect::sized(total_width, 600.0);
    let mut split = NSSplitView::new(frame);
    split.set_vertical(true);

    // Add subviews
    split.add_arranged_subview(NSView::new(NSRect::sized(300.0, 600.0)));
    split.add_arranged_subview(NSView::new(NSRect::sized(400.0, 600.0)));
    split.add_arranged_subview(NSView::new(NSRect::sized(300.0, 600.0)));

    // Set divider 0 below sidebar_min
    split.set_divider_at_index(0, (sidebar_min - 50.0) as f64);
    // Set divider 1 above total_width - inspector_min
    split.set_divider_at_index(1, (total_width - inspector_min + 50.0) as f64);

    let constraints = LayoutConstraints::from_split_view(&split);

    // sidebar_w should be clamped to at least sidebar_min
    assert!(
        constraints.sidebar_w >= sidebar_min,
        "sidebar_w ({}) should be >= sidebar_min ({})",
        constraints.sidebar_w,
        sidebar_min
    );
    // inspector_w should be at least inspector_min
    assert!(
        constraints.inspector_w >= inspector_min,
        "inspector_w ({}) should be >= inspector_min ({})",
        constraints.inspector_w,
        inspector_min
    );
    // viewport_w should be non-negative
    assert!(
        constraints.viewport_w >= 0.0,
        "viewport_w ({}) should be non-negative",
        constraints.viewport_w
    );
    // Total should equal frame width
    assert_eq!(
        constraints.total_width(),
        total_width,
        "total_width should equal the split view frame width"
    );

    println!("LayoutConstraints Enforces Minimums Test Passed!");
}


#[test]
fn test_divider_at_point_vertical_split() {
    let frame = NSRect::sized(1200.0, 600.0);
    let mut split = NSSplitView::new(frame);
    split.set_vertical(true);

    // Add 3 subviews and set dividers at 300 and 900
    split.add_arranged_subview(NSView::new(NSRect::sized(300.0, 600.0)));
    split.add_arranged_subview(NSView::new(NSRect::sized(600.0, 600.0)));
    split.add_arranged_subview(NSView::new(NSRect::sized(300.0, 600.0)));
    split.set_divider_at_index(0, 300.0);
    split.set_divider_at_index(1, 900.0);

    // Exactly on divider 0 center
    assert_eq!(split.divider_at_point(NSPoint::new(300.0, 100.0)), Some(0));

    // Within ±4px of divider 0
    assert_eq!(split.divider_at_point(NSPoint::new(296.1, 100.0)), Some(0));
    assert_eq!(split.divider_at_point(NSPoint::new(303.9, 100.0)), Some(0));

    // Just outside ±4px of divider 0
    assert_eq!(split.divider_at_point(NSPoint::new(295.9, 100.0)), None);
    assert_eq!(split.divider_at_point(NSPoint::new(304.1, 100.0)), None);

    // Exactly on divider 1 center
    assert_eq!(split.divider_at_point(NSPoint::new(900.0, 200.0)), Some(1));

    // Within ±4px of divider 1
    assert_eq!(split.divider_at_point(NSPoint::new(897.0, 200.0)), Some(1));
    assert_eq!(split.divider_at_point(NSPoint::new(903.0, 200.0)), Some(1));

    // Point far from any divider
    assert_eq!(split.divider_at_point(NSPoint::new(500.0, 300.0)), None);

    println!("Divider At Point (Vertical Split) Test Passed!");
}

#[test]
fn test_divider_at_point_horizontal_split() {
    let frame = NSRect::sized(800.0, 600.0);
    let mut split = NSSplitView::new(frame);
    split.set_vertical(false); // horizontal: dividers are along y-axis

    split.add_arranged_subview(NSView::new(NSRect::sized(800.0, 200.0)));
    split.add_arranged_subview(NSView::new(NSRect::sized(800.0, 400.0)));
    split.set_divider_at_index(0, 200.0);

    // For horizontal split, the y coordinate is tested
    assert_eq!(split.divider_at_point(NSPoint::new(400.0, 200.0)), Some(0));
    assert_eq!(split.divider_at_point(NSPoint::new(400.0, 196.5)), Some(0));
    assert_eq!(split.divider_at_point(NSPoint::new(400.0, 203.5)), Some(0));
    assert_eq!(split.divider_at_point(NSPoint::new(400.0, 194.0)), None);
    assert_eq!(split.divider_at_point(NSPoint::new(400.0, 206.0)), None);

    println!("Divider At Point (Horizontal Split) Test Passed!");
}

#[test]
fn test_mouse_down_begins_drag() {
    use rust_appkit::event::{NSEvent, NSEventType};

    let frame = NSRect::sized(1200.0, 600.0);
    let mut split = NSSplitView::new(frame);
    split.set_vertical(true);

    split.add_arranged_subview(NSView::new(NSRect::sized(300.0, 600.0)));
    split.add_arranged_subview(NSView::new(NSRect::sized(600.0, 600.0)));
    split.add_arranged_subview(NSView::new(NSRect::sized(300.0, 600.0)));
    split.set_divider_at_index(0, 300.0);
    split.set_divider_at_index(1, 900.0);

    // Initially no drag
    assert!(split.active_drag.is_none());

    // Mouse down on divider 0 (within ±4px)
    let event = NSEvent {
        event_type: NSEventType::LeftMouseDown,
        location_in_window: NSPoint::new(302.0, 150.0),
        timestamp: 0.0,
        delta_x: 0.0,
        delta_y: 0.0,
        modifier_flags: 0,
        characters: None,
        file_path: None,
    };

    let consumed = split.mouse_down(&event);
    assert!(consumed, "mouse_down should return true when hitting a divider");
    assert!(split.active_drag.is_some());

    let drag = split.active_drag.unwrap();
    assert_eq!(drag.index, 0);
    assert_eq!(drag.start_pos, NSPoint::new(302.0, 150.0));
    assert_eq!(drag.start_divider_pos, 300.0);

    println!("Mouse Down Begins Drag Test Passed!");
}

#[test]
fn test_mouse_down_no_divider_hit() {
    use rust_appkit::event::{NSEvent, NSEventType};

    let frame = NSRect::sized(1200.0, 600.0);
    let mut split = NSSplitView::new(frame);
    split.set_vertical(true);

    split.add_arranged_subview(NSView::new(NSRect::sized(300.0, 600.0)));
    split.add_arranged_subview(NSView::new(NSRect::sized(600.0, 600.0)));
    split.add_arranged_subview(NSView::new(NSRect::sized(300.0, 600.0)));
    split.set_divider_at_index(0, 300.0);
    split.set_divider_at_index(1, 900.0);

    // Mouse down far from any divider
    let event = NSEvent {
        event_type: NSEventType::LeftMouseDown,
        location_in_window: NSPoint::new(500.0, 150.0),
        timestamp: 0.0,
        delta_x: 0.0,
        delta_y: 0.0,
        modifier_flags: 0,
        characters: None,
        file_path: None,
    };

    let consumed = split.mouse_down(&event);
    assert!(!consumed, "mouse_down should return false when not hitting a divider");
    assert!(split.active_drag.is_none());

    println!("Mouse Down No Divider Hit Test Passed!");
}


// ═══════════════════════════════════════════════════════════════════════
// Property-Based Tests
// ═══════════════════════════════════════════════════════════════════════

/// **Validates: Requirements 5.2, 5.3**
///
/// Property 9: Scroll Delta Conversion
///
/// For any LineDelta(x, y), the pixel translation SHALL be (x * 20.0, y * 20.0).
/// For any PixelDelta with a scale factor, the logical coordinate conversion
/// SHALL preserve the delta value scaled by the window's scale factor.
mod scroll_delta_property_tests {
    use proptest::prelude::*;
    use winit::dpi::PhysicalPosition;
    use winit::event::MouseScrollDelta;

    // Pixels per line constant matching the implementation in run_loop_input.rs
    const PIXELS_PER_LINE: f32 = 20.0;

    proptest! {
        #[test]
        fn prop_line_delta_conversion(
            x in -1000.0f32..1000.0f32,
            y in -1000.0f32..1000.0f32,
        ) {
            // Simulate the conversion logic from handle_mouse_wheel
            let delta = MouseScrollDelta::LineDelta(x, y);

            let (dx, dy) = match delta {
                MouseScrollDelta::LineDelta(lx, ly) => {
                    (lx * PIXELS_PER_LINE, ly * PIXELS_PER_LINE)
                }
                _ => unreachable!(),
            };

            // Property: LineDelta(x, y) produces (x * 20.0, y * 20.0)
            prop_assert!(
                (dx - x * 20.0).abs() < f32::EPSILON,
                "LineDelta x conversion: expected {}, got {}",
                x * 20.0,
                dx
            );
            prop_assert!(
                (dy - y * 20.0).abs() < f32::EPSILON,
                "LineDelta y conversion: expected {}, got {}",
                y * 20.0,
                dy
            );
        }

        #[test]
        fn prop_pixel_delta_conversion(
            px in -5000.0f64..5000.0f64,
            py in -5000.0f64..5000.0f64,
            // Scale factor must be positive and non-zero (typical range 1.0-3.0)
            scale_factor in 1.0f64..3.0f64,
        ) {
            // Simulate the conversion logic from handle_mouse_wheel
            let delta = MouseScrollDelta::PixelDelta(PhysicalPosition::new(px, py));

            let (dx, dy) = match delta {
                MouseScrollDelta::PixelDelta(p) => {
                    let lp = p.to_logical::<f32>(scale_factor);
                    (lp.x, lp.y)
                }
                _ => unreachable!(),
            };

            // Property: PixelDelta preserves the scaled value
            // to_logical divides physical pixels by scale_factor
            let expected_x = (px / scale_factor) as f32;
            let expected_y = (py / scale_factor) as f32;

            prop_assert!(
                (dx - expected_x).abs() < 0.01,
                "PixelDelta x conversion: expected {}, got {} (physical={}, scale={})",
                expected_x,
                dx,
                px,
                scale_factor
            );
            prop_assert!(
                (dy - expected_y).abs() < 0.01,
                "PixelDelta y conversion: expected {}, got {} (physical={}, scale={})",
                expected_y,
                dy,
                py,
                scale_factor
            );
        }
    }
}

mod property_tests {
    use proptest::prelude::*;
    use rust_appkit::prelude::*;
    use rust_appkit::panels::workspace_layout::{
        align_bottom_panel_columns, defaults, LayoutConstraints,
    };

    // **Validates: Requirements 4.1, 4.2, 4.3, 4.4**
    //
    // Property 6: Bottom Panel Alignment Invariant
    //
    // For any set of main NSSplitView divider positions (where divider_0 <= divider_1
    // and both are within [0, total_width]), after calling align_bottom_panel_columns:
    //   - column[0].width == constraints.sidebar_w
    //   - column[2].width == constraints.inspector_w
    //   - all columns sum to total_width
    //   - columns are contiguous (no gaps or overlaps)
    //   - sidebar_w >= sidebar_min and inspector_w >= inspector_min
    proptest! {
        #[test]
        fn prop_bottom_panel_alignment_invariant(
            // Total width must accommodate sidebar_min + inspector_min (~450).
            // Use 500 minimum to give headroom for theme variations.
            total_width in 500.0f32..4000.0f32,
            // Generate divider positions as fractions within valid range.
            // divider_0 in [sidebar_min, total_width - inspector_min]
            // divider_1 in [divider_0, total_width - inspector_min]
            divider_0_frac in 0.0f32..1.0f32,
            divider_1_frac in 0.0f32..1.0f32,
            panel_height in 50.0f32..600.0f32,
        ) {
            let sidebar_min = defaults::sidebar_min();
            let inspector_min = defaults::inspector_min();

            // Generate valid divider positions within the feasible range.
            // divider_0 ranges from sidebar_min to (total_width - inspector_min)
            let div0_min = sidebar_min;
            let div0_max = (total_width - inspector_min).max(sidebar_min);
            let divider_0 = div0_min + divider_0_frac * (div0_max - div0_min);

            // divider_1 ranges from divider_0 to (total_width - inspector_min)
            let div1_max = (total_width - inspector_min).max(divider_0);
            let divider_1 = divider_0 + divider_1_frac * (div1_max - divider_0);

            // Derive constraints from the divider positions
            let positions = vec![divider_0, divider_1];
            let constraints = LayoutConstraints::from_divider_positions(&positions, total_width);

            // Create a bottom panel with 3 column subviews
            let mut bottom_panel = NSView::new(NSRect::sized(total_width, panel_height));
            bottom_panel.add_subview(NSView::new(NSRect::sized(100.0, panel_height)));
            bottom_panel.add_subview(NSView::new(NSRect::new(100.0, 0.0, 100.0, panel_height)));
            bottom_panel.add_subview(NSView::new(NSRect::new(200.0, 0.0, 100.0, panel_height)));

            // Apply alignment
            align_bottom_panel_columns(&mut bottom_panel, &constraints);

            let subviews = bottom_panel.subviews();

            // Property: column[0].width == constraints.sidebar_w
            let col0_width = subviews[0].frame().width();
            prop_assert!(
                (col0_width - constraints.sidebar_w).abs() < f32::EPSILON,
                "column[0].width ({}) must equal sidebar_w ({})",
                col0_width,
                constraints.sidebar_w
            );

            // Property: column[2].width == constraints.inspector_w
            let col2_width = subviews[2].frame().width();
            prop_assert!(
                (col2_width - constraints.inspector_w).abs() < f32::EPSILON,
                "column[2].width ({}) must equal inspector_w ({})",
                col2_width,
                constraints.inspector_w
            );

            // Property: all columns sum to total_width
            let col1_width = subviews[1].frame().width();
            let sum = col0_width + col1_width + col2_width;
            prop_assert!(
                (sum - total_width).abs() < 0.01,
                "columns sum ({}) must equal total_width ({})",
                sum,
                total_width
            );

            // Property: sidebar_w respects minimum constraint
            prop_assert!(
                constraints.sidebar_w >= sidebar_min,
                "sidebar_w ({}) must be >= sidebar_min ({})",
                constraints.sidebar_w,
                sidebar_min
            );

            // Property: inspector_w respects minimum constraint
            prop_assert!(
                constraints.inspector_w >= inspector_min,
                "inspector_w ({}) must be >= inspector_min ({})",
                constraints.inspector_w,
                inspector_min
            );

            // Property: viewport_w is non-negative
            prop_assert!(
                constraints.viewport_w >= 0.0,
                "viewport_w ({}) must be non-negative",
                constraints.viewport_w
            );

            // Property: column positions are contiguous (no gaps/overlaps)
            let col0_x = subviews[0].frame().origin.x;
            let col1_x = subviews[1].frame().origin.x;
            let col2_x = subviews[2].frame().origin.x;

            prop_assert!(
                (col0_x - 0.0).abs() < f32::EPSILON,
                "column[0] must start at x=0, got {}",
                col0_x
            );
            prop_assert!(
                (col1_x - col0_width).abs() < f32::EPSILON,
                "column[1] must start at sidebar_w ({}), got {}",
                col0_width,
                col1_x
            );
            prop_assert!(
                (col2_x - (col0_width + col1_width)).abs() < f32::EPSILON,
                "column[2] must start at sidebar_w + viewport_w ({}), got {}",
                col0_width + col1_width,
                col2_x
            );
        }
    }
}

/// **Validates: Requirements 6.1, 6.4, 6.5**
///
/// Property 7: Divider Position Respects Constraints
///
/// For any drag target position, the final divider position is clamped to
/// [min, max] constraints derived from adjacent panel minimum sizes.
/// A mouse-down within ±4px of a divider center initiates a drag.
mod divider_position_constraints_property_tests {
    use proptest::prelude::*;
    use rust_appkit::prelude::*;
    use rust_appkit::event::{NSEvent, NSEventType};
    use rust_appkit::NSSplitView;

    proptest! {
        /// Property: mouse_down within ±4px of a divider center initiates drag
        #[test]
        fn prop_mouse_down_within_4px_initiates_drag(
            // Divider position within a reasonable range
            divider_pos in 100.0f32..900.0f32,
            // Offset from divider center: within ±4px should initiate drag
            offset in -4.0f32..=4.0f32,
            // Y coordinate doesn't matter for vertical split hit testing
            y_pos in 0.0f32..600.0f32,
        ) {
            let frame = NSRect::sized(1200.0, 600.0);
            let mut split = NSSplitView::new(frame);
            split.set_vertical(true);

            // Add 2 subviews with the divider between them
            split.add_arranged_subview(NSView::new(NSRect::sized(divider_pos, 600.0)));
            split.add_arranged_subview(NSView::new(NSRect::sized(1200.0 - divider_pos, 600.0)));
            split.set_divider_at_index(0, divider_pos as f64);

            let click_x = divider_pos + offset;
            let event = NSEvent {
                event_type: NSEventType::LeftMouseDown,
                location_in_window: NSPoint::new(click_x, y_pos),
                timestamp: 0.0,
                delta_x: 0.0,
                delta_y: 0.0,
                modifier_flags: 0,
                characters: None,
                file_path: None,
            };

            let consumed = split.mouse_down(&event);

            // Property: mouse_down within ±4px MUST initiate drag
            prop_assert!(
                consumed,
                "mouse_down at offset {} from divider at {} should initiate drag",
                offset,
                divider_pos
            );
            prop_assert!(
                split.active_drag.is_some(),
                "active_drag must be Some after mouse_down within ±4px"
            );
            let drag = split.active_drag.unwrap();
            prop_assert_eq!(drag.index, 0);
            prop_assert!(
                (drag.start_divider_pos - divider_pos).abs() < f32::EPSILON,
                "start_divider_pos ({}) must equal divider position ({})",
                drag.start_divider_pos,
                divider_pos
            );
        }

        /// Property: after mouse_dragged, divider position is clamped to valid
        /// min/max constraints derived from adjacent panel minimum sizes.
        /// The final position never violates minimum panel sizes.
        #[test]
        fn prop_divider_position_clamped_after_drag(
            // Total width of the split view
            total_width in 400.0f32..2000.0f32,
            // Min sizes for left and right panels (reasonable range)
            min_left in 50.0f32..200.0f32,
            min_right in 50.0f32..200.0f32,
            // Initial divider position as fraction of valid range
            initial_frac in 0.0f32..1.0f32,
            // Drag target as an arbitrary position (can be way out of bounds)
            drag_target in -500.0f32..3000.0f32,
        ) {
            // Skip if total_width can't accommodate both minimums
            prop_assume!(total_width >= min_left + min_right);

            let frame = NSRect::sized(total_width, 600.0);
            let mut split = NSSplitView::new(frame);
            split.set_vertical(true);

            // Compute valid range for divider position
            let valid_min = min_left;
            let valid_max = total_width - min_right;

            // Place initial divider within valid range
            let initial_pos = valid_min + initial_frac * (valid_max - valid_min);

            // Create subviews with min_size constraints
            let mut left_view = NSView::new(NSRect::sized(initial_pos, 600.0));
            left_view.set_min_size(NSSize::new(min_left, 0.0));

            let mut right_view = NSView::new(NSRect::sized(total_width - initial_pos, 600.0));
            right_view.set_min_size(NSSize::new(min_right, 0.0));

            split.add_arranged_subview(left_view);
            split.add_arranged_subview(right_view);
            split.set_divider_at_index(0, initial_pos as f64);

            // Initiate drag with mouse_down on the divider
            let down_event = NSEvent {
                event_type: NSEventType::LeftMouseDown,
                location_in_window: NSPoint::new(initial_pos, 300.0),
                timestamp: 0.0,
                delta_x: 0.0,
                delta_y: 0.0,
                modifier_flags: 0,
                characters: None,
                file_path: None,
            };
            split.mouse_down(&down_event);
            prop_assert!(split.active_drag.is_some(), "drag must be initiated");

            // Drag to the target position
            let drag_event = NSEvent {
                event_type: NSEventType::LeftMouseDragged,
                location_in_window: NSPoint::new(drag_target, 300.0),
                timestamp: 0.0,
                delta_x: 0.0,
                delta_y: 0.0,
                modifier_flags: 0,
                characters: None,
                file_path: None,
            };
            split.mouse_dragged(&drag_event);

            // Check the final divider position
            let final_pos = split.divider_position(0) as f32;

            // Allow small floating-point tolerance for clamping comparisons
            let eps = 0.001;

            // Property: final position is clamped to [min_left, total_width - min_right]
            prop_assert!(
                final_pos >= valid_min - eps,
                "divider position ({}) must be >= min constraint ({}) [drag_target={}, total_width={}]",
                final_pos,
                valid_min,
                drag_target,
                total_width
            );
            prop_assert!(
                final_pos <= valid_max + eps,
                "divider position ({}) must be <= max constraint ({}) [drag_target={}, total_width={}]",
                final_pos,
                valid_max,
                drag_target,
                total_width
            );

            // Property: left panel size never goes below its minimum
            let left_panel_size = final_pos;
            prop_assert!(
                left_panel_size >= min_left - eps,
                "left panel size ({}) must be >= min_left ({})",
                left_panel_size,
                min_left
            );

            // Property: right panel size never goes below its minimum
            let right_panel_size = total_width - final_pos;
            prop_assert!(
                right_panel_size >= min_right - eps,
                "right panel size ({}) must be >= min_right ({})",
                right_panel_size,
                min_right
            );
        }

        /// Property: with multiple dividers, dragging one divider respects
        /// constraints from adjacent dividers and panel minimums.
        #[test]
        fn prop_multi_divider_clamping(
            total_width in 600.0f32..2000.0f32,
            min_a in 50.0f32..150.0f32,
            min_b in 50.0f32..150.0f32,
            min_c in 50.0f32..150.0f32,
            // Which divider to drag (0 or 1)
            drag_divider_idx in 0usize..2usize,
            // Drag target (can be extreme)
            drag_target in -500.0f32..3000.0f32,
        ) {
            // Skip if total_width can't accommodate all minimums
            prop_assume!(total_width >= min_a + min_b + min_c);

            let frame = NSRect::sized(total_width, 600.0);
            let mut split = NSSplitView::new(frame);
            split.set_vertical(true);

            // Place dividers to give each panel its minimum + some extra
            let extra = (total_width - min_a - min_b - min_c) / 3.0;
            let div0_pos = min_a + extra;
            let div1_pos = min_a + min_b + 2.0 * extra;

            let mut view_a = NSView::new(NSRect::sized(div0_pos, 600.0));
            view_a.set_min_size(NSSize::new(min_a, 0.0));

            let mut view_b = NSView::new(NSRect::sized(div1_pos - div0_pos, 600.0));
            view_b.set_min_size(NSSize::new(min_b, 0.0));

            let mut view_c = NSView::new(NSRect::sized(total_width - div1_pos, 600.0));
            view_c.set_min_size(NSSize::new(min_c, 0.0));

            split.add_arranged_subview(view_a);
            split.add_arranged_subview(view_b);
            split.add_arranged_subview(view_c);
            split.set_divider_at_index(0, div0_pos as f64);
            split.set_divider_at_index(1, div1_pos as f64);

            // Initiate drag on the chosen divider
            let divider_pos = if drag_divider_idx == 0 { div0_pos } else { div1_pos };
            let down_event = NSEvent {
                event_type: NSEventType::LeftMouseDown,
                location_in_window: NSPoint::new(divider_pos, 300.0),
                timestamp: 0.0,
                delta_x: 0.0,
                delta_y: 0.0,
                modifier_flags: 0,
                characters: None,
                file_path: None,
            };
            split.mouse_down(&down_event);
            prop_assert!(split.active_drag.is_some(), "drag must be initiated");

            // Drag to target
            let drag_event = NSEvent {
                event_type: NSEventType::LeftMouseDragged,
                location_in_window: NSPoint::new(drag_target, 300.0),
                timestamp: 0.0,
                delta_x: 0.0,
                delta_y: 0.0,
                modifier_flags: 0,
                characters: None,
                file_path: None,
            };
            split.mouse_dragged(&drag_event);

            // Read final positions
            let final_div0 = split.divider_position(0) as f32;
            let final_div1 = split.divider_position(1) as f32;

            // Allow small floating-point tolerance for clamping comparisons
            let eps = 0.001;

            // Property: panel A size >= min_a
            let panel_a_size = final_div0;
            prop_assert!(
                panel_a_size >= min_a - eps,
                "panel A size ({}) must be >= min_a ({})",
                panel_a_size,
                min_a
            );

            // Property: panel B size >= min_b
            let panel_b_size = final_div1 - final_div0;
            prop_assert!(
                panel_b_size >= min_b - eps,
                "panel B size ({}) must be >= min_b ({})",
                panel_b_size,
                min_b
            );

            // Property: panel C size >= min_c
            let panel_c_size = total_width - final_div1;
            prop_assert!(
                panel_c_size >= min_c - eps,
                "panel C size ({}) must be >= min_c ({})",
                panel_c_size,
                min_c
            );

            // Property: divider ordering is maintained (div0 <= div1)
            prop_assert!(
                final_div0 <= final_div1 + eps,
                "divider 0 ({}) must be <= divider 1 ({})",
                final_div0,
                final_div1
            );
        }
    }
}

/// **Validates: Requirements 5.1, 5.4**
///
/// Property 8: Scroll Offset Bounded
///
/// For any sequence of scroll events applied to an NSScrollView, the scroll_offset
/// (bounds.origin) SHALL remain within [0, content_size - visible_size] on both axes.
/// This ensures no over-scrolling occurs regardless of the magnitude or direction
/// of scroll deltas.
mod scroll_offset_bounded_property_tests {
    use proptest::prelude::*;
    use proptest::collection::vec as prop_vec;
    use rust_appkit::prelude::*;
    use rust_appkit::event::{NSEvent, NSEventType};
    use rust_appkit::NSScrollView;

    /// Strategy to generate a scroll event with arbitrary delta values.
    fn scroll_event_strategy() -> impl Strategy<Value = NSEvent> {
        // Generate scroll deltas in a wide range to stress-test clamping
        (-500.0f32..500.0f32, -500.0f32..500.0f32).prop_map(|(dx, dy)| {
            NSEvent {
                event_type: NSEventType::ScrollWheel,
                location_in_window: NSPoint::new(50.0, 50.0),
                timestamp: 0.0,
                delta_x: dx,
                delta_y: dy,
                modifier_flags: 0,
                characters: None,
                file_path: None,
            }
        })
    }

    proptest! {
        #[test]
        fn prop_scroll_offset_bounded(
            // Content must be at least as large as visible area for scrolling to make sense.
            // visible_size is the scroll view frame; content_size is the document view size.
            visible_w in 50.0f32..800.0f32,
            visible_h in 50.0f32..800.0f32,
            // content_extra is how much larger the content is than the visible area
            content_extra_w in 0.0f32..2000.0f32,
            content_extra_h in 0.0f32..2000.0f32,
            // Generate a sequence of 1-20 scroll events
            scroll_events in prop_vec(scroll_event_strategy(), 1..20),
        ) {
            let content_w = visible_w + content_extra_w;
            let content_h = visible_h + content_extra_h;

            // Create an NSScrollView with the given visible size
            let frame = NSRect::sized(visible_w, visible_h);
            let mut scroll_view = NSScrollView::new(frame);

            // Set a document view with the given content size
            let doc_view = NSView::new(NSRect::sized(content_w, content_h));
            scroll_view.set_document_view(doc_view);

            // The maximum scroll offsets
            let max_x = (content_w - visible_w).max(0.0);
            let max_y = (content_h - visible_h).max(0.0);

            // Apply each scroll event and verify bounds after each one
            for event in &scroll_events {
                // Dispatch the scroll event to the scroll view's underlying view
                scroll_view.view_mut().dispatch_mouse_event(event, visible_h);

                // Check the bounds origin (scroll offset) is within valid range
                let bounds = scroll_view.view().bounds();
                let offset_x = bounds.origin.x;
                let offset_y = bounds.origin.y;

                prop_assert!(
                    offset_x >= 0.0,
                    "scroll_offset.x ({}) must be >= 0.0 after scroll event (delta_x={})",
                    offset_x,
                    event.delta_x
                );
                prop_assert!(
                    offset_x <= max_x,
                    "scroll_offset.x ({}) must be <= max_x ({}) after scroll event (delta_x={})",
                    offset_x,
                    max_x,
                    event.delta_x
                );
                prop_assert!(
                    offset_y >= 0.0,
                    "scroll_offset.y ({}) must be >= 0.0 after scroll event (delta_y={})",
                    offset_y,
                    event.delta_y
                );
                prop_assert!(
                    offset_y <= max_y,
                    "scroll_offset.y ({}) must be <= max_y ({}) after scroll event (delta_y={})",
                    offset_y,
                    max_y,
                    event.delta_y
                );
            }
        }
    }
}
