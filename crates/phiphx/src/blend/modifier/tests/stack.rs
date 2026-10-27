//! Array, Triangulate and Solidify: modifiers that rebuild the face list.

use super::super::array::BArray;
use super::super::solidify::BSolidify;
use super::super::triangulate::BTriangulate;
use super::{eval, quad};

#[test]
fn array_repeats_the_mesh_and_everything_that_travels_with_it() {
    let mut m = quad();
    eval(
        &mut m,
        &BArray {
            count: 3,
            offset_type: 1, // constant
            constant: glam::Vec3::new(2.0, 0.0, 0.0),
            relative: glam::Vec3::ZERO,
            offset_ob: 0,
        },
    );
    assert_eq!((m.vertices.len(), m.faces.len()), (12, 3));
    // Each copy is displaced by one whole step, not a cumulative drift.
    assert_eq!(m.vertices[4][0], 2.0);
    assert_eq!(m.vertices[8][0], 4.0);
    // The copies' faces index their own vertices.
    assert_eq!(m.faces.get(2), &[8, 9, 10, 11]);
    // UVs, materials, smooth flags and weights come along.
    assert_eq!(m.loop_uvs.len(), 12);
    assert_eq!(m.face_materials, vec![2, 2, 2]);
    assert_eq!(m.face_smooth, vec![true; 3]);
    assert_eq!(m.dverts.len(), 12);
}

#[test]
fn array_relative_offset_measures_the_mesh_itself() {
    let mut m = quad();
    eval(
        &mut m,
        &BArray {
            count: 2,
            offset_type: 2, // relative
            constant: glam::Vec3::ZERO,
            relative: glam::Vec3::new(1.0, 0.0, 0.0),
            offset_ob: 0,
        },
    );
    // The quad is one unit wide, so one relative step is one unit.
    assert_eq!(m.vertices[4][0], 1.0);
}

#[test]
fn a_single_copy_array_changes_nothing() {
    let mut m = quad();
    let before = m.clone();
    eval(
        &mut m,
        &BArray {
            count: 1,
            offset_type: 1,
            constant: glam::Vec3::new(5.0, 0.0, 0.0),
            relative: glam::Vec3::ZERO,
            offset_ob: 0,
        },
    );
    assert_eq!(m, before);
}

// ── Triangulate ───────────────────────────────────────────────────────────

#[test]
fn triangulate_splits_quads_and_keeps_what_rides_on_each_face() {
    let mut m = quad();
    eval(&mut m, &BTriangulate { min_vertices: 4 });
    assert_eq!(m.faces.len(), 2);
    assert_eq!(m.faces.get(0), &[0, 1, 2]);
    assert_eq!(m.faces.get(1), &[0, 2, 3]);
    // Both halves inherit the face's material and shading.
    assert_eq!(m.face_materials, vec![2, 2]);
    assert_eq!(m.face_smooth, vec![true, true]);
    // One UV per corner of the new faces.
    assert_eq!(m.loop_uvs.len(), 6);
}

#[test]
fn triangulate_leaves_faces_below_its_threshold_alone() {
    let mut m = quad();
    // Blender's default keeps quads: only faces of 5+ corners are split.
    eval(&mut m, &BTriangulate { min_vertices: 5 });
    assert_eq!(m.faces.len(), 1);
    assert_eq!(m.faces.get(0), &[0, 1, 2, 3]);
}

// ── Solidify ──────────────────────────────────────────────────────────────

#[test]
fn solidify_adds_a_second_shell_and_a_rim() {
    let mut m = quad();
    eval(
        &mut m,
        &BSolidify {
            thickness: 0.2,
            offset: -1.0,
            flag: 1 << 0, // MOD_SOLIDIFY_RIM
        },
    );
    // The original four vertices gain a shell of four more.
    assert_eq!(m.vertices.len(), 8);
    // Two shells, plus one rim face per boundary edge of the quad.
    assert_eq!(m.faces.len(), 2 + 4);
    // With offset -1 the outer shell stays put and the inner moves inward.
    assert!((m.vertices[0][2] - 0.0).abs() < 1e-6, "{:?}", m.vertices[0]);
    assert!((m.vertices[4][2] + 0.2).abs() < 1e-6, "{:?}", m.vertices[4]);
}

#[test]
fn a_negative_thickness_pushes_the_original_surface_outward_as_blender_does() {
    // The Female-01 garments: thickness -0.004, offset -1. Blender gives the
    // original vertices the larger of the two offsets (`INIT_VERT_ARRAY_OFFSETS`
    // without FLIP), so the correctly-wound surface moves 4 mm off the base
    // and the reversed copy stays on it. The other way round put the visible
    // surface exactly on the body the garment was made from.
    let mut m = quad();
    eval(&mut m, &BSolidify { thickness: -0.004, offset: -1.0, flag: 0 });
    assert!((m.vertices[0][2] - 0.004).abs() < 1e-6, "original moved out: {:?}", m.vertices[0]);
    assert!((m.vertices[4][2] - 0.0).abs() < 1e-6, "copy stays on the base: {:?}", m.vertices[4]);
}

