//! A node's solid, as the faces that are painted.
//!
//! The footprint is cut by the class's shape (`set.rs`): a rounded box whose
//! corners are arcs of the class radius, or a hexagon for the field class.
//! The solid is that footprint extruded from the floor to its height — a top
//! face, and one side face per footprint edge. Every face carries its outward
//! normal, so the painter can leave out the faces turned away from the eye and
//! light the rest (`paint.rs`).
//!
//! Rings run counter-clockwise seen from above, which is what makes the
//! outward normal of an edge `(dy, -dx)` rather than a guess.

use super::layout::BSolid;
use super::set::{BSolidShape, CORNER_SEGMENTS};

/// One flat face in graph space.
#[derive(Clone, Debug, PartialEq)]
pub struct BFace {
    pub ring: Vec<[f32; 3]>,
    pub normal: [f32; 3],
    /// True for the top, which takes the class colour; a side is the same
    /// colour lit differently.
    pub top: bool,
}

impl BFace {
    /// The face's centre, for depth ordering.
    pub fn centre(&self) -> [f32; 3] {
        let n = self.ring.len().max(1) as f32;
        let sum = self.ring.iter().fold([0.0; 3], |a, p| [a[0] + p[0], a[1] + p[1], a[2] + p[2]]);
        [sum[0] / n, sum[1] / n, sum[2] / n]
    }
}

impl BSolid {
    /// The footprint on the floor, counter-clockwise from above.
    pub fn footprint(&self) -> Vec<(f32, f32)> {
        match self.spec.shape {
            BSolidShape::Hex => self.hexagon(),
            BSolidShape::Rounded => self.rounded(),
        }
    }

    /// A flat-topped hexagon fitted to the footprint: its points a quarter in
    /// along the top and bottom edges, and at mid-height on the two sides.
    fn hexagon(&self) -> Vec<(f32, f32)> {
        let (x0, y0, x1, y1) = self.rect;
        let (qx, mid) = (self.width() / 4.0, (y0 + y1) / 2.0);
        vec![(x0 + qx, y0), (x1 - qx, y0), (x1, mid), (x1 - qx, y1), (x0 + qx, y1), (x0, mid)]
    }

    /// A rounded box: four arcs of the class radius, clamped so two corners
    /// never overlap on a small card.
    fn rounded(&self) -> Vec<(f32, f32)> {
        let (x0, y0, x1, y1) = self.rect;
        let r = self.spec.radius.min(self.width() / 2.0).min(self.depth() / 2.0).max(0.0);
        if r <= 0.0 {
            return vec![(x0, y0), (x1, y0), (x1, y1), (x0, y1)];
        }
        // Corner centres and the angle each arc starts at, counter-clockwise
        // from the bottom-right.
        let corners = [((x1 - r, y0 + r), -90.0_f32), ((x1 - r, y1 - r), 0.0), ((x0 + r, y1 - r), 90.0), ((x0 + r, y0 + r), 180.0)];
        let mut ring = Vec::with_capacity(4 * (CORNER_SEGMENTS + 1));
        for ((cx, cy), start) in corners {
            for step in 0..=CORNER_SEGMENTS {
                let a = (start + 90.0 * step as f32 / CORNER_SEGMENTS as f32).to_radians();
                ring.push((cx + r * a.cos(), cy + r * a.sin()));
            }
        }
        ring
    }

    /// The faces to paint: the top, then a side per footprint edge.
    pub fn faces(&self) -> Vec<BFace> {
        let ring = self.footprint();
        let h = self.height;
        let mut faces = Vec::with_capacity(ring.len() + 1);
        faces.push(BFace { ring: ring.iter().map(|&(x, y)| [x, y, h]).collect(), normal: [0.0, 0.0, 1.0], top: true });
        for (i, &(ax, ay)) in ring.iter().enumerate() {
            let (bx, by) = ring[(i + 1) % ring.len()];
            let (dx, dy) = (bx - ax, by - ay);
            let len = (dx * dx + dy * dy).sqrt();
            if len <= 1e-6 {
                continue;
            }
            faces.push(BFace {
                ring: vec![[ax, ay, 0.0], [bx, by, 0.0], [bx, by, h], [ax, ay, h]],
                normal: [dy / len, -dx / len, 0.0],
                top: false,
            });
        }
        faces
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::set::BSolidSpec;
    use crate::v2::theme::BNodeClass;

    fn solid(class: BNodeClass, rect: (f32, f32, f32, f32)) -> BSolid {
        let spec = BSolidSpec::of(class);
        BSolid { name: "n".into(), title: "n".into(), rect, class, spec, height: spec.height(30.0), rank: 0, muted: false }
    }

    /// Twice the signed area: positive for a counter-clockwise ring.
    fn area2(ring: &[(f32, f32)]) -> f32 {
        ring.iter().enumerate().map(|(i, &(x0, y0))| {
            let (x1, y1) = ring[(i + 1) % ring.len()];
            x0 * y1 - x1 * y0
        }).sum()
    }

    #[test]
    fn footprints_run_counter_clockwise_and_stay_inside_the_rect() {
        for class in [BNodeClass::Output, BNodeClass::Input, BNodeClass::Attribute, BNodeClass::Texture] {
            let s = solid(class, (10.0, -40.0, 150.0, 0.0));
            let ring = s.footprint();
            assert!(area2(&ring) > 0.0, "{class:?} is counter-clockwise");
            for (x, y) in &ring {
                assert!((10.0 - 1e-3..=150.0 + 1e-3).contains(x) && (-40.0 - 1e-3..=1e-3).contains(y), "{class:?} {x},{y}");
            }
        }
        assert_eq!(solid(BNodeClass::Attribute, (0.0, 0.0, 100.0, 50.0)).footprint().len(), 6, "a hex has six corners");
    }

    #[test]
    fn a_small_card_clamps_its_radius_rather_than_folding() {
        // A pill's 26 would cross itself on a card 20 deep; the arcs meet in
        // the middle instead and the ring stays simple.
        let ring = solid(BNodeClass::Input, (0.0, 0.0, 100.0, 20.0)).footprint();
        assert!(area2(&ring) > 0.0);
        assert!(ring.iter().all(|&(_, y)| (0.0 - 1e-3..=20.0 + 1e-3).contains(&y)));
    }

    #[test]
    fn every_side_faces_outward() {
        let s = solid(BNodeClass::Output, (0.0, 0.0, 100.0, 50.0));
        let (cx, cy) = s.centre();
        for face in s.faces().iter().filter(|f| !f.top) {
            let c = face.centre();
            let out = (c[0] - cx) * face.normal[0] + (c[1] - cy) * face.normal[1];
            assert!(out > 0.0, "side at {c:?} points {:?} away from the centre", face.normal);
            assert_eq!(face.ring.len(), 4);
        }
        assert!(s.faces()[0].top && s.faces()[0].ring.iter().all(|p| p[2] == s.height));
    }
}
