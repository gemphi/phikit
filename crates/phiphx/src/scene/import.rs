//! Blend → scene conversion helpers. Split out of `scene/mod.rs` so both
//! the file stays under the 200-line cap and the pure geometry pipeline lives
//! in a focused module.
//!
//! Public entry point: [`SceneModel::from_blend`]. Everything else is `pub(super)`
//! so consumers go through the aggregate `SceneModel` rather than reaching
//! into per-object builders.

use super::{
    SceneAction, SceneCamera, SceneLight, SceneModel,
    SceneDisplayShading, SceneObject, SceneRender, SceneShadow, SceneSsao, SceneTimeline, SceneWorld,
};
use crate::blend::BlendScene;
use super::import_extras::{camera_from_blend, empty_from_blend, light_from_blend};

/// Map an imported [`BlendScene`] into a renderable [`SceneModel`].
///
/// This is the single blend→scene traversal in the codebase (Requirement 7.2);
/// it lives in the shared, kit-agnostic layer and nowhere under
/// `crates/phiber/src/`. It mirrors the fan-triangulation
/// `BlendScene::to_phx` already performs so both paths agree.
///
/// For every mesh object it:
///
/// - Triangulates each polygon like Blender (`polyfill.rs`): triangles as-is,
///   quads (0,1,2)+(0,2,3), n-gons ear-clipped on their dominant plane.
/// - Drops any corner whose vertex index is out of range for that object's
///   vertex buffer.
/// - Takes normals from `loop_normals` when present (one per triangle,
///   [`SceneNormals::PerFace`]), otherwise synthesizes a per-face normal.
///
/// Cameras and lights are partitioned into `cameras` / `lights` vectors so
/// the Rendering / Shading spaces can iterate them without a full scan.
///
/// Requirements: 2.5, 2.6, 7.2
impl SceneModel {
/// Build the runtime scene from a read `.blend` scene, freeing each object's
/// raw mesh buffers as soon as it is converted. Opening a 20 M-triangle file
/// otherwise holds the raw arrays and the runtime scene at the same time.
pub fn from_blend_owned(mut blend: BlendScene, source_path: &str) -> SceneModel {
    let raw = std::mem::take(&mut blend.objects);
    Self::build(raw, &blend, source_path)
}

/// Build from a borrowed scene (tests and small files; the raw arrays stay).
pub fn from_blend(blend: &BlendScene, source_path: &str) -> SceneModel {
    Self::build(blend.objects.clone(), blend, source_path)
}

/// `raw` is consumed: each object's buffers are dropped once it is converted.
fn build(raw: Vec<crate::blend::BlendSceneObject>, blend: &BlendScene, source_path: &str) -> SceneModel {
    let mut mesh_jobs: Vec<usize> = Vec::new();
    let mut cameras: Vec<SceneCamera> = Vec::new();
    let mut lights: Vec<SceneLight> = Vec::new();
    // blend index → `objects` index (None for cameras / lights).
    let mut remap: Vec<Option<usize>> = Vec::with_capacity(blend.objects.len());
    let mut object_types: Vec<String> = Vec::new();
    let mut empties = Vec::new();
    // Which blend object each camera, light, empty and mesh moves with.
    let mut motion = super::motion::SceneMotion { motion: blend.props.motion.clone(), ..Default::default() };
    let (object_count, face_count) = (raw.len(), raw.iter().map(|o| o.faces.len()).sum::<usize>());
    for (blend_i, obj) in raw.iter().enumerate() {
        match obj.object_type.as_str() {
            "CAMERA" => {
                cameras.push(camera_from_blend(obj));
                motion.camera.push(blend_i);
                remap.push(None);
            }
            "LAMP" | "LIGHT" => {
                lights.push(light_from_blend(obj));
                motion.light.push(blend_i);
                remap.push(None);
            }
            "EMPTY" => {
                empties.push(empty_from_blend(obj));
                motion.empty.push(blend_i);
                remap.push(None);
            }
            _ => {
                remap.push(Some(mesh_jobs.len()));
                object_types.push(obj.object_type.clone());
                motion.object.push(blend_i);
                mesh_jobs.push(blend_i);
            }
        }
    }
    // Triangulation + normals are independent per object.
    use rayon::prelude::*;
    // The saved 3D view resolves `Scene.camera` against the objects, so read it
    // before the conversion consumes them.
    let ui = super::ui::ui_from_blend(&blend.props.ui, &raw);
    // Collections and selection name objects by index — also read before the take.
    let collections = super::collection::collections_from_blend(blend, &raw, &remap);
    let selection = super::collection::selection_from_blend(blend, &raw);
    let parents = Self::parents_of(&raw);
    let locals: Vec<[[f32; 4]; 4]> = mesh_jobs.iter().map(|&i| raw[i].matrix_local).collect();
    let extra_locals = raw
        .iter()
        .filter(|o| matches!(o.object_type.as_str(), "CAMERA" | "LAMP" | "LIGHT" | "EMPTY"))
        .map(|o| (o.name.clone(), o.matrix_local))
        .collect();
    // The meshes a rig deforms keep their triangles, to be posed again.
    let rigs = super::SceneRigs {
        rigs: blend.props.rigs.rigs.clone(),
        skins: blend
            .props
            .rigs
            .skins
            .iter()
            .filter_map(|source| {
                let object = remap.get(source.object).copied().flatten()?;
                let rig = blend.props.rigs.rigs.iter().position(|r| r.address == source.armature)?;
                let tris = super::object::evaluated_triangles(&raw[source.object], blend.props.evaluated.get(&source.object));
                Some(std::sync::Arc::new(super::SceneSkin { object, motion: source.object, rig, source: source.clone(), tris }))
            })
            .collect(),
        saved: Vec::new(),
        drivers: blend.props.modifier_drivers.clone(),
        driven: blend
            .props
            .rigs
            .driven
            .iter()
            .filter_map(|source| {
                let object = remap.get(source.object).copied().flatten()?;
                Some(std::sync::Arc::new(super::SceneDriven { object, motion: source.object, source: source.clone() }))
            })
            .collect(),
    };
    motion.rigs = rigs;
    // The rigs as the file was saved, for the viewport to draw.
    if !motion.rigs.rigs.is_empty() {
        motion.rigs.saved = motion.rigs.bones_at(&motion.solved_at(blend.props.frame_current as f32));
    }
    // `into_par_iter` hands each object to the closure by value, so its raw
    // vertices, faces and loop data are freed as the conversion walks on.
    // Collect skipped-modifier warnings before raw is consumed.
    let skipped_warnings: Vec<String> = raw.iter().flat_map(|obj| {
        obj.skipped_modifiers.iter().map(move |m| format!("{}: unsupported modifier {}", obj.name, m))
    }).collect();
    let wanted: std::collections::HashSet<usize> = mesh_jobs.iter().copied().collect();
    let (mut objects, uv_maps): (Vec<SceneObject>, Vec<std::sync::Arc<super::SceneUvMap>>) = raw
        .into_par_iter()
        .enumerate()
        .filter(|(i, _)| wanted.contains(i))
        .map(|(i, obj)| (evaluated_object(&obj, blend.props.evaluated.get(&i)), std::sync::Arc::new(super::SceneUvMap::of(&obj))))
        .unzip();
    let p = &blend.props;
    let missing = super::texture::attach(&mut objects, p, source_path);
    let report = super::report::SceneReadReport {
        scene: blend.name.clone(),
        file_version: p.file_version.clone(),
        duration_ms: 0,
        objects: object_count,
        verts: objects.iter().map(|o| o.mesh.vertices.len()).sum(),
        faces: face_count,
        tris: objects.iter().map(|o| o.mesh.indices.len() / 3).sum(),
        cameras: cameras.len(),
        lights: lights.len(),
        empties: empties.len(),
        memory_bytes: 0,
        warnings: {
            let mut w = missing;
            w.extend(skipped_warnings);
            // The world's HDRI, when it is not where the file says: Blender
            // would light the scene magenta; the renderer uses its studio
            // light instead (`BlendWorld::missing`).
            if let Some(hdri) = &p.world.missing {
                w.push(format!("World: cannot read '{hdri}' — lit with Blender's studio light (forest) instead of the missing-image magenta"));
            }
            w
        },
    };
    SceneModel {
        report,
        objects,
        source_path: Some(source_path.to_string()),
        timeline: SceneTimeline {
            frame_start: p.frame_start,
            frame_end: p.frame_end,
            frame_step: p.frame_step,
            current_frame: p.frame_current,
            fps: p.fps,
        },
        world: SceneWorld {
            horizon_color: p.world_color,
            background: p.world.clone(),
            environment: super::texture::environment(p, source_path),
        },
        render: SceneRender {
            camera: p.camera.clone(),
            cycles: p.cycles.clone(),
            resolution_x: p.resolution_x,
            resolution_y: p.resolution_y,
            resolution_percent: p.resolution_percent,
            engine: p.engine.clone(),
            view_transform: p.view_transform.clone(),
            look: p.look.clone(),
            exposure: p.exposure,
            gamma: p.gamma,
            film_transparent: p.film_transparent,
            output: p.output.clone(),
            border: p.border,
            movie: p.movie,
            pass: p.pass,
            ssao: SceneSsao { samples: p.ssao[2].max(1.0) as u32, distance: p.ssao[0], attenuation: p.ssao[1] },
            shadow: SceneShadow { light_direction: p.light_direction, shift: p.shadow[0], focus: p.shadow[1] },
            display: SceneDisplayShading::of(&p.display_shading, p.render_aa),
        },
        cameras,
        lights,
        actions: p
            .actions
            .iter()
            .map(|a| SceneAction {
                name: a.name.clone(),
                frame_start: a.frame_start,
                frame_end: a.frame_end,
                fcurve_count: a.fcurve_count,
                keys: a.keys.clone(),
                users: a.users.clone(),
                curves: a.curves.clone(),
                slots: a.slots.clone(),
                bindings: a.bindings.clone(),
                manual_range: a.manual_range,
                cyclic: a.cyclic,
            })
            .collect(),
        nla: p.nla.clone(),
        drivers: p.drivers.clone(),
        collections,
        selection,
        object_types,
        empties,
        ui,
        motion,
        markers: p.markers.clone(),
        playback: p.playback,
        parents,
        locals,
        extra_locals,
        uv_maps,
        cursor: p.cursor,
        compositor: p.compositor.clone(),
    }
}

/// `(child, parent)` names from each object's `*parent` pointer; a parent
/// outside the scene's objects is left out.
fn parents_of(raw: &[crate::blend::BlendSceneObject]) -> Vec<(String, String)> {
    let by_addr: std::collections::HashMap<u64, &str> =
        raw.iter().filter(|o| o.block_address != 0).map(|o| (o.block_address, o.name.as_str())).collect();
    raw.iter()
        .filter(|o| o.parent_address != 0)
        .filter_map(|o| Some((o.name.clone(), (*by_addr.get(&o.parent_address)?).to_string())))
        .collect()
}
}

use super::object::evaluated_object;
