// SPDX-License-Identifier: Apache-2.0
//! Tests of `blender/model.rs`, `sync.rs`, `mesh.rs` and `camera.rs`: scenes
//! of Blender's own render tests read by phiphx and joined into a Cycles
//! `Scene`, the counts checked against what phiphx reports.
use std::path::Path;

use crate::blender::*;
use crate::scene::{AttributeStandard, Scene, Surface};
use crate::tests::truth::verdict::root;
use crate::truth::{Image, TestScene};

/// The scene `category/name`, read and joined; `None` without the corpus.
fn join(category: &str, name: &str) -> Option<(Model, SceneSource, Scene, TestScene)> {
    let root = root()?;
    let test = TestScene::new(&root, category, name);
    let model = read_model(&test.blend).unwrap_or_else(|error| panic!("{category}/{name}: {error}"));
    let source = source_of_model(&model);
    let scene = sync_scene(&source);
    Some((model, source, scene, test))
}

fn triangle_count(model: &Model) -> usize {
    model.objects.iter().map(|object| object.mesh.indices.len() / 3).sum()
}

/// The counts every scene must keep: objects, triangles, vertices, and the
/// size, which must be that of the reference image Blender made.
fn assert_counts(category: &str, name: &str) -> (Model, Scene) {
    let Some((model, source, scene, test)) = join(category, name) else { return (Model::default(), Scene::new()) };
    assert_eq!(scene.object.len(), model.objects.len(), "{category}/{name} objects");
    assert_eq!(scene.triangle_count(), triangle_count(&model), "{category}/{name} triangles");
    assert!(scene.object.iter().all(|object| object.geometry < scene.mesh.len()));
    for (object, placed) in model.objects.iter().zip(&scene.object) {
        let mesh = &scene.mesh[placed.geometry];
        assert_eq!(mesh.triangle_count(), object.mesh.indices.len() / 3, "{category}/{name} {}", object.name);
        assert_eq!(mesh.vert_count(), object.mesh.vertices.len());
        let m = object.matrix_world;
        let tfm = placed.transform();
        assert_eq!([tfm.x[3], tfm.y[3], tfm.z[3]], [m[3][0], m[3][1], m[3][2]], "{} is placed where phiphx has it", object.name);
        // Every triangle's shader is one of the scene's.
        assert!(mesh.geometry.node.int_array("shader").iter().all(|&shader| (shader as usize) < scene.shader.len()));
    }
    // The meshes of a vertex-normal attribute have one normal a vertex.
    let normal = AttributeStandard::of("VERTEX_NORMAL");
    for mesh in &scene.mesh {
        let found = mesh.geometry.attribute.find_standard(normal).map(|attribute| attribute.size());
        assert_eq!(found, Some(mesh.vert_count()).filter(|_| mesh.triangle_count() > 0), "{category}/{name} normals");
    }
    // The render size is the size of Blender's own image.
    let reference = Image::read(&test.reference).unwrap();
    assert_eq!((source.width as usize, source.height as usize), (reference.width, reference.height), "{category}/{name} size");
    assert_eq!((scene.camera.node.int("full_width"), scene.camera.node.int("full_height")), (source.width, source.height));
    (model, scene)
}

#[test]
fn a_diffuse_scene_has_its_objects_triangles_and_a_perspective_camera() {
    let (model, scene) = assert_counts("bsdf", "diffuse");
    if model.objects.is_empty() {
        return;
    }
    assert_eq!((model.objects.len(), triangle_count(&model)), (2, 4192));
    // 50 mm lens on a 36 mm sensor, a square frame fitted on the sensor width.
    assert_eq!(scene.camera.kind(), "perspective");
    let fov = 2.0 * (0.5f32 * 36.0 / 50.0).atan();
    assert!((scene.camera.node.float("fov") - fov).abs() < 1e-6);
    let plane = scene.camera.bound("viewplane");
    assert_eq!((plane.left, plane.right, plane.bottom, plane.top), (-1.0, 1.0, -1.0, 1.0));
    assert_eq!((scene.camera.node.float("nearclip"), scene.camera.node.float("farclip")), (model.cameras[0].clip_start, model.cameras[0].clip_end));
    // The world is phiphx's background, colour times strength.
    assert_eq!(scene.world.color, model.world.background.color);
    assert_eq!(scene.world.strength, model.world.background.strength);
}