#[test]
fn solidify_reverses_the_inner_shell_so_it_faces_outward() {
    let mut m = quad();
    eval(&mut m, &BSolidify { thickness: 0.1, offset: -1.0, flag: 0 });
    let inner = m.faces.get(1).to_vec();
    // The inner shell's corners run the other way round from the outer's.
    assert_eq!(inner, vec![7, 6, 5, 4]);
}


#[test]
fn solidify_keeps_uvs_on_both_shells_and_the_rim() {
    // A garment's print is an image texture: dropping the UVs when the
    // shell was added left every solidified piece of clothing untextured.
    let mut m = quad();
    m.loop_uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    eval(&mut m, &BSolidify { thickness: 0.1, offset: -1.0, flag: 1 << 0 });
    assert_eq!(m.loop_uvs.len(), m.faces.corners());
    // The copied shell carries the same UVs, reversed with its corners.
    assert_eq!(&m.loop_uvs[4..8], &[[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]);
}

/// Solid's Color ▸ Attribute through the stack: the colour attribute rides
/// with the UVs — copied by Array, reversed with the inner shell and laid
/// on the rim by Solidify, split with the faces by Triangulate, and
/// interpolated by Subdivision Surface.
#[test]
fn the_colour_attribute_rides_the_stack_as_the_uvs_do() {
    let paint = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 1.0, 1.0]];
    let mut m = quad();
    m.loop_colors = paint.to_vec();
    eval(&mut m, &BArray { count: 2, offset_type: 1, constant: glam::Vec3::new(2.0, 0.0, 0.0), relative: glam::Vec3::ZERO, offset_ob: 0 });
    assert_eq!(m.loop_colors.len(), 8, "the copy's corners take the colours");
    assert_eq!(&m.loop_colors[4..], &paint);
    let mut m = quad();
    m.loop_colors = paint.to_vec();
    eval(&mut m, &BSolidify { thickness: 0.1, offset: -1.0, flag: 1 << 0 });
    assert_eq!(m.loop_colors.len(), m.faces.corners());
    assert_eq!(&m.loop_colors[4..8], &[paint[3], paint[2], paint[1], paint[0]], "the inner shell, reversed with its corners");
    assert_eq!(m.loop_colors[8], m.loop_colors[11], "a rim quad takes its edge's two colours");
    let mut m = quad();
    m.loop_colors = paint.to_vec();
    eval(&mut m, &BTriangulate { min_vertices: 4 });
    assert_eq!(m.faces.len(), 2);
    assert_eq!(m.loop_colors.len(), 6);
    for (c, &v) in m.faces.iter().flatten().enumerate() {
        assert_eq!(m.loop_colors[c], paint[v as usize], "corner {c} keeps its vertex's colour");
    }
    let mut m = quad();
    m.loop_colors = paint.to_vec();
    eval(&mut m, &super::super::subsurf::BSubsurf::new(1, false, false, false));
    assert_eq!((m.faces.len(), m.loop_colors.len()), (4, 16));
    // Each sub-quad's third corner is the face centre: the mean colour.
    assert!(m.loop_colors[2].iter().all(|c| (c - 0.5).abs() < 1e-6), "{:?}", m.loop_colors[2]);
    assert_eq!(m.loop_colors[0], paint[0], "the original corner keeps its colour");
    let mut m = quad();
    eval(&mut m, &BTriangulate { min_vertices: 4 });
    assert!(m.loop_colors.is_empty(), "no attribute, none made up");
}

/// The open edges Solidify rims (`geom::boundary`): an open grid's are its
/// outline, a closed cube has none, and an edge three faces share is not one.
#[test]
fn the_boundary_is_the_edges_one_face_uses() {
    use super::super::geom::boundary;
    use super::super::BlendEvalMesh;
    // Two quads side by side: six outline edges, the shared one left out.
    let two = BlendEvalMesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [2., 0., 0.], [0., 1., 0.], [1., 1., 0.], [2., 1., 0.]],
        faces: [[0u32, 1, 4, 3], [1, 2, 5, 4]].iter().map(|q| q.to_vec()).collect(),
        ..Default::default()
    };
    assert_eq!(boundary(&two), vec![(0, 1), (0, 3), (1, 2), (2, 5), (3, 4), (4, 5)]);
    assert!(boundary(&super::cube()).is_empty(), "a closed mesh has no rim");
    // A fin: three triangles on the edge 0-1.
    let fin = BlendEvalMesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [0., -1., 0.], [0., 0., 1.]],
        faces: [[0u32, 1, 2], [1, 0, 3], [0, 1, 4]].iter().map(|t| t.to_vec()).collect(),
        ..Default::default()
    };
    assert!(!boundary(&fin).contains(&(0, 1)), "used three times is not open");
    assert_eq!(boundary(&fin).len(), 6);
}
