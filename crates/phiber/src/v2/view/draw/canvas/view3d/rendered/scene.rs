//! Scene → phiray triangles and viewport → phiray camera.

use super::super::mesh::BMeshPalette;
use super::vert::BRayVerts;
use crate::v2::scene::camera::{BViewSetup, SCREEN};
use crate::v2::scene::{BObject3D, MESH_SCALE};
use crate::phiphx::scene::SceneNormals;
use crate::phiphx::spatial::SpatialObject3D;
use glam::Vec3;
use gpui::Hsla;
use super::world::{BRayLook, lit};
use phiray::{Camera, Instance, NO_TEX, Scene, Tri};

fn rgba(c: Hsla) -> [f32; 4] {
    let c = c.to_rgb();
    [c.r, c.g, c.b, c.a]
}

/// Theme colour (display) → linear albedo.
fn linear(c: Hsla) -> [f32; 4] {
    let [r, g, b, a] = rgba(c);
    [r.powf(2.2), g.powf(2.2), b.powf(2.2), a]
}

/// Longest texture side uploaded to the viewport, as Blender's Preferences →
/// Viewport → Textures → Limit Size. Material Preview reads the image at this
/// size; the final render is not the viewport's concern.
pub const TEXTURE_LIMIT: u32 = 2048;

/// The compact scene (Blender's Z-up world): each object an instance placed
/// by its matrix, its triangles in its own space with normals, colours and
/// textures.
pub fn build(scene: &[BObject3D], palette: &BMeshPalette) -> Scene {
    build_with(scene, palette, true)
}

/// A new pose of the scene for `Tracer::repose`: the same vertices and
/// faces, placed where the objects are now, and no image at all — the
/// tracer keeps the ones it has.
pub fn build_pose(scene: &[BObject3D], palette: &BMeshPalette) -> Scene {
    build_with(scene, palette, false)
}

/// Each object's placement, in `build`'s order: what moving objects whole
/// hands the tracer (`Tracer::reinstance`).
pub fn worlds(scene: &[BObject3D]) -> Vec<[[f32; 4]; 4]> {
    scene
        .iter()
        .map(|o| match o {
            BObject3D::Primitive { x, y, .. } => Instance::translation([x / MESH_SCALE, y / MESH_SCALE, 0.0]),
            BObject3D::Loaded { matrix_world, .. } => *matrix_world,
        })
        .collect()
}

