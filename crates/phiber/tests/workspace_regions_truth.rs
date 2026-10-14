//! Every region Blender shows in a workspace, phiber shows too
//! (`truth/blender_workspaces.py`, `truth/workspaces.txt`).
//!
//! `workspace_truth.rs` holds the *areas* to `startup.blend` — which editor
//! sits where, and how wide. The truth file carries a further column its
//! parser throws away: the **regions** each area shows, with their sizes,
//! straight out of `screen.areas[].regions`. That column is what says a
//! Dope Sheet has a channel column 200 wide and an N panel 220 wide, and
//! that the Timeline in the Rendering workspace has neither.
//!
//! Nothing read it, so a host could declare three regions where Blender
//! shows four and every area test still passed. This reads it.
//!
//! The comparison is by region **kind**, not by pixel: a region's size is
//! the user's to drag, and `workspace_truth.rs` already holds the areas to
//! Blender's geometry. What matters here is that the region exists at all,
//! because a region phiber does not declare is a panel the user cannot
//! open and an edge they cannot drag.

use phiber::v2::host::BHostType;
use phiber::v2::region::BRegionType;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// One area as the truth file spells it, with the regions column this test
/// exists to read.
struct BlenderArea {
    workspace: String,
    kind: String,
    mode: String,
    region: BTreeSet<String>,
}

fn truth() -> Vec<BlenderArea> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/truth/workspaces.txt");
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let field: Vec<&str> = line.split('\t').collect();
            if field.len() < 8 {
                return None;
            }
            // `HEADER:1574x26 CHANNELS:200x381` → { HEADER, CHANNELS }.
            let region = field[7]
                .split_whitespace()
                .filter_map(|cell| cell.split(':').next())
                .map(str::to_string)
                .collect();
            Some(BlenderArea {
                workspace: field[0].to_string(),
                kind: field[1].to_string(),
                mode: field[2].to_string(),
                region,
            })
        })
        .collect()
}

/// Blender's region type name → phiber's. `WINDOW` is dropped by the dump
/// script (it is the canvas every area has); phiber calls it `Center` and
/// declares it, so it is added here rather than treated as missing.
fn region_of(name: &str) -> Option<BRegionType> {
    Some(match name {
        "HEADER" => BRegionType::Header,
        "FOOTER" => BRegionType::Footer,
        "TOOL_HEADER" => BRegionType::ToolHeader,
        "TOOLS" => BRegionType::Tools,
        "UI" => BRegionType::UI,
        "CHANNELS" => BRegionType::Channels,
        "NAVIGATION_BAR" => BRegionType::NavBar,
        "ASSET_SHELF" => BRegionType::AssetShelf,
        "ASSET_SHELF_HEADER" => BRegionType::AssetShelfHeader,
        "HUD" => BRegionType::HUD,
        "EXECUTE" => BRegionType::Execute,
        "TOOL_PROPS" => BRegionType::ToolProps,
        "PREVIEW" => BRegionType::Preview,
        // `FOOTER` aside, anything else Blender grows later is not a
        // region phiber claims to have; a `None` here is reported rather
        // than silently passing.
        _ => return None,
    })
}

/// Blender's editor → the phiber host that stands in for it. A Dope Sheet
/// in `TIMELINE` mode is phiber's Time host, not its Action host — the same
/// split Blender makes with one editor and two modes.
fn host_of(kind: &str, mode: &str) -> Option<BHostType> {
    Some(match (kind, mode) {
        ("DOPESHEET_EDITOR", "TIMELINE") => BHostType::Time,
        ("DOPESHEET_EDITOR", _) => BHostType::Action,
        ("VIEW_3D", _) => BHostType::View3D,
        ("IMAGE_EDITOR", _) => BHostType::Image,
        ("OUTLINER", _) => BHostType::Outliner,
        ("PROPERTIES", _) => BHostType::Buts,
        ("NLA_EDITOR", _) => BHostType::Nla,
        ("NODE_EDITOR", _) => BHostType::Node,
        ("GRAPH_EDITOR", _) => BHostType::Ipo,
        ("SEQUENCE_EDITOR", _) => BHostType::Seq,
        ("TEXT_EDITOR", _) => BHostType::Text,
        ("CONSOLE", _) => BHostType::Console,
        ("INFO", _) => BHostType::Info,
        ("FILE_BROWSER", _) => BHostType::File,
        ("SPREADSHEET", _) => BHostType::Spreadsheet,
        _ => return None,
    })
}

