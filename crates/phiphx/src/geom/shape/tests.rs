//! Shapes: the table's names and parts, closed where they should be,
//! normals outward, the dice's pips, and the USDA they write.

use super::{PipFace, Shape, ShapeForm, ShapePart};
use std::collections::HashMap;

fn sub(a: [f32; 3], b: [f32; 3]) -> glam::Vec3 {
    glam::Vec3::from(a) - glam::Vec3::from(b)
}

/// Newell's normal of a face (its winding's direction, area-weighted).
fn face_normal(part: &ShapePart, face: &[u32]) -> glam::Vec3 {
    let p = |k: usize| glam::Vec3::from(part.point[face[k % face.len()] as usize]);
    (0..face.len()).fold(glam::Vec3::ZERO, |n, k| n + p(k).cross(p(k + 1)))
}

#[test]
fn table_lists_every_shape_with_its_part() {
    let all = Shape::all();
    for name in ["cube", "dice", "sphere", "cylinder", "cone", "torus", "plane", "capsule", "pyramid", "tetrahedron", "octahedron", "icosahedron"] {
        assert!(all.iter().any(|s| s.name == name), "shape.tsv has no {name}");
    }
    for shape in &all {
        let part: Vec<String> = shape.part().iter().map(|p| p.name.clone()).collect();
        let look: Vec<String> = shape.look.iter().map(|l| l.name.clone()).collect();
        assert_eq!(part, look, "{}: parts as the table names them", shape.name);
        for p in shape.part() {
            assert!(!p.count.is_empty(), "{}/{} has faces", shape.name, p.name);
            assert_eq!(p.normal.len(), p.index.len(), "{}/{}: a normal per corner", shape.name, p.name);
        }
    }
    assert_eq!(Shape::named("dice").unwrap().part().len(), 2);
}

#[test]
fn closed_shape_share_every_edge_once_each_way() {
    for shape in Shape::all().into_iter().filter(|s| s.form.closed()) {
        for part in shape.part() {
            let mut edge: HashMap<(u32, u32), usize> = HashMap::new();
            for face in part.face() {
                for k in 0..face.len() {
                    *edge.entry((face[k], face[(k + 1) % face.len()])).or_default() += 1;
                }
            }
            for (&(a, b), &n) in &edge {
                assert_eq!(n, 1, "{}/{}: edge {a}-{b} used {n} times one way", shape.name, part.name);
                assert_eq!(edge.get(&(b, a)), Some(&1), "{}/{}: edge {a}-{b} is open", shape.name, part.name);
            }
        }
    }
}

#[test]
fn plane_is_open_and_faces_up() {
    let plane = Shape::all().into_iter().find(|s| s.form == ShapeForm::Plane).unwrap();
    for part in plane.part() {
        for face in part.face() {
            assert!(face_normal(&part, face).normalize().y > 0.99);
        }
    }
}

#[test]
fn normal_point_outward() {
    for shape in Shape::all() {
        for part in shape.part() {
            let mut volume = 0.0f32;
            let mut at = 0usize;
            for face in part.face() {
                let n = face_normal(&part, face);
                for k in 0..face.len() {
                    let corner = glam::Vec3::from(part.normal[at + k]);
                    assert!(corner.dot(n) > 0.0, "{}/{}: a corner normal against its face", shape.name, part.name);
                }
                let p0 = part.point[face[0] as usize];
                for k in 1..face.len() - 1 {
                    let (a, b) = (sub(part.point[face[k] as usize], p0), sub(part.point[face[k + 1] as usize], p0));
                    volume += glam::Vec3::from(p0).dot(a.cross(b)) / 6.0;
                }
                at += face.len();
            }
            if shape.form.closed() {
                assert!(volume > 0.0, "{}/{}: faces wind inward (volume {volume})", shape.name, part.name);
            }
        }
    }
}

#[test]
fn dice_has_21_pips_and_opposite_face_sum_7() {
    let face = PipFace::all();
    assert_eq!(face.iter().map(|f| f.cell.len()).sum::<usize>(), 21);
    let mut value: Vec<u32> = face.iter().map(|f| f.value).collect();
    value.sort();
    assert_eq!(value, [1, 2, 3, 4, 5, 6]);
    for f in &face {
        assert_eq!(f.cell.len() as u32, f.value, "face {} shows {} pips", f.value, f.cell.len());
        let other = face.iter().find(|o| o.axis == f.axis && o.sign == -f.sign).expect("an opposite face");
        assert_eq!(f.value + other.value, 7);
    }
    let dice = Shape::named("dice").unwrap();
    let pip = dice.part().into_iter().find(|p| p.name == "pip").unwrap();
    assert_eq!(islands(&pip), 21, "21 separate pip discs");
    let flat = dice.size / 2.0 - dice.bevel;
    assert!(dice.spread + dice.pip <= flat, "every pip on the flat of its face");
}

/// Separate pieces of a part (faces joined through shared points).
fn islands(part: &ShapePart) -> usize {
    let mut root: Vec<usize> = (0..part.point.len()).collect();
    fn find(root: &mut Vec<usize>, i: usize) -> usize {
        let mut i = i;
        while root[i] != i {
            root[i] = root[root[i]];
            i = root[i];
        }
        i
    }
    for face in part.face() {
        for k in 1..face.len() {
            let (a, b) = (find(&mut root, face[0] as usize), find(&mut root, face[k] as usize));
            root[a] = b;
        }
    }
    (0..part.point.len()).filter(|&i| find(&mut root, i) == i).count()
}

#[test]
fn shape_usda_reads_back() {
    for shape in Shape::all() {
        let text = shape.to_usda();
        for needle in ["#usda 1.0", "metersPerUnit = 1", "upAxis = \"Y\"", "UsdPreviewSurface", "rel material:binding", "interpolation = \"faceVarying\""] {
            assert!(text.contains(needle), "{}: no {needle}", shape.name);
        }
        assert!(text.contains(&format!("defaultPrim = \"{}\"", shape.name)));
        let mesh = crate::import::usd::UsdImporter::import_usda_as_mesh(&text).expect("reads back");
        let part = shape.part();
        assert_eq!(mesh.len(), part.len(), "{}: a mesh per part", shape.name);
        for (m, p) in mesh.iter().zip(&part) {
            assert_eq!(m.name, p.name);
            assert_eq!(m.positions, p.point, "{}/{}: points", shape.name, p.name);
            assert_eq!(m.indices.len(), p.triangle_count(), "{}/{}: triangles", shape.name, p.name);
        }
    }
}

#[test]
fn phiogy_shape_file_match_generator() {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../appkit/app/phiogy/asset/shape");
    for shape in Shape::all() {
        let file = folder.join(format!("{}.usda", shape.name));
        let on_disk = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{}: {e} - run `cargo run -p phiphx --example shape_usd -- {}`", file.display(), folder.display()));
        assert_eq!(on_disk.replace('\r', ""), shape.to_usda(), "{} drifted from shape.tsv; regenerate with the shape_usd example", file.display());
    }
}
