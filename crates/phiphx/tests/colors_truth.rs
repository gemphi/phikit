//! The active colour attribute as Blender saved it (`fixtures/make_colors.py`):
//! the cube's byte colours a corner — red, green, blue in turn, sRGB bytes
//! read back linear — and the plane's float colours a point, spread over
//! its corners. Skipped until the fixture is made (it takes Blender).

use phiphx::blend::BlendFile;
use std::path::PathBuf;

#[test]
fn the_cubes_corners_and_the_planes_points_keep_their_colours() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/colors.blend");
    let Ok(bytes) = std::fs::read(&path) else { return };
    let scene = BlendFile::parse_bytes(&bytes).unwrap().extract_scene().unwrap();
    let cube = scene.objects.iter().find(|o| o.name == "Cube").expect("the cube");
    assert_eq!(cube.loop_colors.len(), 24, "a colour a corner");
    let near = |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-3);
    assert!(near(cube.loop_colors[0], [1.0, 0.0, 0.0]) && near(cube.loop_colors[1], [0.0, 1.0, 0.0]) && near(cube.loop_colors[2], [0.0, 0.0, 1.0]), "{:?}", &cube.loop_colors[..3]);
    let plane = scene.objects.iter().find(|o| o.name == "Painted").expect("the plane");
    assert_eq!(plane.loop_colors.len(), 4);
    for (loop_i, &v) in plane.faces.iter().flatten().enumerate() {
        let want = [0.25 * v as f32, 0.5, 1.0 - 0.25 * v as f32];
        assert!(near(plane.loop_colors[loop_i], want), "corner {loop_i} is point {v}'s colour: {:?}", plane.loop_colors[loop_i]);
    }
}
