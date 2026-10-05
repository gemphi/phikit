//! Space area shape unit tests and generated composition preservation coverage.
//!
//! Feature: phiber-blender-space-parity
//! Validates: Requirements 2.1, 3.1, 4.1, 5.1, 6.1, 7.1, 8.1, 9.1, 10.1, 11.1, 12.1
//! Feature: phiber-resolved-ui-style-settings, Property 11
//! **Validates: Requirements 11.2**
//!
//! Every space declares the correct number of areas in each zone. Values
//! are ground-truthed against the requirements document.

use phiber::v2::{
    host::BHostType,
    space::{BSpaceItem, BSpaceType},
    state::{BState, key::ui_style as style_key},
    theme::{BPaletteName, resolve_ui_style, ui_style_defaults},
    view::{
        BView, BViewItem,
        draw::space::{BSpaceCompositionShape, composition_shape},
    },
    zone::{BZoneType, visibility},
};
use proptest::prelude::*;
use std::{fs, path::PathBuf};

fn renderer(relative: &str) -> String {
    let path = module_file(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/v2/view/draw")
        .join(relative));
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn count(space: BSpaceType, zone: BZoneType) -> usize {
    space.spec().areas_in(zone).len()
}

fn default_host(space: BSpaceType, zone: BZoneType) -> Option<BHostType> {
    space.spec().areas_in(zone).first().map(|a| a.default_host)
}

#[test]
fn layout_areas() {
    // Requirement 2: Top, Center, Right×2, Bottom×2 — plus the Phiber-native
    // Activity rail, in the Rail zone every space carries (global navigation).
    assert_eq!(count(BSpaceType::Layout, BZoneType::Top), 1);
    assert_eq!(count(BSpaceType::Layout, BZoneType::Rail), 1);
    assert_eq!(count(BSpaceType::Layout, BZoneType::Left), 0);
    assert_eq!(count(BSpaceType::Layout, BZoneType::Center), 1);
    assert_eq!(count(BSpaceType::Layout, BZoneType::Right), 2);
    assert_eq!(count(BSpaceType::Layout, BZoneType::Bottom), 2);
    assert_eq!(
        default_host(BSpaceType::Layout, BZoneType::Top),
        Some(BHostType::Topbar)
    );
    assert_eq!(
        default_host(BSpaceType::Layout, BZoneType::Rail),
        Some(BHostType::Activity)
    );
    for space in BSpaceType::ALL {
        assert_eq!(
            default_host(*space, BZoneType::Rail),
            Some(BHostType::Activity),
            "{space:?} carries the activity rail"
        );
    }
    assert_eq!(
        default_host(BSpaceType::Layout, BZoneType::Center),
        Some(BHostType::View3D)
    );
}

#[test]
fn modeling_areas() {
    // Requirement 3: three zones.
    assert_eq!(count(BSpaceType::Modeling, BZoneType::Top), 1);
    assert_eq!(count(BSpaceType::Modeling, BZoneType::Center), 1);
    assert_eq!(count(BSpaceType::Modeling, BZoneType::Bottom), 1);
    assert_eq!(
        default_host(BSpaceType::Modeling, BZoneType::Center),
        Some(BHostType::View3D)
    );
}

#[test]
fn sculpting_areas() {
    // `startup.blend`: viewport, Outliner over Properties, status bar.
    assert_eq!(count(BSpaceType::Sculpting, BZoneType::Top), 1);
    assert_eq!(count(BSpaceType::Sculpting, BZoneType::Center), 1);
    assert_eq!(count(BSpaceType::Sculpting, BZoneType::Right), 2);
    assert_eq!(count(BSpaceType::Sculpting, BZoneType::Bottom), 1);
}

#[test]
fn uv_editing_areas() {
    // Requirement 5: four zones — Top, Left, Center, Bottom.
    assert_eq!(count(BSpaceType::UvEditing, BZoneType::Top), 1);
    assert_eq!(count(BSpaceType::UvEditing, BZoneType::Left), 1);
    assert_eq!(count(BSpaceType::UvEditing, BZoneType::Center), 1);
    assert_eq!(count(BSpaceType::UvEditing, BZoneType::Bottom), 1);
    assert_eq!(
        default_host(BSpaceType::UvEditing, BZoneType::Left),
        Some(BHostType::Uv)
    );
}

#[test]
fn shading_uses_three_stacked_middle_columns() {
    let spec = BSpaceType::Shading.spec();
    let hosts = |zone| {
        spec.areas_in(zone)
            .iter()
            .map(|area| area.default_host)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        hosts(BZoneType::Left),
        vec![BHostType::File, BHostType::Image]
    );
    assert_eq!(
        hosts(BZoneType::Center),
        vec![BHostType::View3D, BHostType::Shader]
    );
    assert_eq!(
        hosts(BZoneType::Right),
        vec![BHostType::Outliner, BHostType::Buts]
    );
    assert_eq!(hosts(BZoneType::Bottom), vec![BHostType::Statusbar], "the window's status bar");
}

#[test]
fn rendering_is_the_image_editor_with_properties_and_a_timeline() {
    // Blender's Rendering workspace (`startup.blend`, read by
    // `truth/blender_workspaces.py`): the Image Editor on Render Result,
    // a Properties editor down the right, a Timeline strip, no Outliner.
    assert_eq!(count(BSpaceType::Rendering, BZoneType::Top), 1);
    assert_eq!(count(BSpaceType::Rendering, BZoneType::Center), 1);
    assert_eq!(count(BSpaceType::Rendering, BZoneType::Right), 1);
    assert_eq!(count(BSpaceType::Rendering, BZoneType::Left), 0);
    assert_eq!(
        default_host(BSpaceType::Rendering, BZoneType::Center),
        Some(BHostType::Image)
    );
    assert_eq!(default_host(BSpaceType::Rendering, BZoneType::Right), Some(BHostType::Buts));
    assert_eq!(default_host(BSpaceType::Rendering, BZoneType::Bottom), Some(BHostType::Time));
}

#[test]
fn compositing_areas() {
    // `startup.blend`: node editor, Outliner over Properties, a Dope Sheet
    // strip, status bar.
    assert_eq!(count(BSpaceType::Compositing, BZoneType::Top), 1);
    assert_eq!(count(BSpaceType::Compositing, BZoneType::Center), 1);
    assert_eq!(count(BSpaceType::Compositing, BZoneType::Right), 2);
    assert_eq!(count(BSpaceType::Compositing, BZoneType::Bottom), 2);
    assert_eq!(
        default_host(BSpaceType::Compositing, BZoneType::Center),
        Some(BHostType::Compositor)
    );
    assert_eq!(default_host(BSpaceType::Compositing, BZoneType::Bottom), Some(BHostType::Action));
}

#[test]
fn geometry_nodes_areas() {
    // Requirement 11: five zones — Top, Center, Right, Bottom×2.
    assert_eq!(count(BSpaceType::GeometryNodes, BZoneType::Top), 1);
    assert_eq!(count(BSpaceType::GeometryNodes, BZoneType::Center), 1);
    let bottom = count(BSpaceType::GeometryNodes, BZoneType::Bottom);
    assert!(bottom >= 2, "GeometryNodes should have >=2 bottom slots");
}

#[test]
fn scripting_areas() {
    // Blender's `Scripting` workspace in startup.blend: a left column of
    // viewport, Python console and Info log; the Text editor in the centre;
    // Outliner and Properties on the right.
    assert_eq!(count(BSpaceType::Scripting, BZoneType::Top), 1);
    assert_eq!(count(BSpaceType::Scripting, BZoneType::Left), 3);
    assert_eq!(count(BSpaceType::Scripting, BZoneType::Center), 1);
    assert_eq!(default_host(BSpaceType::Scripting, BZoneType::Center), Some(BHostType::Text));
    assert_eq!(count(BSpaceType::Scripting, BZoneType::Right), 2);
    assert_eq!(count(BSpaceType::Scripting, BZoneType::Bottom), 1);
}

/// Post-decomposition shape regression: the hierarchy routers still own the
/// same top/middle/bottom and left/center/right composition, while every flex
/// layer between the workspace and viewport remains explicitly bounded.
///
/// Feature: phiber-resolved-ui-style-settings
/// Validates: Requirements 11.2, 11.3, 13.8
#[test]
fn focused_renderers_preserve_space_shape_viewport_fill_and_bounds() {
    let space = renderer("space/mod.rs");
    let zone = renderer("zone/mod.rs");
    let host = renderer("host/mod.rs");
    let region = renderer("region/mod.rs");
    let viewport = renderer("canvas/view3d/mod.rs");

    for band in [
        "top: Vec<&'a BView>",
        "middle: Vec<&'a BView>",
        "bottom: Vec<&'a BView>",
    ] {
        assert!(space.contains(band), "workspace partition lost `{band}`");
    }
    assert!(space.contains("= composition(list);"));
    // The space's containers are `BSpacePart` decisions: a full-window column
    // root, a row middle band, and a centre column that fills and may shrink
    // — a column, because the bottom band (Timeline, Dope Sheet) hangs under
    // the centre editor while the side zones run to the status bar.
    let space_parts = renderer("space/part.rs");
    assert!(space_parts.contains("Self::Root => BFlex::column()"));
    assert!(space_parts.contains(".full()"));
    assert!(space_parts.contains("Self::Middle => BFlex::row()"));
    assert!(space_parts.contains("BFlex::column().fills().height(BLen::Full).min_height(BLen::Zero).unclamped()"));
    let band = renderer("space/band.rs");
    assert!(band.contains("fn split_bottom(") && band.contains("is_fixed_bar()"), "the status bar leaves the band");

    // A zone lays out along its own axis, gapped by the resize seam, and its
    // area slots may shrink on both axes.
    // The zone lays out along its own kind's axis — now a variant binding
    // (`BZonePart::Frame { kind, .. }`) rather than a struct field.
    assert!(zone.contains("BFlex::along(kind.axis())"));
    assert!(zone.contains(".gap(BSpace::Seam)"));
    assert!(zone.contains("BFlex::column().unclamped().shares(share)"));

    // A host fills its slot at full size; its middle band fills and runs the
    // full width.
    assert!(host.contains(".fills()\n                .min_height(BLen::Zero)\n                .full()"));
    assert!(host.contains("BFlex::row().fills().min_height(BLen::Zero).width(BLen::Full)"));
    assert!(region.contains("let flex = flex.fills().min_height(BLen::Zero).full();"));
    assert!(viewport.contains(".flex_1()"));
    assert!(viewport.contains(".min_h_0()"));
    assert!(viewport.contains(".size_full()"));
}

fn composition_state(visible: [bool; 3], gap: f32) -> BState {
    let mut state = BState::new();
    for (zone, show) in visibility::TOGGLEABLE.into_iter().zip(visible) {
        state.set(
            &visibility::key(zone).expect("toggleable zone has a key"),
            show.to_string(),
        );
    }
    state.set(&style_key::RESIZE_GAP, gap.to_string());
    state
}

fn generated_shape(
    space: BSpaceType,
    state: &BState,
    include_full: bool,
) -> BSpaceCompositionShape {
    let BView::Space { mut list, .. } = BSpaceItem::new(space).build(state) else {
        unreachable!("a space item always builds a space view")
    };
    if include_full
        && !list.iter().any(|view| {
            matches!(
                view,
                BView::Zone {
                    kind: BZoneType::Full,
                    ..
                }
            )
        })
    {
        list.push(BView::Zone {
            kind: BZoneType::Full,
            list: Vec::new(),
        });
    }
    composition_shape(&list)
}

fn expected_shape(
    space: BSpaceType,
    visible: [bool; 3],
    include_full: bool,
) -> BSpaceCompositionShape {
    let mut shape = BSpaceCompositionShape {
        top: vec![],
        middle: vec![],
        bottom: vec![],
        extras: 0,
    };
    // Zones in layout order (`IN_ORDER`): the rail leads the middle band.
    for &zone in BZoneType::IN_ORDER {
        let shown = visibility::TOGGLEABLE
            .iter()
            .position(|candidate| *candidate == zone)
            .is_none_or(|index| visible[index]);
        if !space.spec().areas_in(zone).is_empty() && shown {
            match zone {
                BZoneType::Top => shape.top.push(zone),
                BZoneType::Bottom => shape.bottom.push(zone),
                _ => shape.middle.push(zone),
            }
        }
    }
    if include_full && !shape.middle.contains(&BZoneType::Full) {
        shape.middle.push(BZoneType::Full);
    }
    shape
}

#[test]
fn full_zone_uses_the_middle_composition_band() {
    let shape = composition_shape(&[BView::Zone {
        kind: BZoneType::Full,
        list: Vec::new(),
    }]);
    assert_eq!(shape.middle, vec![BZoneType::Full]);
    assert!(shape.top.is_empty() && shape.bottom.is_empty() && shape.extras == 0);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Property 11: Space composition remains shape-preserving.
    /// Feature: phiber-resolved-ui-style-settings, Property 11
    /// **Validates: Requirements 11.2**
    #[test]
    fn resolved_style_variants_preserve_space_composition_shape(
        space in prop::sample::select(BSpaceType::EVERY.to_vec()),
        visible in any::<[bool; 3]>(),
        include_full in any::<bool>(),
        palette_index in 0usize..BPaletteName::ALL.len(),
        palette_offset in 1usize..BPaletteName::ALL.len(),
        gap in 0.0f32..16.0,
        gap_delta in 0.25f32..16.0,
    ) {
        let state_a = composition_state(visible, gap);
        let state_b = composition_state(visible, gap + gap_delta);
        let palette_a = BPaletteName::ALL[palette_index];
        let palette_b = BPaletteName::ALL[(palette_index + palette_offset) % BPaletteName::ALL.len()];
        let style_a = resolve_ui_style(ui_style_defaults(palette_a), &state_a);
        let style_b = resolve_ui_style(ui_style_defaults(palette_b), &state_b);
        prop_assert_ne!(style_a, style_b);

        prop_assert_eq!(generated_shape(space, &state_a, include_full), expected_shape(space, visible, include_full));
        prop_assert_eq!(generated_shape(space, &state_b, include_full), expected_shape(space, visible, include_full));
    }
}

// ── Task 8.4 — Overlay zero-footprint regression (post-decomposition) ────────
//
// Feature: phiber-resolved-ui-style-settings
// Validates: Requirements 11.2, 11.3, 13.8 (seam overlay does not distort
// space composition after renderer decomposition)
//
// After the renderer was decomposed into BFocusedRenderer roles, every zone
// and space flex container must still carry the sizing props that prevent
// overlay absolute-positioned children from inflating layout.

/// Post-decomposition: zone.rs must use `.min_w_0()` and `.min_h_0()` so that
/// absolute-positioned overlays (BSeam) cannot force the flex container wider
/// or taller than its allocated fraction.
#[test]
fn zone_flex_containers_prevent_overlay_inflation() {
    let zone_src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/v2/view/draw/zone/mod.rs"),
    )
    .expect("zone.rs must exist");

    // `unclamped()` is min-width and min-height zero.
    assert!(
        zone_src.contains("BFlex::column().unclamped()"),
        "zone.rs area slots must be unclamped so absolute overlays do not inflate the flex"
    );
    // The gap between areas is the resize-seam token, which resolves from the
    // resolved style — not a hardcoded value.
    assert!(zone_src.contains(".gap(BSpace::Seam)"), "zone gap must be the seam token");
    let tokens = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/v2/view/draw/flex/token.rs"),
    )
    .expect("flex tokens");
    assert!(
        tokens.contains("Self::Seam => ctx.style.resize.gap"),
        "the seam token must resolve from ctx.style.resize.gap"
    );
}

