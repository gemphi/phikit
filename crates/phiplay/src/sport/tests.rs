use super::*;
use crate::field::BMark;

#[test]
fn every_sport_lines_up_a_full_side_on_its_own_field() {
    for sport in BSport::ALL {
        let spec = sport.spec();
        assert_eq!(spec.positions.len(), spec.side as usize, "{}: one position per player", spec.label);
        assert!(spec.roster >= spec.side, "{}: the roster covers the side", spec.label);
        for p in spec.positions {
            assert!(p.at[0].abs() <= 1.0 && p.at[1].abs() <= 1.0, "{} {}", spec.label, p.code);
        }
        // Every mark lies on the floor; every paint on the playing area.
        let [fx, fy] = spec.field.floor_half();
        for mark in spec.field.marks {
            let points: Vec<[f32; 2]> = match *mark {
                BMark::Line { a, b } => vec![a, b],
                BMark::Arc { c, r, from, to } => {
                    // A full circle is written 0 → 360, which `rem_euclid` folds to 0.
                    let span = if to - from >= 360.0 { 360.0 } else { (to - from).rem_euclid(360.0) };
                    (0..=32)
                        .map(|k| {
                            let a = (from + span * k as f32 / 32.0).to_radians();
                            [c[0] + r * a.cos(), c[1] + r * a.sin()]
                        })
                        .collect()
                }
                BMark::Spot { c, .. } => vec![c],
                BMark::Frame { min, max } => vec![min, max],
            };
            for p in points {
                assert!(p[0].abs() <= fx + 1e-3 && p[1].abs() <= fy + 1e-3, "{}: {mark:?} off the floor", spec.label);
            }
        }
        for paint in spec.field.paint {
            assert!(spec.field.contains(paint.min) && spec.field.contains(paint.max), "{}: paint off the court", spec.label);
        }
        assert!(spec.ball.radius > 0.1 && spec.ball.radius < 0.125 && !spec.ball.panels.is_empty());
    }
}

#[test]
fn the_rulebooks_numbers_hold() {
    use crate::field::BMark::*;
    // FIBA: the three-point arc meets its straight lines.
    let b = BSport::Basketball.spec();
    let arc = b.field.marks.iter().find_map(|m| match *m { Arc { c, r, from, .. } if (r - 6.75).abs() < 1e-6 && c[0] > 0.0 => Some((c, r, from)), _ => None });
    let (c, r, from) = arc.expect("the east arc");
    let end = [c[0] + r * from.to_radians().cos(), c[1] + r * from.to_radians().sin()];
    assert!((end[1] - 6.6).abs() < 1e-3 && (end[0] - 11.0099).abs() < 1e-3, "{end:?}");
    // IFAB: the penalty arc's ends lie on the penalty area's line.
    let s = BSport::Soccer.spec();
    let (c, r, from) = s.field.marks.iter().find_map(|m| match *m { Arc { c, r, from, .. } if (r - 9.15).abs() < 1e-6 && c[0] > 1.0 => Some((c, r, from)), _ => None }).unwrap();
    let x = c[0] + r * from.to_radians().cos();
    assert!((x - 36.0).abs() < 1e-3, "the arc ends on the area's line: {x}");
    // Netball: every position may stand somewhere, and only the shooters
    // may enter the attacking circle.
    let n = BSport::Netball.spec();
    let shooters: Vec<_> = n.positions.iter().filter(|p| p.zones.contains(&BZone::AttackCircle)).map(|p| p.code).collect();
    assert_eq!(shooters, ["GS", "GA"]);
    assert_eq!(n.rules.hold, Some(3.0));
}