fn build_with(scene: &[BObject3D], palette: &BMeshPalette, images: bool) -> Scene {
    let mut out = Scene::default();
    if images {
        palette.lead(&mut out.atlas);
    }
    let mut ids: std::collections::HashMap<*const Vec<u8>, u32> = std::collections::HashMap::new();
    let mut verts = BRayVerts::default();
    for (o, world) in scene.iter().zip(worlds(scene)) {
        // The object's own colour and its name's hash: Solid's Color ▸
        // Object and ▸ Random (`workbench::Material(Object &, random)`).
        let (name, color, shadow) = match o {
            BObject3D::Primitive { name, .. } => (name.as_str(), [1.0; 3], crate::phiphx::blend::BlendShadowCast::default()),
            BObject3D::Loaded { name, color, shadow, .. } => (name.as_str(), [color[0], color[1], color[2]], *shadow),
        };
        let id = out.instance_of(world, color, phiray::trace::hash::ObjectHash::of(name));
        // Solid and Wireframe follow Viewport Display ▸ Shadow
        // (`OB_DRAW_NO_SHADOW_CAST`, `workbench_shadow.cc`); the ray-traced
        // modes Visibility ▸ Ray ▸ Shadow (`OB_HIDE_SHADOW`).
        if !shadow.casts_in(palette.shading.uses_viewport_color()) {
            out.hide_shadow(id);
        }
        match o {
            BObject3D::Primitive { shape, .. } => {
                let (v, idx, n) = SpatialObject3D::buffers(shape.mesh_type());
                let base = match shape {
                    crate::v2::scene::BShape::Cube => palette.cube,
                    crate::v2::scene::BShape::Sphere => palette.sphere,
                    crate::v2::scene::BShape::Gem => palette.gem,
                };
                let p = |i: u32| v.get(i as usize).map(|v| [v.0, v.2, v.1]).unwrap_or_default();
                let nn = |i: u32| n.get(i as usize).map(|n| [n.0, n.2, n.1]).unwrap_or_default();
                for t in idx.chunks_exact(3) {
                    out.add(&Tri::new([p(t[0]), p(t[1]), p(t[2])], [nn(t[0]), nn(t[1]), nn(t[2])], linear(base)).with_pbr(0.45, 0.0));
                }
            }
            BObject3D::Loaded { mesh, uvs, .. } => {
                let n_of = |c: usize, v: usize| -> (u32, [f32; 3]) {
                    // Corner normals (split / smooth) interpolate; zero = flat.
                    // Per-face normals are flat shading: a zero normal tells the tracer to use the face normal.
                    match &mesh.normals {
                        _ if !mesh.corners.is_empty() => (mesh.corners.id(c), mesh.corners.normal(c)),
                        SceneNormals::PerVertex(n) => (0, n.get(v).copied().unwrap_or_default()),
                        SceneNormals::PerFace(_) => (0, [0.0; 3]),
                    }
                };
                let count = mesh.vertices.len();
                verts.start(count);
                for (face, t) in mesh.indices.chunks_exact(3).enumerate() {
                    let i = [t[0] as usize, t[1] as usize, t[2] as usize];
                    if i.iter().any(|&k| k >= count) {
                        continue;
                    }
                    let vid = [0, 1, 2].map(|k| {
                        let (nid, n) = n_of(face * 3 + k, i[k]);
                        verts.id(t[k], nid, || out.vertex(mesh.vertices[i[k]], n, uvs.get(i[k]).copied().unwrap_or_default()))
                    });
                    // The colour attribute at the corners (Color ▸ Attribute);
                    // a vertex shared by corners of two colours takes the last.
                    for (k, &c) in mesh.corners.colors.get(face * 3..face * 3 + 3).unwrap_or(&[]).iter().enumerate() {
                        out.paint(vid[k], c);
                    }
                    let color = o.triangle_color(face, palette.shading).unwrap_or_else(|| linear(palette.loaded));
                    // PHIBER_OVERRIDE=1 is Blender's View Layer → Material
                    // Override with a white diffuse: every surface the same,
                    // so a comparison measures the *light* and nothing else.
                    let slot = if std::env::var_os("PHIBER_OVERRIDE").is_some() { None } else { o.triangle_slot(face) };
                    let color = if slot.is_none() && std::env::var_os("PHIBER_OVERRIDE").is_some() { [0.8, 0.8, 0.8, 1.0] } else { color };
                    // Solid and Wireframe (Workbench) shade with the material's
                    // viewport roughness and metallic, the rest with its BSDF's.
                    let (rough, metal) = match (slot, palette.shading.uses_viewport_color()) {
                        (Some(m), true) => (m.viewport_pbr[0], m.viewport_pbr[1]),
                        (Some(m), false) => (m.roughness, m.metallic),
                        (None, true) => (0.4, 0.0),
                        (None, false) => (0.5, 0.0),
                    };
                    let mut tri = Tri::new([[0.0; 3]; 3], [[0.0; 3]; 3], color).with_pbr(rough, metal);
                    if let Some(b) = slot.map(|m| m.bsdf) {
                        tri.color[3] = b.alpha;
                        tri = tri
                            .with_optics(b.emitted(), b.transmission, b.ior)
                            .with_subsurface(b.subsurface, b.scatter())
                            .with_sheen(b.sheen, b.sheen_roughness, b.sheen_tint)
                            .with_coat(b.coat, b.coat_roughness)
                            .with_anisotropy(b.anisotropic, b.anisotropic_rotation);
                    }
                    // Workbench (Solid / Wireframe) shades from the material's
                    // viewport colour and reads no image at all — so no image
                    // goes to the GPU in those modes. Uploading a character's
                    // normal and roughness maps for a mode that never samples
                    // them is what pushed the atlas past one binding and made
                    // the GPU decline the whole scene. Color ▸ Texture is the
                    // exception: Solid then shows the base-colour image, and
                    // that image alone.
                    let uv_ok = images && i.iter().all(|&k| k < uvs.len());
                    let has_uv = uv_ok && !palette.shading.uses_viewport_color();
                    let has_base_image = has_uv || (uv_ok && palette.texture && palette.shading == crate::v2::scene::BShading::Solid);
                    let mut add = |t: &crate::phiphx::scene::SceneTexture| {
                        *ids.entry(std::sync::Arc::as_ptr(&t.rgba))
                            .or_insert_with(|| out.atlas.add_bounded(t.width, t.height, &t.rgba, TEXTURE_LIMIT))
                    };
                    let textured = slot.and_then(|m| m.texture.as_ref());
                    if let (Some(tex), true) = (textured, has_base_image) {
                        // Base colour is the tint the image is multiplied by (image = white when folded).
                        let tint = slot.map_or([1.0; 4], |m| m.base_color);
                        tri.color = [tint[0], tint[1], tint[2], tri.color[3]];
                        tri = tri.with_texture(add(tex), i.map(|k| uvs[k]));
                    }
                    // A linked image replaces the socket value, so the scalar becomes 1
                    // (emission keeps its strength) and the shader multiplies by the image.
                    let images = slot.map(|m| m.maps.images()).unwrap_or_default();
                    if has_uv && images.iter().any(Option::is_some) {
                        let maps = [0, 1, 2].map(|k| images[k].map_or(NO_TEX, &mut add));
                        tri.rough = if images[0].is_some() { 1.0 } else { tri.rough };
                        tri.metal = if images[1].is_some() { 1.0 } else { tri.metal };
                        if images[2].is_some() {
                            tri.emit = [slot.map_or(1.0, |m| m.bsdf.emission_strength); 3];
                        }
                        tri = tri.with_maps(maps, i.map(|k| uvs[k]));
                    }
                    let bumps = slot.and_then(|m| Some((m.normal_map.as_ref()?, m.normal_texture.as_ref()?.1)));
                    if let (Some((map, strength)), true) = (bumps, has_uv) {
                        tri = tri.with_normal_map(add(map), strength, i.map(|k| uvs[k]));
                    }
                    let mat = out.material(tri.mat());
                    out.face(vid, mat);
                }
            }
        }
    }
    out
}

