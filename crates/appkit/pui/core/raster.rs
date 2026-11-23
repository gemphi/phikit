//! A picture of triangles in space, made on the processor: seen from a
//! camera (`lens.rs`), each in its colour, lit as a studio lights a thing
//! (`sun.rs`), the nearer over the farther, a curve shaded round from
//! the normals at its corners (`scan.rs`). The picture is a frame of a
//! film (blue, green, red, how solid, a row after a row), so a part
//! shows it as it shows any picture that moves. It is made larger than
//! it is shown and made smaller (`smooth`), so no edge is jagged. The
//! work is shared out, a share a thread. A picture can be kept with its
//! depths and drawn over again: a still world made once, and what moves
//! on it each frame, with the shadow it casts.

use super::lens::{Lens, cross, dot, sub, unit};
use super::scan::{Band, Flat, Ink, fill};
use super::{PCamera, PSun};

/// Triangles in one colour (red, green, blue, 0 to 1), and the way the
/// surface faces at each corner of each (none: each faces its own way).
pub struct PSolid<'a> {
    pub triangle: &'a [[[f32; 3]; 3]],
    pub normal: &'a [[[f32; 3]; 3]],
    pub colour: [f32; 3],
}

/// A picture and the depth of each of its points. It may be a piece cut
/// from a larger one (`cut`): then it stands at `at` in a picture of the
/// size `whole`, and what is drawn into it falls where it would there.
#[derive(Debug, Clone, PartialEq)]
pub struct PRaster {
    pub width: u32,
    pub height: u32,
    pub pixel: Vec<u8>,
    pub depth: Vec<f32>,
    pub at: [u32; 2],
    pub whole: [u32; 2],
}

/// A triangle's three corners, or three ways its corners face.
type Tri = [[f32; 3]; 3];

/// How many threads share some work: as many as there are, but no
/// more than there are `least` of it, so a small job is not slowed by
/// starting threads it does not need.
fn share(work: usize, least: usize) -> usize {
    std::thread::available_parallelism().map_or(4, |n| n.get()).min(work / least.max(1) + 1).max(1)
}

/// The triangles put on a picture of a size, a share a thread: cut at
/// the near plane, and those wholly off the picture left out.
fn flats(lens: Lens, raster: &PRaster, near: f32, shape: &[PSolid]) -> Vec<Flat> {
    let (w, h) = (raster.width as f32, raster.height as f32);
    let (whole, at) = (raster.whole.map(|v| v as f32), raster.at.map(|v| v as f32));
    let all: Vec<(&Tri, Option<&Tri>, [f32; 3])> = shape.iter().flat_map(|s| s.triangle.iter().enumerate().map(move |(i, t)| (t, s.normal.get(i), s.colour))).collect();
    std::thread::scope(|scope| {
        let made: Vec<_> = all.chunks(all.len().div_ceil(share(all.len(), 8192)).max(1)).map(|part| scope.spawn(move || {
            let mut out = Vec::new();
            for (t, normal, colour) in part {
                let face = || [unit(cross(sub(t[1], t[0]), sub(t[2], t[0]))); 3];
                let normal = normal.copied().unwrap_or_else(face);
                for (t, normal) in near_side(&lens, **t, normal, near) {
                    let corner = t.map(|p| lens.project(p, whole[0], whole[1])).map(|c| [c[0] - at[0], c[1] - at[1], c[2]]);
                    let off = |i: usize, most: f32| corner.iter().all(|c| c[i] < 0.0) || corner.iter().all(|c| c[i] > most);
                    match off(0, w) || off(1, h) {
                        true => {}
                        false => out.push(Flat { corner, normal, colour: *colour }),
                    }
                }
            }
            out
        })).collect();
        made.into_iter().flat_map(|m| m.join().unwrap_or_default()).collect()
    })
}

/// The part of a triangle at or past the near plane, as no, one or two
/// triangles, the ways its corners face carried along with them: a
/// triangle that reaches behind the eye is cut where it crosses, not
/// left out.
fn near_side(lens: &Lens, t: Tri, n: Tri, near: f32) -> Vec<(Tri, Tri)> {
    let ahead = t.map(|p| dot(sub(p, lens.eye), lens.forward) - near);
    let mix = |a: usize, b: usize| {
        let k = ahead[a] / (ahead[a] - ahead[b]);
        ([0, 1, 2].map(|i| t[a][i] + (t[b][i] - t[a][i]) * k), [0, 1, 2].map(|i| n[a][i] + (n[b][i] - n[a][i]) * k))
    };
    let kept: Vec<usize> = (0..3).filter(|&i| ahead[i] >= 0.0).collect();
    match kept.len() {
        3 => vec![(t, n)],
        0 => Vec::new(),
        // One corner ahead: the triangle from it to where its two edges cross.
        1 => {
            let (a, b, c) = (kept[0], (kept[0] + 1) % 3, (kept[0] + 2) % 3);
            let ((pb, nb), (pc, nc)) = (mix(a, b), mix(a, c));
            vec![([t[a], pb, pc], [n[a], nb, nc])]
        }
        // Two ahead: the four-sided rest, as two triangles.
        _ => {
            let c = (0..3).find(|i| !kept.contains(i)).unwrap_or(0);
            let (a, b) = ((c + 1) % 3, (c + 2) % 3);
            let ((pa, na), (pb, nb)) = (mix(a, c), mix(b, c));
            vec![([t[a], t[b], pb], [n[a], n[b], nb]), ([t[a], pb, pa], [n[a], nb, na])]
        }
    }
}

