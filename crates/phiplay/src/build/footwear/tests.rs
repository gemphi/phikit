use super::*;
use crate::kit::footwear::{BExtra, BFootwearSet, DESIGNS};

fn part<'a>(parts: &'a [BPart], design: &str, name: &str) -> Option<&'a BPart> {
    parts.iter().find(|p| p.name == format!("{design} L {name}"))
}

fn span(p: &BPart, axis: usize) -> (f32, f32) {
    p.mesh.positions.iter().fold((f32::MAX, f32::MIN), |(lo, hi), q| (lo.min(q[axis]), hi.max(q[axis])))
}

#[test]
fn every_design_stands_on_the_floor_and_reaches_its_height() {
    for d in DESIGNS.iter() {
        let parts = pair(d);
        for p in &parts {
            assert!(p.mesh.positions.iter().flatten().all(|x| x.is_finite()), "{}", p.name);
        }
        let floor = parts.iter().map(|p| span(p, 2).0).fold(f32::MAX, f32::min);
        assert!(floor.abs() < 0.003, "{} stands on the floor: {floor}", d.label);
        assert!(part(&parts, d.label, "Sole").is_some(), "{}: a sole", d.label);
        if !d.open {
            let top = span(part(&parts, d.label, "Upper").expect("an upper"), 2).1;
            let want = match d.set {
                BFootwearSet::TallBoot | BFootwearSet::AboveKnee => 0.4,
                BFootwearSet::ThighHigh => 0.38,
                BFootwearSet::MidCalf | BFootwearSet::MediumHigh => 0.25,
                BFootwearSet::AnkleBoot | BFootwearSet::AboveAnkle => 0.14,
                _ => 0.025,
            };
            assert!(top > want, "{} reaches {top}, under {want}", d.label);
        }
        if d.heel.height() > 0.02 {
            let (lo, hi) = span(part(&parts, d.label, "Heel").unwrap_or_else(|| panic!("{}: a heel", d.label)), 2);
            assert!(lo.abs() < 0.003 && hi > d.heel.height() - 0.005 && hi < d.heel.height() + 0.03, "{}: heel {lo}..{hi}", d.label);
        }
        let l = part(&parts, d.label, "Sole").unwrap();
        let r = parts.iter().find(|p| p.name == format!("{} R Sole", d.label)).unwrap();
        assert!((span(l, 0).0 + span(r, 0).1).abs() < 1e-4, "{}: the right is the left's mirror", d.label);
    }
}

#[test]
fn the_trim_is_what_the_design_says_and_lies_on_the_shoe() {
    for d in DESIGNS.iter() {
        let parts = pair(d);
        let has = |name: &str| part(&parts, d.label, name).is_some();
        assert_eq!(has("Bow"), d.extra == BExtra::Ribbons, "{}", d.label);
        assert_eq!(has("Thong Out"), d.extra == BExtra::Thong, "{}", d.label);
        assert_eq!(has("Ankle Strap"), d.extra == BExtra::AnkleStrap, "{}", d.label);
        assert_eq!(has("Rim Scallops"), d.piping && !d.open, "{}", d.label);
        assert_eq!(has("Zip"), d.zip, "{}", d.label);
        assert_eq!(has("Foxing"), d.foxing && !d.open, "{}", d.label);
        // Nothing hangs off in the air: every part's box touches the shoe's.
        if let Some(upper) = part(&parts, d.label, "Upper") {
            let (ux, uy, uz) = (span(upper, 0), span(upper, 1), span(upper, 2));
            for p in parts.iter().filter(|p| p.name.starts_with(&format!("{} L Band", d.label))) {
                let (px, py, pz) = (span(p, 0), span(p, 1), span(p, 2));
                assert!(px.0 > ux.0 - 0.01 && px.1 < ux.1 + 0.01 && py.0 > uy.0 - 0.01 && py.1 < uy.1 + 0.01 && pz.1 < uz.1 + 0.01, "{}: {} lies on the upper", d.label, p.name);
            }
        }
    }
}

#[test]
fn a_card_and_a_sheet_show_the_shoe() {
    let parts = picture::left(&pair(&DESIGNS[0]));
    let card = picture::picture(&parts, 64, picture::BShot::SideOuter);
    let drawn = card.rgba.chunks(4).filter(|p| p[3] > 0).count();
    assert!(drawn > 64 * 64 / 10, "the shoe fills the card: {drawn} px");
    let sheet = picture::sheet(&parts, 32);
    assert_eq!((sheet.width, sheet.height), (32 * 6, 32));
}

#[test]
fn nothing_inside_the_shoe_shows_through_its_sole() {
    // Seen from below, the sole hides the insole: under every insole point
    // the sole's bottom lies lower, wherever the arch lifts it.
    for d in DESIGNS.iter() {
        let parts = pair(d);
        let (Some(bed), Some(sole)) = (part(&parts, d.label, "Insole"), part(&parts, d.label, "Sole")) else { continue };
        for q in &bed.mesh.positions {
            let under = sole.mesh.positions.iter().filter(|s| (s[0] - q[0]).hypot(s[1] - q[1]) < 0.008).map(|s| s[2]).fold(f32::MAX, f32::min);
            assert!(under < q[2] - 0.001, "{}: the insole at {q:?} pokes through the sole ({under})", d.label);
        }
    }
}

#[test]
fn every_mesh_is_clean() {
    // Indices in range, no triangle without area, every normal a unit.
    for d in DESIGNS.iter() {
        for p in pair(d) {
            let m = &p.mesh;
            let at = |i: u32| Vec3::from(m.positions[i as usize]);
            assert!(m.indices.iter().all(|&i| (i as usize) < m.positions.len()), "{}: an index out of range", p.name);
            let flat = m.indices.chunks(3).filter(|t| (at(t[1]) - at(t[0])).cross(at(t[2]) - at(t[0])).length() < 1e-10).count();
            assert_eq!(flat, 0, "{}: {} triangles, some without area", p.name, m.indices.len() / 3);
            let bent = m.normals.iter().filter(|n| (Vec3::from(**n).length() - 1.0).abs() > 0.01).count();
            assert_eq!(bent, 0, "{}: {bent} normals are not unit", p.name);
        }
    }
}

