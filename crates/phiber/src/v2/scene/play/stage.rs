//! The match's staging: the lamps as lights, the broadcast camera, the
//! players' colours, the read report — and the demo the figure comes from.

use crate::phiphx::blend::AREA_ENERGY_SCALE;
use crate::phiphx::scene::{SceneCamera, SceneLight, SceneMaterial, SceneModel};
use glam::{Mat4, Quat, Vec3};
use phiplay::BChoice;
use phiplay::build::venue::BLamp;

/// `m` recoloured: a flat colour, or the tint over its image.
pub(super) fn tint(m: &mut SceneMaterial, colour: phiplay::Rgb) {
    let [r, g, b] = colour.linear();
    m.base_color = [r, g, b, 1.0];
    m.viewport_color = [r, g, b, 1.0];
    m.bsdf.base = [r, g, b, 1.0];
}

/// A venue lamp as a light.
pub(super) fn light(k: usize, lamp: &BLamp) -> SceneLight {
    let (at, colour, energy, kind, size) = match *lamp {
        BLamp::Area { at, size, colour, power } => (at, colour, power * AREA_ENERGY_SCALE, "AREA", size),
        BLamp::Sun { at, colour, strength } => (at, colour, strength, "SUN", [0.0093; 2]),
    };
    SceneLight {
        name: format!("Lamp {}", k + 1),
        location: at.w_axis.truncate().into(),
        matrix_world: at.to_cols_array_2d(),
        color: colour.linear(),
        energy,
        kind: kind.to_string(),
        spot_size: 0.0,
        spot_blend: 0.0,
        size: size[0],
        size_y: size[1],
        shape: if kind == "AREA" { "RECTANGLE" } else { "SQUARE" }.to_string(),
    }
}

/// The broadcast camera: on the halfway line beyond the near touchline,
/// up in the first tiers, framing the whole field.
pub(super) fn broadcast(choice: &BChoice, demo: &SceneModel) -> Option<SceneCamera> {
    use phiplay::build::venue::bowl::{BACK, GAP, RISE};
    let [hx, hy] = choice.sport.spec().field.floor_half();
    // Back in the near stand — never past three quarters of its depth, so a
    // pitch's camera stays inside the bowl — and a couple of metres over the
    // terrace it stands on.
    let back = (f32::from(choice.venue.spec().tiers) * BACK * 0.75).min(2.0 + hx * 0.55);
    let terrace = ((back - GAP) / BACK).floor().max(0.0) * RISE;
    let from = Vec3::new(0.0, -(hy + back), terrace + 2.5 + hx * 0.08);
    // A camera looks down its −z with +y up.
    let forward = (Vec3::new(0.0, 0.0, 1.0) - from).normalize();
    let right = forward.cross(Vec3::Z).normalize();
    let look = Quat::from_mat3(&glam::Mat3::from_cols(right, right.cross(forward), -forward));
    let at = Mat4::from_rotation_translation(look, from);
    let mut camera = demo.cameras.first()?.clone();
    camera.name = "Broadcast".into();
    camera.location = from.into();
    camera.matrix_world = at.to_cols_array_2d();
    // Wide enough for a court's length; a pitch is followed in part, as a
    // broadcast's main camera frames it.
    camera.focal_length_mm = if hx > 25.0 { 20.0 } else { 32.0 };
    camera.clip_end = camera.clip_end.max(hx * 8.0);
    Some(camera)
}

/// The read report a file would carry: what was built, and how big.
pub(super) fn report(model: &mut SceneModel, choice: &BChoice) {
    let r = &mut model.report;
    r.scene = format!("{} — {} v {}", choice.sport.spec().label, choice.home.spec().name, choice.away.spec().name);
    r.file_version = "phiplay".into();
    r.objects = model.objects.len();
    // Shared meshes count once each time they are drawn, as Blender's
    // statistics count linked duplicates.
    r.verts = model.objects.iter().map(|o| o.mesh.vertices.len()).sum();
    r.tris = model.objects.iter().map(|o| o.mesh.indices.len() / 3).sum();
    r.faces = r.tris;
    r.cameras = model.cameras.len();
    r.lights = model.lights.len();
}

/// The embedded demo, opened: the figure the players are made of.
pub fn demo() -> Option<SceneModel> {
    let bytes = crate::v2::scene::template::embedded_bytes(crate::v2::scene::template::DEMO)?;
    crate::phiphx::Phx::open_bytes(crate::v2::scene::template::DEMO, bytes).ok()
}

/// The match a `play:` path names (`phiplay::pick::encode`'s line; empty
/// for the default match), built — what a background render opens.
pub fn open(choice: &str) -> Result<SceneModel, String> {
    let demo = demo().ok_or("open: the embedded demo")?;
    super::arena(&phiplay::pick::decode(choice), super::SECONDS, &demo).ok_or_else(|| "open: the demo has no rigged figure".to_string())
}
