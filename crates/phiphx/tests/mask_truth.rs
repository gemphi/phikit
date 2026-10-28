//! The Mask modifier (`modifier/mask.rs`) keeps what Blender keeps: eight
//! meshes (`fixtures/make_mask.py`) — Vertex Group mode plain and inverted,
//! the Smooth cut plain and inverted, a hexagon cut into two sub-faces,
//! Armature mode with one bone selected, the `EYE-L` stack on a closed
//! sphere (Subdivision, then an inverted mask at threshold 0) and a group
//! the mesh does not have — compared with Blender's own evaluation
//! (`fixtures/mask.tsv`): vertex for vertex, triangle count, and each
//! triangle's material in order.

use phiphx::phx::Phx;
use std::path::Path;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/mask.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/mask.tsv");

struct Truth {
    name: String,
    tris: usize,
    materials: String,
    verts: Vec<[f32; 3]>,
}

fn truth() -> Vec<Truth> {
    std::fs::read_to_string(TRUTH)
        .expect("truth file")
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            let [name, count, tris, materials, coords] = f.as_slice() else { return None };
            let nums: Vec<f32> = coords.split(' ').filter_map(|x| x.parse().ok()).collect();
            let verts: Vec<[f32; 3]> = nums.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
            let tris = tris.parse().ok()?;
            (verts.len() == count.parse::<usize>().ok()?).then(|| Truth {
                name: name.to_string(),
                tris,
                materials: materials.to_string(),
                verts,
            })
        })
        .collect()
}

#[test]
fn mask_matches_blender() {
    let model = Phx::open(Path::new(FILE)).expect("the mask file opens");
    let want = truth();
    assert_eq!(want.len(), 8, "eight grids in the truth");
    // The grid is 2 m across: 1e-5 of its size.
    let limit = 2e-5;
    for t in &want {
        let ob = model.objects.iter().find(|o| o.name == t.name).unwrap_or_else(|| panic!("{}", t.name));
        let ours = &ob.mesh.vertices;
        assert_eq!(ours.len(), t.verts.len(), "{}: vertex count", t.name);
        assert_eq!(ob.mesh.indices.len() / 3, t.tris, "{}: triangle count", t.name);
        let dist = |p: &[f32; 3], q: &[f32; 3]| (0..3).map(|i| (p[i] - q[i]).powi(2)).sum::<f32>().sqrt();
        // The Mask keeps its input's order; the Subdivision before it numbers
        // its new vertices differently from Blender's, so that one compares
        // each vertex with the nearest of the other set, both ways.
        let nearest = |a: &[[f32; 3]], b: &[[f32; 3]]| {
            a.iter().map(|p| b.iter().map(|q| dist(p, q)).fold(f32::MAX, f32::min)).fold(0.0, f32::max)
        };
        let max = match t.name.as_str() {
            "Subsurf" => nearest(ours, &t.verts).max(nearest(&t.verts, ours)),
            _ => ours.iter().zip(&t.verts).map(|(p, q)| dist(p, q)).fold(0.0, f32::max),
        };
        eprintln!("{:14} {} verts, {} tris, max {max:.3e}", t.name, ours.len(), t.tris);        assert!(max < limit, "{}: a vertex {max} from Blender's", t.name);
        let materials: String = ob.material_indices.iter().map(|m| m.to_string()).collect();
        // A single-material result leaves the per-triangle list empty.
        match materials.is_empty() {
            true => assert!(t.materials.chars().all(|c| c == t.materials.chars().next().unwrap_or('0')), "{}: materials", t.name),
            false => assert_eq!(materials, t.materials, "{}: each triangle's material", t.name),
        }
    }
    // The modes differ: the cut adds vertices, the invert keeps the rest.
    let count = |n: &str| want.iter().find(|t| t.name == n).map_or(0, |t| t.verts.len());
    assert!(count("Smooth") > count("Group"), "the smooth cut adds vertices");
    assert_eq!(count("Missing"), 100, "an unknown group leaves the grid whole");
}