#[test]
fn an_emission_scene_has_an_emission_shader_and_a_diffuse_one() {
    let (model, scene) = assert_counts("shader", "emission");
    if model.objects.is_empty() {
        return;
    }
    assert_eq!(scene.shader.len(), 2);
    assert!(scene.surface.contains(&Surface::Emission { color: [0.8; 3], strength: 1.0 }), "{:?}", scene.surface);
    assert!(scene.surface.contains(&Surface::Diffuse { color: [0.5; 3] }));
    assert!(scene.shader.iter().enumerate().all(|(place, shader)| shader.id as usize == place));
    assert_eq!(scene.shader.iter().filter(|shader| shader.emission).count(), 1);
    // The emitter's triangles name the emission shader.
    let emitter = scene.surface.iter().position(|surface| matches!(surface, Surface::Emission { .. })).unwrap() as i32;
    assert!(scene.mesh.iter().any(|mesh| mesh.geometry.node.int_array("shader").iter().all(|&shader| shader == emitter)));
}

#[test]
fn an_orthographic_camera_gets_the_ortho_viewplane() {
    let (model, scene) = assert_counts("camera", "camera_ortho");
    if model.objects.is_empty() {
        return;
    }
    assert_eq!(scene.camera.kind(), "orthograph");
    // A square frame of ortho scale s: the viewplane is s/2 each way
    // (`blender_camera_viewplane`).
    let half = model.cameras[0].ortho_scale / 2.0;
    let plane = scene.camera.bound("viewplane");
    assert!((plane.left + half).abs() < 1e-5 && (plane.right - half).abs() < 1e-5, "{plane:?}");
    assert!((plane.bottom + half).abs() < 1e-5 && (plane.top - half).abs() < 1e-5);
    // The camera is where the object is, looking down Cycles' +Z.
    let m = model.cameras[0].matrix_world;
    let matrix = scene.camera.node.transform("matrix");
    assert_eq!([matrix.x[3], matrix.y[3], matrix.z[3]], [m[3][0], m[3][1], m[3][2]]);
    let look = matrix.column(2);
    let blender_forward = [-m[2][0], -m[2][1], -m[2][2]];
    let length = blender_forward.iter().map(|part| part * part).sum::<f32>().sqrt();
    assert!(look.iter().zip(blender_forward).all(|(a, b)| (a - b / length).abs() < 1e-5), "{look:?} against {blender_forward:?}");
}

#[test]
fn a_light_scene_keeps_three_objects_and_their_materials() {
    let (model, scene) = assert_counts("light", "point_light");
    if model.objects.is_empty() {
        return;
    }
    assert_eq!((model.objects.len(), triangle_count(&model)), (3, 6368));
    // Three distinct materials; a black emission of strength 1 is no emitter.
    assert_eq!(scene.shader.len(), 3);
    assert!(scene.surface.iter().all(|surface| matches!(surface, Surface::Diffuse { .. })));
}

#[test]
fn smooth_and_flat_triangles_are_told_apart() {
    let (model, scene) = assert_counts("mesh", "normal_types");
    if model.objects.is_empty() {
        return;
    }
    assert_eq!((model.objects.len(), triangle_count(&model)), (7, 480));
    let smooth: usize = scene.mesh.iter().map(|mesh| mesh.geometry.node.boolean_array("smooth").iter().filter(|&&smooth| smooth).count()).sum();
    let expected: usize = scene.object.iter().zip(&model.objects).map(|(_, object)| (0..object.mesh.indices.len() / 3).filter(|t| (0..3).any(|c| object.mesh.corners.normal(t * 3 + c) != [0.0; 3])).count()).sum();
    assert_eq!(smooth, expected);
    assert!(smooth > 0 && smooth < 480, "some smooth and some flat: {smooth}");
}

#[test]
fn the_size_is_the_resolution_times_the_percentage() {
    let Some((model, source, _, _)) = join("bsdf", "diffuse") else { return };
    let render = &model.render;
    assert_eq!((render.resolution_x, render.resolution_y, render.resolution_percent), (256, 256, 50));
    assert_eq!((source.width, source.height), (128, 128));
}

