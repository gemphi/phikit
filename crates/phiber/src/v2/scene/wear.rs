//! The wardrobe as a scene: a footwear set (`phiplay::kit::footwear`)
//! stood on a studio cyclorama — the sketches' navy, a warm key, a cool fill
//! and a rim — each design a pair and a collection of its own, the camera in
//! front at a shopper's height.
//!
//! `wear:<set>` opens a set's four designs side by side, `wear:<collection>`
//! one design of each of its sets, `wear:` one of each set of the Cut-Off
//! Series. What the Project Browser's Footwear folders open, and what a
//! background render takes (`cli::open`).

use super::play::{convert, demo};
use crate::phiphx::blend::AREA_ENERGY_SCALE;
use crate::phiphx::scene::{SceneCollection, SceneLight, SceneModel};
use glam::{Mat3, Mat4, Quat, Vec2, Vec3};
use phiplay::build::footwear::pair;
use phiplay::build::{BMat, BMesh, BPart};
use phiplay::build::footwear::picture::{BShot, left, picture};
use phiplay::kit::footwear::{BCollection, BFootwearSet, BFootwearSpec};

/// The sketches' ground.
const NAVY: phiplay::Rgb = phiplay::Rgb::hex(0x101B3E);
/// Between one pair and the next.
const SPACING: f32 = 0.55;

/// What a `wear:` path names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BShelf {
    Set(BFootwearSet),
    Collection(BCollection),
}

impl BShelf {
    pub fn by_slug(slug: &str) -> Option<BShelf> {
        match slug {
            "" => Some(BShelf::Collection(BCollection::CutOff)),
            s => BFootwearSet::by_slug(s).map(BShelf::Set).or_else(|| BCollection::ALL.into_iter().find(|c| c.slug() == s).map(BShelf::Collection)),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            BShelf::Set(s) => s.spec().label,
            BShelf::Collection(c) => c.label(),
        }
    }

    /// Its designs: a set's four, or the first of each of a collection's sets.
    pub fn designs(self) -> Vec<&'static BFootwearSpec> {
        match self {
            BShelf::Set(set) => set.designs().collect(),
            BShelf::Collection(c) => c.sets().filter_map(|s| s.designs().next()).collect(),
        }
    }
}

/// A transform at `from` whose −z looks at `to`, +y up — a camera's, an
/// area lamp's.
fn aim(from: Vec3, to: Vec3) -> Mat4 {
    let forward = (to - from).normalize();
    let right = forward.cross(Vec3::Z).normalize();
    Mat4::from_rotation_translation(Quat::from_mat3(&Mat3::from_cols(right, right.cross(forward), -forward)), from)
}

/// The cyclorama: a floor that sweeps up into a back wall with no corner,
/// `half` wide either side.
fn cyclorama(half: f32) -> BMesh {
    let mut rows: Vec<Vec2> = (0..=10).map(|k| Vec2::new(-3.0 + 3.6 * k as f32 / 10.0, 0.0)).collect();
    let (centre, radius) = (Vec2::new(0.6, 1.2), 1.2);
    rows.extend((1..=16).map(|k| { let a = std::f32::consts::FRAC_PI_2 * k as f32 / 16.0; centre + Vec2::new(a.sin(), -a.cos()) * radius }));
    rows.push(Vec2::new(1.8, 4.0));
    let mut m = BMesh::default();
    for (i, w) in rows.windows(2).enumerate() {
        let (a, b) = (w[0], w[1]);
        let n = Vec3::new(0.0, -(b.y - a.y), b.x - a.x).normalize();
        let base = m.vertex(Vec3::new(-half, a.x, a.y), n, Vec2::new(0.0, i as f32));
        m.vertex(Vec3::new(half, a.x, a.y), n, Vec2::new(1.0, i as f32));
        m.vertex(Vec3::new(half, b.x, b.y), n, Vec2::new(1.0, i as f32 + 1.0));
        m.vertex(Vec3::new(-half, b.x, b.y), n, Vec2::new(0.0, i as f32 + 1.0));
        m.triangle(base, base + 1, base + 2);
        m.triangle(base, base + 2, base + 3);
    }
    m
}

fn lamp(name: &str, from: Vec3, to: Vec3, colour: [f32; 3], watts: f32, size: f32) -> SceneLight {
    let at = aim(from, to);
    SceneLight {
        name: name.into(),
        location: from.into(),
        matrix_world: at.to_cols_array_2d(),
        color: colour,
        energy: watts * AREA_ENERGY_SCALE,
        kind: "AREA".into(),
        spot_size: 0.0,
        spot_blend: 0.0,
        size,
        size_y: size,
        shape: "SQUARE".into(),
    }
}

