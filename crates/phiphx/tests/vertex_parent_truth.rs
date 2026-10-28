//! Vertex parenting (`blend/object/vertex_parent.rs`): children hung from
//! one or three vertices stand where Blender stands them — on a parent
//! under a Subdivision and on one its rig poses, empties, a mesh, a light,
//! a camera and a grandchild (`fixtures/make_vertex_parent.py`), against
//! the world matrices Blender dumped from the saved file
//! (`fixtures/vertex_parent_truth.txt`, `truth/blender_eval_dump.py`) —
//! on an eager open and on a progressive one, whose children move when
//! the evaluated meshes land.
//!
//! Two groups, because `give_parvert` indexes into the parent's evaluated
//! mesh: a child of the Armature-posed "Skin" is exact, since that mesh is
//! Blender's vertex for vertex, while a child of "Sheet" is only as good
//! as our Subdivision's vertex **order** — the same surface in another
//! order (nearest-vertex 2.8e-3 over the sheet, 3.2 index-wise), which is
//! `modifier/subsurf.rs`'s open item and not this port's. So the children
//! on it are held to 5e-3 and the exact group to 1e-6, and every child's
//! rotation — the frame built on the three vertices — to 1e-6 either way.

use glam::Mat4;
use phiphx::phx::Phx;
use phiphx::scene::SceneModel;
use std::collections::HashMap;
use std::path::Path;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/vertex_parent.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/vertex_parent_truth.txt");

/// Children whose parent's evaluated mesh is Blender's vertex for vertex:
/// the Armature-posed "Skin", and the index past the mesh (which takes the
/// parent's origin, with nothing evaluated).
const EXACT: [&str; 3] = ["Posed", "Posed One", "Past"];
/// Children of the subdivided "Sheet", bounded by our Subdivision's vertex
/// order rather than by the parenting.
const SUBDIVIDED: [&str; 6] = ["On One", "On Three", "Block", "Grand", "Lamp", "Cam"];

/// The `world` lines: name → Blender's world matrix (column layout).
fn truth() -> HashMap<String, Mat4> {
    std::fs::read_to_string(TRUTH)
        .expect("truth file")
        .lines()
        .filter_map(|l| {
            let mut it = l.split('\t');
            let (kind, name, _, v) = (it.next()?, it.next()?, it.next()?, it.next()?);
            let nums: Vec<f32> = v.split(' ').filter_map(|x| x.parse().ok()).collect();
            (kind == "world" && nums.len() == 16).then(|| (name.to_string(), Mat4::from_cols_array_2d(&std::array::from_fn(|c| std::array::from_fn(|r| nums[r * 4 + c])))))
        })
        .collect()
}

/// The world matrix of whatever `name` became in the scene.
fn world_of(model: &SceneModel, name: &str) -> Mat4 {
    let m = model
        .objects
        .iter()
        .find(|o| o.name == name)
        .map(|o| o.matrix_world)
        .or_else(|| model.empties.iter().find(|e| e.name == name).map(|e| e.matrix_world))
        .or_else(|| model.cameras.iter().find(|c| c.name == name).map(|c| c.matrix_world))
        .or_else(|| model.lights.iter().find(|l| l.name == name).map(|l| l.matrix_world))
        .unwrap_or_else(|| panic!("{name} in the scene"));
    Mat4::from_cols_array_2d(&m)
}

fn check(model: &SceneModel, how: &str) {
    let want = truth();
    let mut bad = Vec::new();
    for (name, tol) in EXACT.iter().map(|n| (*n, 1e-6)).chain(SUBDIVIDED.iter().map(|n| (*n, 5e-3))) {
        let theirs = want[name];
        let ours = world_of(model, name);
        let max = ours.to_cols_array().iter().zip(theirs.to_cols_array()).map(|(a, b)| (a - b).abs()).fold(0.0, f32::max);
        println!("{how} {name}: max {max:.2e} (tolerance {tol:e})");
        if max > tol {
            bad.push(format!("{how} {name}: max {max:e}\n ours {ours}\n theirs {theirs}"));
        }
    }
    assert!(bad.is_empty(), "{bad:#?}");
    // The frame itself is right whatever the parent's mesh: the rotation
    // `tri_to_quat` builds on the three vertices matches to a float's
    // accuracy, and only where a subdivided parent's vertex lands differs.
    for name in SUBDIVIDED {
        let (ours, theirs) = (world_of(model, name), want[name]);
        let rot = (0..3).flat_map(|c| (0..3).map(move |r| (c, r))).map(|(c, r)| (ours.col(c)[r] - theirs.col(c)[r]).abs()).fold(0.0, f32::max);
        assert!(rot < 1e-6, "{how} {name}: rotation off by {rot:e}");
    }
}

#[test]
fn children_stand_on_their_parents_evaluated_vertices() {
    let model = Phx::open(Path::new(FILE)).expect("the vertex parent file opens");
    check(&model, "eager");
    // The sheet is subdivided, so its vertices are not the saved ones, and
    // the two children hang from different ones.
    let want = truth();
    let sheet = model.objects.iter().find(|o| o.name == "Sheet").expect("Sheet");
    assert!(sheet.mesh.vertices.len() > 25, "the sheet is subdivided");
    assert!((want["On Three"].w_axis - want["On One"].w_axis).length() > 0.1, "the children sit on different vertices");
}

#[test]
fn a_progressive_open_moves_them_when_the_meshes_land() {
    let (mut base, deferred) = Phx::open_progressive(Path::new(FILE), &|_, _| true).expect("progressive open");
    let deferred = deferred.expect("the sheet and the skin have modifiers");
    // Before the modifiers land, the children sit on the SAVED vertices:
    // right for the vertex count, wrong for the place.
    let before = world_of(&base, "Posed");
    base.apply_deferred(deferred.finish());
    check(&base, "progressive");
    let after = world_of(&base, "Posed");
    assert!((before.w_axis - after.w_axis).length() > 1e-3, "the posed skin moved its child when it landed");
}