#[test]
fn objects_that_share_a_mesh_and_materials_share_the_cycles_mesh() {
    let Some((model, source, scene, _)) = join("light", "instanced_mesh_lights") else { return };
    assert_eq!(scene.object.len(), model.objects.len());
    assert!(source.mesh.len() <= source.object.len());
    assert!(scene.object.iter().all(|object| object.geometry < scene.mesh.len()));
}

#[test]
fn a_file_that_is_not_there_is_an_error_not_a_panic() {
    assert!(read_model(Path::new("no/such/file.blend")).is_err());
    assert!(read_blend(Path::new("no/such/file.blend")).is_err());
}

#[test]
fn a_source_with_no_camera_keeps_the_default_one_at_its_size() {
    let source = SceneSource { width: 64, height: 32, world_strength: 1.0, ..SceneSource::default() };
    let scene = sync_scene(&source);
    assert_eq!(scene.object.len(), 0);
    assert_eq!(scene.camera.node.int("full_width"), 1024, "no camera was synced");
}

#[test]
fn a_camera_with_a_wide_frame_fits_the_sensor_width() {
    let camera = CameraSource {
        name: "c".into(),
        matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 5.0, 1.0]],
        orthographic: false,
        lens: 50.0,
        sensor_width: 36.0,
        sensor_height: 24.0,
        sensor_fit: SensorFit::Auto,
        shift: [0.0, 0.0],
        ortho_scale: 6.0,
        clip_start: 0.1,
        clip_end: 100.0,
    };
    let (plane, aspect, sensor) = viewplane(&camera, 200, 100);
    assert_eq!((aspect, sensor), (2.0, 36.0));
    assert_eq!((plane.left, plane.right, plane.bottom, plane.top), (-2.0, 2.0, -1.0, 1.0));
    // A tall frame fits the sensor width in Auto too, but on its height side.
    let (plane, aspect, _) = viewplane(&camera, 100, 200);
    assert_eq!((aspect, plane.top), (2.0, 2.0));
    // Shift moves the plane by twice the aspect times the shift.
    let shifted = CameraSource { shift: [0.5, 0.0], ..camera };
    let (plane, _, _) = viewplane(&shifted, 100, 100);
    assert_eq!((plane.left, plane.right), (0.0, 2.0));
}
#[test]
fn the_output_a_cycles_render_uses_decides_the_surface() {
    // Two Material Output nodes, one for each engine: the file's blue
    // emission is the one Cycles shades, not the red one beside it.
    let Some((_, source, scene, _)) = join("shader", "material_output_selection") else { return };
    assert_eq!(source.mesh.iter().flat_map(|mesh| &mesh.material).count(), 1);
    assert!(scene.surface.contains(&Surface::Emission { color: [0.0, 0.0, 1.0], strength: 1.0 }), "{:?}", scene.surface);
    assert!(!scene.surface.iter().any(|surface| matches!(surface, Surface::Emission { color, .. } if color[0] > 0.5)));
}

#[test]
fn a_graph_that_is_more_than_one_closure_is_not_taken_for_one() {
    // `shader/emission` colours a diffuse with a checker texture: the join
    // does not pretend that it is the node's own colour.
    let Some((_, source, _, _)) = join("shader", "emission") else { return };
    let materials: Vec<_> = source.mesh.iter().flat_map(|mesh| &mesh.material).collect();
    assert!(materials.iter().any(|material| material.node_kind.iter().any(|kind| kind == "ShaderNodeTexChecker")));
    assert!(materials.iter().any(|material| material.node == NodeSurface::Unknown));
}

#[test]
fn the_scene_s_colour_management_lights_and_cycles_settings_are_read() {
    let Some((model, source, _, _)) = join("shader", "material_output_selection") else { return };
    assert_eq!(source.view_transform, model.render.view_transform);
    assert_eq!(source.light_count, 0);
    assert_eq!(source.cycles.get("samples"), Some(&25.0));
    assert_eq!(source.cycles.get("sampling_pattern"), Some(&1.0));
    let Some((_, lit, _, _)) = join("bsdf", "diffuse") else { return };
    assert_eq!(lit.light_count, 1);
}