/// The designs `shelf` names, each a pair, on the cyclorama, under the
/// camera and the render settings of `demo`. `None` when the demo has no
/// camera to start from.
pub fn stage(shelf: BShelf, demo: &SceneModel) -> Option<SceneModel> {
    let list = shelf.designs();
    let mut model = SceneModel { world: demo.world.clone(), render: demo.render.clone(), playback: demo.playback.clone(), ..Default::default() };
    model.timeline = crate::phiphx::scene::SceneTimeline { frame_start: 1, frame_end: 1, frame_step: 1, current_frame: 1, fps: 24.0 };
    let sky = NAVY.linear();
    model.world.horizon_color = sky;
    model.world.background.color = sky;
    model.world.background.strength = 0.35;
    let width = SPACING * list.len() as f32;
    let floor = BPart::new("Cyclorama", cyclorama(width.max(2.0) + 2.0), BMat::plain("Cyclorama", NAVY, 0.62), Mat4::IDENTITY);
    let stage_index = model.add_object(convert::object(&floor), None);
    let mut groups: Vec<(String, Vec<usize>)> = vec![("Stage".into(), vec![stage_index])];
    // The figure faces −y; the pairs stand in a row across x, each turned a
    // little toward the camera's side so its outside shows.
    for (k, design) in list.iter().enumerate() {
        let x = (k as f32 - (list.len() as f32 - 1.0) / 2.0) * SPACING;
        let at = Mat4::from_translation(Vec3::new(x, 0.0, 0.0)) * Mat4::from_rotation_z(0.35);
        let mut members = Vec::new();
        for part in pair(design) {
            let placed = BPart { at, ..part };
            members.push(model.add_object(convert::object(&placed), None));
        }
        groups.push((design.label.to_string(), members));
    }
    // The light: warm key from the front left, cool fill from the right, a
    // rim from behind and above.
    let target = Vec3::new(0.0, 0.0, 0.25);
    model.lights.push(lamp("Key", Vec3::new(-2.2, -2.6, 2.6), target, [1.0, 0.86, 0.7], 900.0, 1.6));
    model.lights.push(lamp("Fill", Vec3::new(2.8, -2.2, 1.2), target, [0.7, 0.8, 1.0], 260.0, 2.2));
    model.lights.push(lamp("Rim", Vec3::new(0.6, 2.0, 2.4), target, [1.0, 0.95, 0.9], 700.0, 1.2));
    // The camera: in front, a little above the shafts' middle, framing the
    // row.
    let mut camera = demo.cameras.first()?.clone();
    let tall = list.iter().any(|d| matches!(d.set, BFootwearSet::TallBoot | BFootwearSet::ThighHigh | BFootwearSet::MidCalf | BFootwearSet::AboveKnee | BFootwearSet::MediumHigh));
    let aim_at = Vec3::new(0.0, 0.0, if tall { 0.3 } else { 0.1 });
    let from = aim_at + Vec3::new(0.0, -(width * 1.25 + 0.6), 0.45);
    camera.name = "Studio".into();
    camera.location = from.into();
    camera.matrix_world = aim(from, aim_at).to_cols_array_2d();
    camera.focal_length_mm = 50.0;
    model.render.camera = camera.name.clone();
    model.extra_locals.push((camera.name.clone(), camera.matrix_world));
    model.cameras.push(camera);
    for l in &model.lights {
        model.extra_locals.push((l.name.clone(), l.matrix_world));
    }
    let root = model.collections.first_mut()?;
    for (name, members) in groups {
        root.object_indices.retain(|i| !members.contains(i));
        root.children.push(SceneCollection { name, children: Vec::new(), object_indices: members, extra_names: Vec::new(), exclude: false, hide_viewport: false, visibility: Default::default() });
    }
    root.extra_names = model.cameras.iter().map(|c| c.name.clone()).chain(model.lights.iter().map(|l| l.name.clone())).collect();
    let r = &mut model.report;
    r.scene = format!("Footwear — {}", shelf.label());
    r.file_version = "phiplay".into();
    r.objects = model.objects.len();
    r.verts = model.objects.iter().map(|o| o.mesh.vertices.len()).sum();
    r.tris = model.objects.iter().map(|o| o.mesh.indices.len() / 3).sum();
    r.faces = r.tris;
    r.cameras = model.cameras.len();
    r.lights = model.lights.len();
    Some(model)
}

/// The stage a `wear:` path names, built — what a background render opens.
pub fn open(slug: &str) -> Result<SceneModel, String> {
    let shelf = BShelf::by_slug(slug).ok_or_else(|| format!("open: no footwear set or collection {slug:?}"))?;
    let demo = demo().ok_or("open: the embedded demo")?;
    stage(shelf, &demo).ok_or_else(|| "open: the demo has no camera".to_string())
}

/// A design drawn from `shot` — the Project Browser's card and views.
pub fn shot(design: &BFootwearSpec, shot: BShot, edge: u32) -> phiplay::build::BImage {
    picture(&left(&pair(design)), edge, shot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_set_is_its_four_pairs_on_the_stage_in_front_of_the_camera() {
        let Some(demo) = demo() else { return };
        let model = stage(BShelf::Set(BFootwearSet::Heel), &demo).expect("the stage");
        let root = &model.collections[0];
        let names: Vec<&str> = root.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["Stage", "Gala", "Rose", "Amber", "Pearl"]);
        assert!(root.children[1..].iter().all(|c| c.object_indices.len() >= 10), "a pair is many parts");
        assert_eq!(model.render.camera, "Studio");
        assert_eq!(model.lights.len(), 3);
        assert!(open("no-such-set").is_err());
        assert_eq!(BShelf::by_slug("").map(|s| s.designs().len()), Some(6));
        assert_eq!(BShelf::by_slug("first-batch").map(|s| s.designs().len()), Some(6));
    }
}