/// `BRegionType` and `BHostType` are small `Copy` enums without `Ord`, and
/// a test is no reason to add it to a shared type — the lists here are a
/// handful of entries, so they are `Vec`s searched linearly.
fn declared(host: BHostType) -> Vec<BRegionType> {
    host.spec().regions.iter().map(|region| region.kind).collect()
}

/// What Blender shows, per host, across every workspace it appears in.
fn blender_regions() -> Vec<(BHostType, Vec<BRegionType>)> {
    let mut out: Vec<(BHostType, Vec<BRegionType>)> = Vec::new();
    for area in truth() {
        let Some(host) = host_of(&area.kind, &area.mode) else { continue };
        let slot = match out.iter_mut().find(|(seen, _)| *seen == host) {
            Some(slot) => slot,
            None => {
                out.push((host, Vec::new()));
                out.last_mut().expect("just pushed")
            }
        };
        for name in &area.region {
            if let Some(region) = region_of(name) {
                if !slot.1.contains(&region) {
                    slot.1.push(region);
                }
            }
        }
    }
    out
}

/// How phiber provides a region Blender docks into the area.
///
/// Blender makes every panel a **region**: a docked strip with its own
/// draggable edge, listed in `screen.areas[].regions`. phiber declares some
/// of them the same way and provides others as floating chrome over the
/// canvas or as a `part::` table — the viewport's T and N panels float
/// (`canvas/view3d/corner.rs`: the overlay "keeps clear of the floating
/// toolbar and sidebar"), and the Dope Sheet's channel column is painted
/// inside its canvas (`row::set::paint_channels`, 200 wide as Blender's is).
///
/// Neither shape is wrong, but the difference has to be **recorded** rather
/// than discovered: a region phiber provides by another route is fine, and a
/// region it provides by no route at all is a panel the user cannot open.
/// This enum is that record, and the test below is what keeps it true.
#[derive(Debug, PartialEq, Eq)]
enum BProvided {
    /// Declared on the host, as Blender docks it.
    Region,
    /// Drawn over the canvas instead of docked beside it.
    Floating,
    /// Painted inside the canvas — no edge of its own to drag.
    InCanvas,
}

/// Where each region Blender shows in these two spaces actually comes from.
/// Adding a row is a decision; the test checks the row is true.
fn provided(host: BHostType, region: BRegionType) -> Option<BProvided> {
    use BProvided::{Floating, InCanvas, Region};
    Some(match (host, region) {
        // The Dope Sheet: header and footer are docked, the channel column
        // is painted in the canvas, and the sidebar floats.
        (BHostType::Action, BRegionType::Channels) => InCanvas,
        (BHostType::Action, BRegionType::UI) => Floating,
        // The Image editor's sidebar is `part::sidebar::image`'s table,
        // drawn as floating chrome (`the_image_editor_sidebar_has_its_panels`).
        (BHostType::Image, BRegionType::UI) => Floating,
        (BHostType::Image, BRegionType::Tools) => Floating,
        // The viewport's tool header, toolbar and asset shelf all float.
        (BHostType::View3D, BRegionType::ToolHeader) => Floating,
        (BHostType::View3D, BRegionType::Tools) => Floating,
        (BHostType::View3D, BRegionType::AssetShelf) => Floating,
        (BHostType::View3D, BRegionType::AssetShelfHeader) => Floating,
        // Everything else is expected to be docked, and the test says so.
        _ => Region,
    })
}