impl PRaster {
    /// A picture of one colour (blue, green, red, how solid) and nothing
    /// in it, every depth as far as can be.
    pub fn new(width: u32, height: u32, ground: [u8; 4]) -> PRaster {
        let n = width as usize * height as usize;
        PRaster { width, height, pixel: ground.repeat(n), depth: vec![f32::MAX; n], at: [0, 0], whole: [width, height] }
    }

    /// The piece of it from a point, so wide and so high (as much of
    /// that as it has), to draw into apart from the rest.
    pub fn cut(&self, x: u32, y: u32, width: u32, height: u32) -> PRaster {
        let (x, y) = (x.min(self.width), y.min(self.height));
        let (w, h) = (width.min(self.width - x) as usize, height.min(self.height - y) as usize);
        let wide = self.width as usize;
        let rows = (y as usize..y as usize + h).map(|row| row * wide + x as usize);
        let pixel = rows.clone().flat_map(|start| self.pixel[start * 4..(start + w) * 4].iter().copied()).collect();
        let depth = rows.flat_map(|start| self.depth[start..start + w].iter().copied()).collect();
        PRaster { width: w as u32, height: h as u32, pixel, depth, at: [self.at[0] + x, self.at[1] + y], whole: self.whole }
    }

    fn bands(&mut self, flat: &[Flat], sun: &PSun, view: [f32; 3], ink: Ink) {
        let width = self.width as usize;
        let rows = (self.height as usize).div_ceil(share(self.width as usize * self.height as usize, 65536)).max(1);
        std::thread::scope(|scope| {
            for (i, (pixel, depth)) in self.pixel.chunks_mut(rows * width * 4).zip(self.depth.chunks_mut(rows * width)).enumerate() {
                scope.spawn(move || fill(flat, Band { pixel, depth, width, top: i * rows }, sun, view, ink));
            }
        });
    }

    /// Draw shapes into it, the nearer over what is there.
    pub fn draw(&mut self, camera: &PCamera, shape: &[PSolid], sun: &PSun) {
        let lens = camera.lens();
        let flat = flats(lens, self, camera.near, shape);
        self.bands(&flat, sun, lens.forward, Ink::Lit);
    }

    /// Darken by a share what the triangles fall on, where they lie on
    /// what is drawn already: a shadow laid on a floor.
    pub fn shadow(&mut self, camera: &PCamera, triangle: &[[[f32; 3]; 3]], share: f32) {
        let lens = camera.lens();
        let shape = [PSolid { triangle, normal: &[], colour: [0.0; 3] }];
        let flat = flats(lens, self, camera.near, &shape);
        self.bands(&flat, &PSun::default(), lens.forward, Ink::Shadow(share));
    }