/// The viewport's orbit camera (see `scene/camera.rs`) in phiray terms.
pub fn camera(yaw: f32, pitch: f32, pan: (f32, f32), zoom: f32, setup: BViewSetup, aspect: f32, look: &BRayLook) -> Camera {
    let (sy, cy) = yaw.to_radians().sin_cos();
    let (sp, cp) = pitch.to_radians().sin_cos();
    let right = Vec3::new(cy, -sy, 0.0);
    let up = Vec3::new(sp * sy, sp * cy, cp);
    let back = Vec3::new(-sy * cp, -cy * cp, sp);
    let scale = SCREEN * zoom;
    // Studio key light fixed to the view (used when the file has no lights).
    let sun = (right * 0.35 + up * 0.45 + back * 0.82).normalize();
    let ms = MESH_SCALE;
    let mut cam = Camera {
        right: right.to_array(),
        up: up.to_array(),
        back: back.to_array(),
        persp: setup.perspective as u32,
        scale,
        units: ms,
        pivot: [setup.pivot.0 / ms, setup.pivot.2 / ms, setup.pivot.1 / ms],
        aspect,
        sun: sun.to_array(),
        dist: 1.0 / (scale * setup.fov),
        pan: [pan.0, pan.1],
        // Nothing to draw once the floor and its three axes are all off.
        grid: if look.grid_hide & 0b1111 == 0b0111 { 0.0 } else { 1.0 },
        grid_hide: look.grid_hide,
        grid_color: rgba(look.grid),
        ..Default::default()
    };
    lit(&mut cam, look);
    cam
}