/// The seam overlay is rendered *outside* the area's flex children — it sits
/// in the zone container as an absolute overlay so it does not contribute to
/// the composition shape counts (top/middle/bottom/extras are area-level).
///
/// Regression: after decomposition, `composition_shape` must count zones not
/// seam overlays — so the extras count stays 0 for well-formed spaces.
#[test]
fn composition_shape_extras_zero_for_well_known_spaces() {
    use phiber::v2::state::BState;

    for &space in BSpaceType::EVERY {
        let state = BState::new();
        let shape = generated_shape(space, &state, false);
        assert_eq!(
            shape.extras, 0,
            "space {space:?} should have extras==0, got {}",
            shape.extras
        );
    }
}

/// A module is a file or a folder; read whichever spelling exists.
fn module_file(path: std::path::PathBuf) -> std::path::PathBuf {
    if path.exists() {
        return path;
    }
    // `x.rs` asked for, `x/mod.rs` exists.
    let folder = path.with_extension("").join("mod.rs");
    if folder.exists() {
        return folder;
    }
    // `x/mod.rs` asked for, `x.rs` exists.
    if path.file_name().is_some_and(|n| n == "mod.rs") {
        if let Some(parent) = path.parent() {
            let file = parent.with_extension("rs");
            if file.exists() {
                return file;
            }
        }
    }
    path
}