    /// The picture made `by` times smaller across and down, each point
    /// the mean of those it stands for, so an edge falls off smoothly.
    /// Where a point is partly ground, it is that much less solid, and
    /// its colour is the solid points' own.
    pub fn smooth(&self, by: u32) -> (u32, u32, Vec<u8>) {
        let by = by.max(1) as usize;
        let (w, h, wide) = (self.width as usize / by, self.height as usize / by, self.width as usize);
        let mut out = vec![0u8; w * h * 4];
        for (i, point) in out.chunks_mut(4).enumerate() {
            let (x, y) = (i % w * by, i / w * by);
            let mut sum = [0u32; 4];
            for (dx, dy) in (0..by).flat_map(|dy| (0..by).map(move |dx| (dx, dy))) {
                let at = ((y + dy) * wide + x + dx) * 4;
                let a = self.pixel[at + 3] as u32;
                (0..3).for_each(|k| sum[k] += self.pixel[at + k] as u32 * a);
                sum[3] += a;
            }
            let solid = sum[3].max(1);
            (0..3).for_each(|k| point[k] = (sum[k] / solid) as u8);
            point[3] = (sum[3] / (by * by) as u32) as u8;
        }
        (w as u32, h as u32, out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera() -> PCamera {
        PCamera { eye: [0.0, 1.0, 3.0], at: [0.0, 0.0, 0.0], up: [0.0, 1.0, 0.0], fov: 45.0, near: 0.01 }
    }

    #[test]
    fn a_triangle_ahead_is_drawn_in_its_colour_and_the_nearer_wins() {
        let far = [[[-1.0, -1.0, -1.0], [1.0, -1.0, -1.0], [0.0, 1.0, -1.0]]];
        let near = [[[-0.2, -0.2, 0.5], [0.2, -0.2, 0.5], [0.0, 0.2, 0.5]]];
        let mut raster = PRaster::new(64, 48, [0, 0, 0, 0]);
        let sun = PSun::default();
        raster.draw(&camera(), &[PSolid { triangle: &far, normal: &[], colour: [1.0, 0.0, 0.0] }, PSolid { triangle: &near, normal: &[], colour: [0.0, 0.0, 1.0] }], &sun);
        let middle = camera().project([0.0, 0.0, 0.5], 64.0, 48.0);
        let at = (middle[1] as usize * 64 + middle[0] as usize) * 4;
        assert_eq!(&raster.pixel[at..at + 4], &[255, 0, 0, 255], "the near blue one, in blue, green, red, solid");
        assert!(raster.pixel.chunks(4).any(|p| p == [0, 0, 255, 255]), "the far red one shows round it");
        assert!(raster.pixel.chunks(4).any(|p| p == [0, 0, 0, 0]), "and the ground round that");
        // Made smaller, an edge is part solid, and keeps its own colour.
        let (w, h, small) = raster.smooth(2);
        assert_eq!((w, h), (32, 24));
        assert!(small.chunks(4).any(|p| p[3] > 0 && p[3] < 255 && p[2] == 255), "a red edge, part solid");
    }

    #[test]
    fn a_piece_cut_out_and_drawn_into_is_as_the_whole_drawn_into() {
        let tri = [[[-0.3, -0.3, 0.0], [0.3, -0.3, 0.0], [0.0, 0.3, 0.0]]];
        let shape = [PSolid { triangle: &tri, normal: &[], colour: [1.0, 1.0, 0.0] }];
        let mut whole = PRaster::new(64, 48, [0, 0, 0, 0]);
        whole.draw(&camera(), &shape, &PSun::default());
        let mut piece = PRaster::new(64, 48, [0, 0, 0, 0]).cut(20, 10, 24, 28);
        piece.draw(&camera(), &shape, &PSun::default());
        assert_eq!((piece.width, piece.height, piece.at), (24, 28, [20, 10]));
        assert_eq!(piece.pixel, whole.cut(20, 10, 24, 28).pixel);
    }

    #[test]
    fn a_floor_that_reaches_behind_the_eye_is_drawn_to_the_picture_s_foot() {
        // A floor under the eye, running far ahead and far behind it.
        let floor = [[[-50.0, 0.0, 50.0], [50.0, 0.0, 50.0], [0.0, 0.0, -50.0]]];
        let mut raster = PRaster::new(64, 48, [0, 0, 0, 0]);
        raster.draw(&camera(), &[PSolid { triangle: &floor, normal: &[], colour: [1.0; 3] }], &PSun::default());
        let foot = &raster.pixel[(47 * 64 + 32) * 4..(47 * 64 + 33) * 4];
        assert_eq!(foot, &[255, 255, 255, 255], "the floor right under the eye is drawn");
    }

    #[test]
    fn a_shadow_darkens_only_what_it_lies_on() {
        let floor = [[[-2.0, 0.0, -2.0], [2.0, 0.0, -2.0], [0.0, 0.0, 2.0]]];
        let mut raster = PRaster::new(64, 48, [0, 0, 0, 0]);
        raster.draw(&camera(), &[PSolid { triangle: &floor, normal: &[], colour: [1.0; 3] }], &PSun::default());
        let lying = [[[-0.3, 0.0, -0.3], [0.3, 0.0, -0.3], [0.0, 0.0, 0.3]]];
        let above = [[[-0.3, 0.5, -0.3], [0.3, 0.5, -0.3], [0.0, 0.5, 0.3]]];
        let mut shaded = raster.clone();
        shaded.shadow(&camera(), &lying, 0.5);
        assert!(shaded.pixel.chunks(4).any(|p| p[0] == 127), "half as bright under it");
        let mut missed = raster.clone();
        missed.shadow(&camera(), &above, 0.5);
        assert_eq!(missed.pixel, raster.pixel, "nothing where it lies on nothing drawn");
    }

    #[test]
    fn a_point_of_the_picture_falls_back_on_the_floor_it_came_from() {
        let (c, spot) = (camera(), [0.4, 0.0, -0.3]);
        let p = c.project(spot, 640.0, 480.0);
        let back = c.floor(p[0], p[1], 640.0, 480.0, 0.0).expect("ahead");
        assert!((0..3).all(|i| (back[i] - spot[i]).abs() < 1e-4), "{back:?}");
        assert!(c.floor(320.0, 480.0, 640.0, 480.0, 5.0).is_none(), "looking down, a floor above the eye is behind it");
        // Turned and drawn nearer, it looks at the same place from nearer.
        let turned = c.orbit(1.0, 0.2, 2.0);
        let far = |c: &PCamera| (0..3).map(|i| (c.eye[i] - c.at[i]).powi(2)).sum::<f32>().sqrt();
        assert!(turned.at == c.at && (far(&turned) * 2.0 - far(&c)).abs() < 1e-4);
    }
}
