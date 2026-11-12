// SPDX-License-Identifier: Apache-2.0
//! Tests of `scene/update.rs` and `scene/background.rs`: the managers run
//! in Cycles' order and fill the device scene from a joined scene.
use crate::kernel::types::{SHADER_EXCLUDE_ANY, SHADER_EXCLUDE_DIFFUSE, SHADER_MASK};
use crate::kernel::{ShaderNodeType, svm_node};
use crate::scene::{Background, DeviceScene, Scene, Shader, Surface};
use crate::tests::kernel::rig_scene;

#[test]
fn the_shaders_are_the_scene_s_then_the_world_s_then_the_default_surface() {
    let mut scene = Scene::new();
    scene.add_shader(Surface::Diffuse { color: [0.5; 3] });
    scene.add_shader(Surface::Emission { color: [1.0; 3], strength: 2.0 });
    scene.world.color = [0.1, 0.2, 0.3];
    let surface = scene.shader_surface();
    assert_eq!(surface.len(), 4);
    assert_eq!(surface[2], Surface::Background { color: [0.1, 0.2, 0.3], strength: 1.0 });
    assert_eq!(surface[3], Surface::default());
}

#[test]
fn an_update_fills_every_manager_s_part_of_the_device_scene() {
    let mut scene = rig_scene();
    let dscene = scene.device_update();
    // Shaders: 1 of the scene, the world, the default.
    assert_eq!(dscene.shader.len(), 3);
    assert_eq!(dscene.svm_node[0], svm_node(ShaderNodeType::ShaderJump));
    // The camera.
    assert_eq!(dscene.data.float("cam", "width"), 16.0);
    // Objects and geometry.
    assert_eq!(dscene.object.len(), 1);
    assert_eq!(dscene.tri_vindex.len(), 2);
    assert!(!dscene.bvh_node.is_empty() || !dscene.bvh_leaf_node.is_empty());
    assert_eq!(dscene.object_flag.len(), 1);
    // The integrator and the film.
    assert!(dscene.data.int("integrator", "max_bounce") > 0);
    assert_eq!(dscene.data.int("film", "pass_stride"), 4);
    assert!(!dscene.lookup_table.is_empty());
}

#[test]
fn the_triangles_name_the_shader_of_their_slot() {
    let mut scene = rig_scene();
    let dscene = scene.device_update();
    assert!(dscene.tri_shader.iter().all(|shader| shader & SHADER_MASK == 0));
}

#[test]
fn the_world_is_the_shader_after_the_scene_s_and_every_ray_sees_it() {
    let mut scene = rig_scene();
    let dscene = scene.device_update();
    let shader = dscene.data.int("background", "surface_shader") as u32;
    assert_eq!(shader & SHADER_MASK, 1);
    assert_eq!(shader & SHADER_EXCLUDE_ANY, 0);
    assert_eq!(dscene.data.int("background", "transparent"), 0);
    assert_eq!(dscene.data.int("background", "volume_shader"), -1);
}

#[test]
fn a_world_with_no_node_or_a_hidden_one_is_excluded_from_the_rays_that_do_not_see_it() {
    let mut dscene = DeviceScene::new();
    let shader = Shader { id: 4, ..Shader::default() };
    Background::default().device_update(&mut dscene, &shader, false);
    assert_eq!(dscene.data.int("background", "surface_shader") as u32 & SHADER_EXCLUDE_ANY, SHADER_EXCLUDE_ANY);
    let hidden_from_diffuse = Background { visibility: Background::default().visibility & !crate::kernel::flag("PATH_RAY_VISIBILITY_DIFFUSE"), ..Background::default() };
    hidden_from_diffuse.device_update(&mut dscene, &shader, true);
    let word = dscene.data.int("background", "surface_shader") as u32;
    assert_eq!((word & SHADER_MASK, word & SHADER_EXCLUDE_ANY), (4, SHADER_EXCLUDE_DIFFUSE));
}

#[test]
fn a_transparent_film_makes_a_transparent_background() {
    let mut dscene = DeviceScene::new();
    Background { transparent: true, ..Background::default() }.device_update(&mut dscene, &Shader::default(), true);
    assert_eq!(dscene.data.int("background", "transparent"), 1);
}
