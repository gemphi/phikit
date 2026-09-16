//! Bug-condition exploration tests — phiber-viewport-layout-fixes (Task 1).
//!
//! Feature: phiber-viewport-layout-fixes (bugfix)
//! Validates: Requirements 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7,
//!            2.8, 2.9, 2.10, 2.11, 2.12, 2.13, 2.14
//!
//! **Property 1: Bug Condition** — Viewport/Layout/Menu/Gizmo Defects Present.
//!
//! These assertions encode the EXPECTED (fixed) behaviour for each of the six
//! defects C1..C6. They are DELIBERATELY expected to FAIL on the current,
//! UNFIXED code: each failure is a concrete counterexample proving the defect
//! exists. Once the fixes in task 3 land, these same tests turn green and
//! validate the fix (task 3.7).
//!
//! The defects are deterministic (fixed sources / metric tokens / model
//! output), so each check is scoped to its concrete failing case rather than
//! randomised. View-layer geometry that requires a live gpui context is
//! asserted against the module source (paint code), while the pure model
//! layers (scene object mesh link, activity grouping) are asserted directly.
//!
//! Kit rule: phiber is a buikit app — these tests only read phiber's own
//! sources and public model types; they introduce no puikit/suikit imports
//! and no shared-logic duplication.

use std::path::{Path, PathBuf};

/// Absolute path to a file under `crates/phiber/src/`.
fn src(rel: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("src");
    for seg in rel.split('/') {
        p.push(seg);
    }
    p
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// Read every `.rs` file under `crates/phiber/src/<rel_dir>` (recursively)
/// and concatenate their source. Used where a render path has been split
/// across a folder + focused submodules (e.g. `view3d.rs` + `view3d/mesh.rs`)
/// so a contract check sees the whole path, not just the entry file.
fn read_tree(rel_dir: &str) -> String {
    fn walk(dir: &Path, out: &mut String) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                if let Ok(text) = std::fs::read_to_string(&path) {
                    out.push_str(&text);
                    out.push('\n');
                }
            }
        }
    }
    let mut base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    base.push("src");
    for seg in rel_dir.split('/') {
        base.push(seg);
    }
    // Include the sibling `<dir>.rs` entry file plus the `<dir>/` submodules.
    let mut out = String::new();
    let entry_file = base.with_extension("rs");
    if let Ok(text) = std::fs::read_to_string(&entry_file) {
        out.push_str(&text);
        out.push('\n');
    }
    walk(&base, &mut out);
    out
}

