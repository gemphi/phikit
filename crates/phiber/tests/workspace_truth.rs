//! Every workspace lays out the editors Blender's `startup.blend` lays out,
//! where Blender puts them — against `truth/workspaces.txt`, which
//! `truth/blender_workspaces.py` writes with `bpy` from the factory startup
//! file (1920 × 1080). The window-level top bar, status bar and phiber's own
//! activity rail are outside the comparison.

use phiber::v2::area::BAreaType;
use phiber::v2::host::BHostType;
use phiber::v2::space::BSpaceType;
use std::collections::BTreeMap;

const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/truth/workspaces.txt");

/// Where Blender's right column starts, and how wide a full-width area is.
const RIGHT_COLUMN_X: i32 = 1579;
const FULL_WIDTH: i32 = 1500;

/// One Blender area as the truth file spells it.
struct BlenderArea {
    kind: String,
    mode: String,
    x: i32,
    y: i32,
    w: i32,
}

/// Blender's workspace name → phiber's space: by label, or by the slug
/// (`UV Editing` is phiber's *UV Lights*, `Texture Paint` its *Texturing*).
fn space_of(workspace: &str) -> Option<BSpaceType> {
    phiber::v2::app::load_ui::space_for(workspace)
}

/// The phiber host a Blender area type (and mode, and workspace) means.
fn host_of(area: &BlenderArea, space: BSpaceType) -> Option<BHostType> {
    Some(match (area.kind.as_str(), area.mode.as_str()) {
        ("OUTLINER", _) => BHostType::Outliner,
        ("PROPERTIES", _) => BHostType::Buts,
        ("VIEW_3D", _) => BHostType::View3D,
        ("DOPESHEET_EDITOR", "TIMELINE") => BHostType::Time,
        ("DOPESHEET_EDITOR", _) => BHostType::Action,
        ("IMAGE_EDITOR", "UV") => BHostType::Uv,
        ("IMAGE_EDITOR", _) => BHostType::Image,
        ("NODE_EDITOR", _) => match space {
            BSpaceType::Compositing => BHostType::Compositor,
            BSpaceType::GeometryNodes => BHostType::GeoNode,
            _ => BHostType::Shader,
        },
        ("SPREADSHEET", _) => BHostType::Spreadsheet,
        ("FILE_BROWSER", _) => BHostType::File,
        ("CONSOLE", _) => BHostType::Console,
        ("INFO", _) => BHostType::Info,
        ("TEXT_EDITOR", _) => BHostType::Text,
        _ => return None,
    })
}

/// Which of phiber's area kinds a Blender area is: the right column is
/// Right; a full-width area is Center when it is the topmost thing left of
/// that column and Bottom when something sits above it; the rest are Left
/// when they touch the window's left edge and Center otherwise.
fn kind_of(area: &BlenderArea, all: &[BlenderArea]) -> BAreaType {
    if area.x >= RIGHT_COLUMN_X {
        return BAreaType::Right;
    }
    if area.w >= FULL_WIDTH {
        let topmost = !all.iter().any(|other| other.x < RIGHT_COLUMN_X && other.y > area.y);
        return if topmost { BAreaType::Center } else { BAreaType::Bottom };
    }
    if area.x <= 2 { BAreaType::Left } else { BAreaType::Center }
}

fn truth() -> BTreeMap<String, Vec<BlenderArea>> {
    let mut out: BTreeMap<String, Vec<BlenderArea>> = BTreeMap::new();
    for line in std::fs::read_to_string(TRUTH).expect("truth file").lines() {
        let column: Vec<&str> = line.split('\t').collect();
        if column.len() < 7 {
            continue;
        }
        out.entry(column[0].to_string()).or_default().push(BlenderArea {
            kind: column[1].to_string(),
            mode: column[2].to_string(),
            x: column[3].parse().unwrap(),
            y: column[4].parse().unwrap(),
            w: column[5].parse().unwrap(),
        });
    }
    out
}

/// Where phiber knowingly differs, and why. Anything else is a failure.
const KNOWN: &[(BSpaceType, &str)] = &[
    (BSpaceType::Scripting, "Blender stacks two Outliners in the right column; two areas with one default host would share one slot key, so phiber has one"),
];

#[test]
fn every_workspace_has_blenders_editors_where_blender_puts_them() {
    let truth = truth();
    assert_eq!(truth.len(), 11, "eleven factory workspaces");
    let mut wrong = Vec::new();
    for (workspace, areas) in &truth {
        let space = space_of(workspace).unwrap_or_else(|| panic!("{workspace} is not a phiber space"));
        let mut want: Vec<(BAreaType, BHostType)> =
            areas.iter().filter_map(|area| Some((kind_of(area, areas), host_of(area, space)?))).collect();
        let mut have: Vec<(BAreaType, BHostType)> = space
            .spec()
            .areas
            .iter()
            .filter(|area| !matches!(area.default_host, BHostType::Topbar | BHostType::Statusbar | BHostType::Activity))
            .map(|area| (area.kind, area.default_host))
            .collect();
        if KNOWN.iter().any(|(known, _)| *known == space) {
            // The second Outliner is the one known difference.
            if let Some(index) = want.iter().rposition(|(kind, host)| *kind == BAreaType::Right && *host == BHostType::Outliner) {
                want.remove(index);
            }
        }
        let key = |pair: &(BAreaType, BHostType)| format!("{:?} {:?}", pair.0, pair.1);
        want.sort_by_key(key);
        have.sort_by_key(key);
        if want != have {
            wrong.push(format!(
                "{workspace}: Blender has {:?}, phiber has {:?}",
                want.iter().map(key).collect::<Vec<_>>(),
                have.iter().map(key).collect::<Vec<_>>()
            ));
        }
    }
    assert!(wrong.is_empty(), "workspaces differ from startup.blend:\n  {}", wrong.join("\n  "));
}

#[test]
fn the_animation_and_rendering_workspaces_open_where_blender_opens_them() {
    let truth = truth();
    let animation = &truth["Animation"];
    let camera = animation.iter().find(|a| a.kind == "VIEW_3D" && a.x <= 2).expect("a left viewport");
    assert_eq!(kind_of(camera, animation), BAreaType::Left);
    let sheet = animation.iter().find(|a| a.kind == "DOPESHEET_EDITOR").expect("a Dope Sheet");
    assert_eq!((kind_of(sheet, animation), sheet.mode.as_str()), (BAreaType::Bottom, "DOPESHEET"));
    let rendering = &truth["Rendering"];
    let image = rendering.iter().find(|a| a.kind == "IMAGE_EDITOR").expect("an Image editor");
    assert_eq!(kind_of(image, rendering), BAreaType::Center, "the Render Result view is the main area");
    assert!(rendering.iter().all(|a| a.kind != "OUTLINER"), "no Outliner in Blender's Rendering workspace");
}
