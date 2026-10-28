//! The display list as a mesh (`mesh_nurbs_displist_to_mesh`,
//! `mesh_convert.cc`): every list's points become vertices of their own
//! (nothing is merged), a swept list's grid becomes quads, a fill its
//! triangles, and an unfilled outline loose edges.

use super::displist::{TDispList, TDlKind, DL_CYCL_U, DL_CYCL_V};
use super::{TMesh, CU_3D, CU_BACK, CU_FRONT};

pub fn from_displist(flag: u32, disp: &[TDispList]) -> TMesh {
    // 2D polys are filled as `DL_INDEX3`; an unfilled text keeps them.
    let conv_polys = !(flag & CU_3D == 0 && flag & (CU_FRONT | CU_BACK) != 0);
    let mut m = TMesh::default();
    let face = |m: &mut TMesh, f: Vec<u32>, dl: &TDispList| {
        m.faces.push(f);
        m.face_materials.push(dl.col.max(0) as u16);
        m.face_smooth.push(dl.smooth);
    };
    for dl in disp {
        let start = m.vertices.len() as u32;
        match dl.kind {
            TDlKind::Segm => {
                m.vertices.extend(dl.verts.iter().map(|v| v.to_array()));
                for a in 0..dl.parts {
                    let ofs = start + (a * dl.nr) as u32;
                    m.loose_edges.extend((1..dl.nr as u32).map(|b| (ofs + b - 1, ofs + b)));
                }
            }
            TDlKind::Poly if conv_polys => {
                m.vertices.extend(dl.verts.iter().map(|v| v.to_array()));
                for a in 0..dl.parts {
                    let ofs = start + (a * dl.nr) as u32;
                    let nr = dl.nr as u32;
                    m.loose_edges.extend((0..nr).map(|b| {
                        let (x, y) = (ofs + b, ofs + (b + 1) % nr);
                        (x.min(y), x.max(y))
                    }));
                }
            }
            TDlKind::Poly => {}
            TDlKind::Index3 => {
                m.vertices.extend(dl.verts.iter().map(|v| v.to_array()));
                for t in &dl.index {
                    face(&mut m, vec![start + t[0], start + t[2], start + t[1]], dl);
                }
            }
            TDlKind::Surf => {
                m.vertices.extend(dl.verts.iter().map(|v| v.to_array()));
                let (nr, parts) = (dl.nr as u32, dl.parts as u32);
                let (cyc_u, cyc_v) = (dl.flag & DL_CYCL_U != 0, dl.flag & DL_CYCL_V != 0);
                for a in 0..parts {
                    if !cyc_v && a == parts - 1 {
                        break;
                    }
                    let (mut p1, mut p2, mut p3, mut p4, b0) = match cyc_u {
                        true => {
                            let p1 = start + nr * a;
                            let p2 = p1 + nr - 1;
                            (p1, p2, p1 + nr, p2 + nr, 0)
                        }
                        false => {
                            let p2 = start + nr * a;
                            let p1 = p2 + 1;
                            (p1, p2, p1 + nr, p2 + nr, 1)
                        }
                    };
                    if cyc_v && a == parts - 1 {
                        p3 -= parts * nr;
                        p4 -= parts * nr;
                    }
                    for _ in b0..nr {
                        face(&mut m, vec![p1, p3, p4, p2], dl);
                        p4 = p3;
                        p3 += 1;
                        p2 = p1;
                        p1 += 1;
                    }
                }
            }
        }
    }
    m
}