/// Strip full-line `//` comments so assertions match real code, not prose in
/// doc comments (several defect files describe the OLD behaviour in comments).
fn code_only(text: &str) -> String {
    text.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------------------
// C1 — Viewport must render real phiphx meshes, not flat 2D glyphs.
//
// Expected (fixed): each `BObject` carries a link to its phiphx mesh type
// (`PhiQuiMeshType` / `SpatialObject3D`) resolvable from the single scene
// table, and the View3D render path resolves a real mesh instead of drawing a
// flat `div` glyph.
//
// UNFIXED counterexample: `BObject` has only `name/shape/x/y` (no mesh link),
// and `view3d.rs` draws flat `object_glyph` `div`s. FAILS here — correct.
// ---------------------------------------------------------------------------

#[test]
fn c1_bobject_links_to_a_phiphx_mesh_type() {
    // The phiphx mesh library IS available to phiber (this import compiles),
    // so a mesh link is resolvable in principle.
    let _mesh_type_exists: fn() -> phiber::phiphx::PhiQuiMeshType =
        || phiber::phiphx::PhiQuiMeshType::Cube;

    let scene = code_only(&read(&src("v2/scene/mod.rs")));
    let links_mesh = scene.contains("PhiQuiMeshType")
        || scene.contains("SpatialObject3D")
        || scene.contains("phiphx")
        || scene.contains("mesh_type");
    assert!(
        links_mesh,
        "C1: BObject / scene::mod does not link any object to a phiphx mesh \
         type (PhiQuiMeshType / SpatialObject3D). Counterexample: BObject only \
         carries `name`, `shape`, `x`, `y` — no mesh reference, so no real \
         mesh is resolvable for Cube/Sphere/Gem from the single scene table."
    );
}

#[test]
fn c1_view3d_resolves_a_real_mesh_not_a_flat_glyph() {
    // The View3D render path was split into `view3d.rs` + focused submodules
    // (`view3d/mesh.rs`, etc.); read the whole tree so the mesh-sourcing
    // check sees the submodule that actually resolves the phiphx geometry.
    let view3d = code_only(&read_tree("v2/view/render/gpui/atom/canvas/view3d"));

    // Fixed code sources geometry from the shared phiphx mesh library.
    let sources_phiphx_mesh = view3d.contains("phiphx")
        || view3d.contains("PhiQuiMeshType")
        || view3d.contains("SpatialObject3D")
        || view3d.contains("SpatialMeshElement");
    assert!(
        sources_phiphx_mesh,
        "C1: View3D render path does not source any phiphx mesh geometry. \
         Counterexample: `object_glyph` renders BShape::{{Cube,Sphere,Gem}} as \
         flat decorative `div`s (tilted square / shaded circle / diamond) \
         instead of a resolved phiphx mesh."
    );

    // And it no longer draws objects via the flat-glyph path.
    assert!(
        !view3d.contains("fn object_glyph"),
        "C1: View3D still defines `object_glyph`, the flat-2D-glyph draw path. \
         Counterexample: objects are painted as flat `div`s, not real meshes."
    );
}

// ---------------------------------------------------------------------------
// C2 — Area and zone seams use the render pass's resolved resize geometry.
//
// The resize seam is intentionally independent from the boundary gap: the
// default 8px seam is a stronger affordance than the ordinary border while
// the 14px hit band remains zero-footprint. Both hierarchy levels must read
// the same resolved values instead of legacy global metrics.
// ---------------------------------------------------------------------------

#[test]
fn c2_resize_seams_use_resolved_geometry() {
    // The hierarchy routers own gap allocation; the post-decomposition seam
    // modules own the concrete hit band, line, and drag payload. Read both so
    // every resolved field remains covered at its focused ownership site.
    let area_seam = format!(
        "{}\n{}",
        code_only(&read(&src("v2/view/render/gpui/zone.rs"))),
        code_only(&read(&src("v2/view/render/gpui/zone/seam.rs")))
    );
    let zone_seam = format!(
        "{}\n{}",
        code_only(&read(&src("v2/view/render/gpui/space.rs"))),
        code_only(&read(&src("v2/view/render/gpui/space/seam.rs")))
    );

    for (name, source) in [("area", area_seam), ("zone", zone_seam)] {
        for field in [
            "visible_seam",
            "hit_width",
            "gap",
            "inset",
            "snap_threshold",
            "min_extent",
        ] {
            assert!(
                source.contains(&format!("c.style.resize.{field}")),
                "C2: {name} seam does not source `{field}` from the resolved style"
            );
        }
        assert!(
            !source.contains("METRICS.resize_") && !source.contains("METRICS.stroke"),
            "C2: {name} seam still reads legacy resize/stroke metrics"
        );
    }
}

// ---------------------------------------------------------------------------
// C3 — Stacked-area allocation uses the resolved resize gap. The absolute
// hit overlay must not replace that flex gap or add another flow child.
// ---------------------------------------------------------------------------

#[test]
fn c3_zone_view_uses_the_resolved_resize_gap() {
    let zone = code_only(&read(&src("v2/view/render/gpui/zone.rs")));

    assert!(
        zone.contains(".gap(px(c.style.resize.gap))"),
        "C3: zone::view does not separate adjacent areas with the resolved resize gap"
    );
    assert!(
        !zone.contains(".gap(px(METRICS.gap_"),
        "C3: zone::view still uses a legacy static gap metric"
    );
}

// ---------------------------------------------------------------------------
// C4 — An opened empty submenu (`Sub([])`) must render a VISIBLE (min-width,
// padded) menu panel, not a zero-child / omitted panel.
//
// UNFIXED counterexample: `popover::view` builds the panel then iterates the
// (empty) `body` list, yielding a panel with zero rows that collapses to zero
// height and reads as invisible/omitted. FAILS here — correct.
// ---------------------------------------------------------------------------

#[test]
fn c4_empty_submenu_renders_a_visible_panel() {
    let popover = code_only(&read(&src("v2/view/render/gpui/popover.rs")));

    // Fixed code makes an empty body still read as a visible panel: it either
    // guards `body.is_empty()` or gives the panel a min-height / placeholder
    // row so a zero-child flex column cannot collapse to nothing.
    let handles_empty_body = popover.contains("is_empty")
        || popover.contains("min_h")
        || popover.contains("placeholder");
    assert!(
        handles_empty_body,
        "C4: popover::view does not keep an opened empty submenu visible. \
         Counterexample: for a `Sub([])` item the panel is built and then zero \
         `body` rows are appended, so the flex column collapses to zero height \
         and the menu is invisible/omitted."
    );
}

// ---------------------------------------------------------------------------
// C5 — The activity bar must group entries into Top / Center / Bottom regions
// with flexible space between them, honouring each entry's `BActivityGroup`.
//
// This is directly observable on the pure model: `BActivityList::items`
// produces a flat run today. The fixed list emits region separators / spacers,
// so its output length exceeds the raw entry count (spacers between regions).
//
// UNFIXED counterexample: `items()` maps ENTRIES 1:1 into a single contiguous
// `Vec<BView>` of length == ENTRIES.len(), ignoring the group. FAILS — correct.
// ---------------------------------------------------------------------------

#[test]
fn c5_activity_list_partitions_into_three_regions_with_spacers() {
    use phiber::v2::part::activity::BActivityList;
    use phiber::v2::part::activity::set::{BActivityGroup, ENTRIES};
    use phiber::v2::state::BState;
    use phiber::v2::view::BViewList;

    // The entries genuinely span all three groups, so a correct layout MUST
    // produce a 3-region structure — the flattening is a real defect.
    let has_top = ENTRIES.iter().any(|e| e.group == BActivityGroup::Top);
    let has_center = ENTRIES.iter().any(|e| e.group == BActivityGroup::Center);
    let has_bottom = ENTRIES.iter().any(|e| e.group == BActivityGroup::Bottom);
    assert!(
        has_top && has_center && has_bottom,
        "C5 setup: ENTRIES should declare Top, Center and Bottom entries"
    );

    let s = BState::new();
    let items = BActivityList::new().items(&s);

    // A grouped three-region layout inserts flexible spacers BETWEEN the
    // regions, so the emitted view count must exceed the raw entry count.
    // The unfixed list is a single contiguous run: items.len() == ENTRIES.len().
    assert!(
        items.len() > ENTRIES.len(),
        "C5: BActivityList::items produced {} views for {} entries — a single \
         contiguous run with NO spacers. Counterexample: entries are flattened \
         into one run, ignoring BActivityGroup, so there is no Top/Center/Bottom \
         layout and Profile/Settings are not pushed to the bottom.",
        items.len(),
        ENTRIES.len()
    );
}

#[test]
fn c5_activity_list_source_honors_activity_group() {
    let list = code_only(&read(&src("v2/part/activity/list.rs")));
    assert!(
        list.contains("BActivityGroup") || list.contains(".group"),
        "C5: BActivityList::items never references BActivityGroup. \
         Counterexample: `items()` maps ENTRIES straight into a flat Vec<BView> \
         without partitioning by Top/Center/Bottom."
    );
}

// ---------------------------------------------------------------------------
// C6 — The navigate gizmo (axis compass + control cluster) must draw shapes
// matching Blender's navigate gizmo, not orbiting axis balls / generic discs.
//
// UNFIXED counterexample: `compass.rs` builds `pos_ball` / `neg_ball` orbiting
// discs; `gizmo.rs` builds generic round `nav_btn` icon discs. FAILS — correct.
// ---------------------------------------------------------------------------

#[test]
fn c6_compass_uses_blender_navigate_shapes_not_orbiting_balls() {
    let compass = code_only(&read(&src(
        "v2/view/render/gpui/atom/canvas/gizmo/compass.rs",
    )));

    let uses_orbiting_balls = compass.contains("fn pos_ball") || compass.contains("fn neg_ball");
    assert!(
        !uses_orbiting_balls,
        "C6: compass::view still constructs `pos_ball` / `neg_ball` orbiting \
         axis discs. Counterexample: the axis compass draws six balls orbiting \
         a hub rather than Blender's navigate axis-cluster shapes."
    );
}

#[test]
fn c6_control_cluster_uses_blender_shapes_not_generic_discs() {
    let gizmo = code_only(&read(&src("v2/view/render/gpui/atom/canvas/gizmo.rs")));

    // The generic disc control is `nav_btn` (a `rounded_full` icon disc used
    // for zoom / pan / camera / perspective). Blender-matching shapes replace
    // this generic disc builder.
    let uses_generic_discs = gizmo.contains("fn nav_btn");
    assert!(
        !uses_generic_discs,
        "C6: gizmo control cluster still builds generic round `nav_btn` icon \
         discs for zoom / pan / camera / perspective. Counterexample: the nav \
         controls are generic `rounded_full` discs, not Blender navigate gizmo \
         shapes."
    );
}