#[test]
fn every_region_blender_shows_is_one_phiber_provides_somehow() {
    let want = blender_regions();
    assert!(!want.is_empty(), "the truth file is there and parses");
    let host = [BHostType::Action, BHostType::Time, BHostType::Image, BHostType::Outliner, BHostType::Buts, BHostType::View3D];

    let mut wrong = Vec::new();
    for host in host {
        let Some((_, want)) = want.iter().find(|(seen, _)| *seen == host) else { continue };
        let have = declared(host);
        for &region in want {
            match provided(host, region) {
                // Claimed as docked: it must actually be declared.
                Some(BProvided::Region) if !have.contains(&region) => {
                    wrong.push(format!("{host:?} {region:?}: expected a docked region, and the host declares none"));
                }
                // Claimed as provided another way: it must NOT be declared,
                // or the record is stale and the note above is misleading.
                Some(BProvided::Floating | BProvided::InCanvas) if have.contains(&region) => {
                    wrong.push(format!("{host:?} {region:?}: recorded as not docked, but the host now declares it — update `provided`"));
                }
                None => wrong.push(format!("{host:?} {region:?}: Blender shows it and phiber provides it by no route at all")),
                _ => {}
            }
        }
    }
    assert!(wrong.is_empty(), "regions differ from startup.blend:\n  {}", wrong.join("\n  "));
}

/// The Dope Sheet's docked regions. Its channel column and sidebar come
/// from elsewhere (`provided`), but the header and footer are Blender's own
/// docked strips and must stay that way.
#[test]
fn the_dope_sheet_keeps_blenders_header_and_footer_docked() {
    let have = declared(BHostType::Action);
    assert!(have.contains(&BRegionType::Header), "Blender's Dope Sheet has a header");
    assert!(have.contains(&BRegionType::Footer), "and a footer");
    assert!(have.contains(&BRegionType::Center), "and the canvas between them");
}

/// The Image editor — the Rendering workspace's main area. Blender shows
/// `HEADER` and `UI`; the header is docked here and the sidebar floats, so
/// what this pins is the header and the canvas a render arrives in.
#[test]
fn the_image_editor_keeps_its_header_and_the_canvas_a_render_arrives_in() {
    let have = declared(BHostType::Image);
    assert!(have.contains(&BRegionType::Header));
    assert!(have.contains(&BRegionType::Center));
}

/// The Timeline is the same Blender editor in another mode, and Blender
/// gives it a header and nothing else — no channel column, no footer, no
/// side panel. A Timeline that grew them would be a Dope Sheet.
#[test]
fn the_timeline_is_a_header_and_a_canvas_and_no_more() {
    let have = declared(BHostType::Time);
    assert!(have.contains(&BRegionType::Header));
    assert!(have.contains(&BRegionType::Center));
    for region in [BRegionType::Channels, BRegionType::UI, BRegionType::Footer] {
        assert!(!have.contains(&region), "Blender's Timeline has no {region:?}");
    }
}

/// Every region name the dump produces is one this test knows how to map.
/// Without this a new Blender region would be skipped by `region_of` and
/// the comparison would quietly get weaker.
#[test]
fn every_region_blender_names_is_one_this_test_maps() {
    let unmapped: BTreeSet<String> = truth()
        .iter()
        .flat_map(|area| area.region.iter())
        .filter(|name| region_of(name).is_none())
        .cloned()
        .collect();
    assert!(unmapped.is_empty(), "regions the dump names and this test does not map: {unmapped:?}");
}

/// And every workspace in the dump resolves to a host, for the same reason.
#[test]
fn every_editor_blender_names_is_one_this_test_maps() {
    let unmapped: BTreeSet<String> = truth()
        .iter()
        .filter(|area| host_of(&area.kind, &area.mode).is_none())
        .map(|area| format!("{} ({})", area.kind, area.workspace))
        .collect();
    assert!(unmapped.is_empty(), "editors the dump names and this test does not map: {unmapped:?}");
}
