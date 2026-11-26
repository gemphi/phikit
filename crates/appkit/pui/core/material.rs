//! What a surface is made of, as Reality Composer Pro's materials say it:
//! its colour and a second one, how rough and how much metal it is, and
//! a pattern of its relief, after the watch's bands: a woven loop, the
//! ribs of a rubber band, a braid, a mesh of steel, links, knit, leather.
//! A pattern is laid on a thing from three sides at once, as a box would
//! lay it, so a thing needs no map of its own; where the surface faces
//! between two sides the two are blended. Every material is a row of the
//! kit's table (`asset/material.tsv`); here is only how each pattern
//! runs.

use super::lens::unit;

/// How a pattern runs over a surface, in repeats: what it shows of the
/// second colour (0 to 1) and how high it stands (0 to 1) at each point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PPattern {
    Plain,
    Weave,
    Ribs,
    Braid,
    Mesh,
    Link,
    Pills,
    Knit,
    Holes,
    Brushed,
    Grain,
    Quilt,
    Channel,
    Wood,
    Speckle,
    Canvas,
}

impl PPattern {
    /// The pattern a table names.
    pub fn named(word: &str) -> PPattern {
        match word {
            "weave" => PPattern::Weave,
            "ribs" => PPattern::Ribs,
            "braid" => PPattern::Braid,
            "mesh" => PPattern::Mesh,
            "link" => PPattern::Link,
            "pills" => PPattern::Pills,
            "knit" => PPattern::Knit,
            "holes" => PPattern::Holes,
            "brushed" => PPattern::Brushed,
            "grain" => PPattern::Grain,
            "quilt" => PPattern::Quilt,
            "channel" => PPattern::Channel,
            "wood" => PPattern::Wood,
            "speckle" => PPattern::Speckle,
            "canvas" => PPattern::Canvas,
            _ => PPattern::Plain,
        }
    }

    /// The second colour's share and the height at a point, in repeats.
    fn at(self, u: f32, v: f32) -> (f32, f32) {
        let (fu, fv) = (u - u.floor(), v - v.floor());
        let cell = (u.floor(), v.floor());
        match self {
            PPattern::Plain => (0.0, 0.5),
            // Threads over and under: along one way in a cell, across in
            // the next; the second colour in threads here and there.
            PPattern::Weave => {
                let along = (cell.0 + cell.1) as i32 % 2 == 0;
                let (across, run) = match along {
                    true => (fv, fu),
                    false => (fu, fv),
                };
                let thread = (std::f32::consts::PI * across).sin();
                let fleck = (hash(cell.0 * 0.5, (v * 2.0).floor()) > 0.55) as u8 as f32;
                (fleck, thread * (0.75 + 0.25 * (std::f32::consts::PI * run).sin()))
            }
            // Rounded ribs across, a narrow groove between.
            PPattern::Ribs => (0.0, smooth(0.0, 0.25, (fv - 0.5).abs().mul_add(-2.0, 1.0)).sqrt()),
            // Strands slanting one way in a column, the other in the next.
            PPattern::Braid => {
                let way = match cell.0 as i32 % 2 == 0 {
                    true => 1.0,
                    false => -1.0,
                };
                let strand = (v * 2.0 + way * fu * 2.0).rem_euclid(1.0);
                let h = (std::f32::consts::PI * strand).sin().powf(0.6);
                (hash(cell.0, (v * 2.0 + way * fu * 2.0).floor()) * 0.35, h)
            }
            // Little rings, each row a half ring on from the last.
            PPattern::Mesh => {
                let shift = (cell.1 as i32 % 2) as f32 * 0.5;
                let x = (u + shift).rem_euclid(1.0) - 0.5;
                let d = (x * x + (fv - 0.5) * (fv - 0.5)).sqrt();
                (0.0, (1.0 - ((d - 0.32).abs() * 7.0)).clamp(0.0, 1.0))
            }
            // Links: a long block a row, its ends round, a gap between,
            // brushed along.
            PPattern::Link => {
                let edge = (fv.min(1.0 - fv) * 9.0).min(fu.min(1.0 - fu) * 2.5).clamp(0.0, 1.0);
                let brush = hash(0.0, (v * 40.0).floor() + (u * 3.0).floor()) * 0.04;
                (0.0, smooth(0.0, 1.0, edge) * 0.96 + brush)
            }
            // Soft pills lying across, one a row, as the magnetic link's.
            PPattern::Pills => {
                let x = ((fu - 0.5).abs() - 0.36).max(0.0) / 0.14;
                let y = (fv - 0.5).abs() / 0.42;
                let d = (x * x + y * y).sqrt();
                (0.0, (1.0 - d * d).max(0.0).sqrt())
            }
            // Knit: a column of stitches leaning in a V, wide stripes of
            // the second colour along.
            PPattern::Knit => {
                let lean = (fu - 0.5).abs() * 1.2;
                let stitch = (fv * 2.0 + lean).rem_euclid(1.0);
                let stripe = ((u / 6.0).rem_euclid(1.0) < 0.5) as u8 as f32;
                (stripe, (std::f32::consts::PI * stitch).sin() * (1.0 - (fu - 0.5).abs()))
            }
            // Smooth, a hole now and then pressed in along the middle.
            PPattern::Holes => {
                let x = (fu - 0.5) / 0.09;
                let y = (fv - 0.5) / 0.16;
                let d = (x * x + y * y).sqrt();
                (0.0, 0.5 + 0.5 * smooth(0.6, 1.0, d) - 0.5)
            }
            // Fine lines along, as a file leaves on metal.
            PPattern::Brushed => (0.0, 0.5 + 0.5 * (hash(0.0, (v * 80.0).floor()) - 0.5)),
            // Little pebbles of a hide, near and far apart.
            PPattern::Grain => {
                let d = cellular(u * 3.0, v * 3.0);
                (0.0, (1.0 - d * 1.4).clamp(0.0, 1.0))
            }
            // Diamonds drawn in by stitches, a button where four meet.
            PPattern::Quilt => {
                let (a, b) = ((u + v).rem_euclid(1.0), (u - v).rem_euclid(1.0));
                let seam = (a - 0.5).abs().max((b - 0.5).abs()) * 2.0;
                let button = ((a.min(1.0 - a)).powi(2) + (b.min(1.0 - b)).powi(2)).sqrt();
                let dome = (1.0 - seam * seam).max(0.0).sqrt();
                (((button < 0.08) as u8 as f32) * 0.6, dome * smooth(0.0, 0.12, button))
            }
            // Channels: long rounded rolls side by side, a seam between.
            PPattern::Channel => (0.0, (std::f32::consts::PI * fu).sin().powf(0.35)),
            // Rings of a tree's years, waving.
            PPattern::Wood => {
                let ring = (v * 7.0 + (u * 2.1).sin() * 0.6 + (u * 9.0).sin() * 0.08).rem_euclid(1.0);
                (smooth(0.35, 0.65, ring) * smooth(1.0, 0.8, ring), 0.5 + 0.1 * ring)
            }
            // Chips set in stone.
            PPattern::Speckle => {
                let d = cellular(u * 2.0, v * 2.0);
                (((d < 0.22) as u8 as f32), 0.5)
            }
            // A fine plain weave, one thread over one under.
            PPattern::Canvas => {
                let over = ((cell.0 + cell.1) as i32 % 2) as f32;
                let thread = (std::f32::consts::PI * fu).sin() * over + (std::f32::consts::PI * fv).sin() * (1.0 - over);
                (hash(cell.0, cell.1) * 0.25, thread)
            }
        }
    }
}

/// A material: its colour and its second, how rough (0 smooth, 1 chalk)
/// and how metal (0 to 1), its pattern, how many repeats of it go to a
/// metre, and how deep its relief looks.
#[derive(Debug, Clone, PartialEq)]
pub struct PMaterial {
    pub name: String,
    pub colour: [f32; 3],
    pub second: [f32; 3],
    pub rough: f32,
    pub metal: f32,
    pub pattern: PPattern,
    pub scale: f32,
    pub depth: f32,
}

/// How a point of a surface looks: its colour, the way it faces with its
/// relief, how rough and how metal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PSurface {
    pub colour: [f32; 3],
    pub normal: [f32; 3],
    pub rough: f32,
    pub metal: f32,
}

impl PMaterial {
    /// The materials of a table: a row each, its name, its pattern, its
    /// colour and its second (#rrggbb, as seen), how rough, how metal,
    /// repeats a metre and how deep. `#` and blank lines are notes.
    pub fn table(text: &str) -> Vec<PMaterial> {
        text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()).filter_map(|line| {
            let cell: Vec<&str> = line.split('\t').collect();
            let number = |i: usize| cell.get(i).and_then(|c| c.parse::<f32>().ok());
            Some(PMaterial {
                name: cell.first()?.to_string(),
                pattern: PPattern::named(cell.get(1)?),
                colour: seen(cell.get(2)?)?,
                second: seen(cell.get(3)?)?,
                rough: number(4)?,
                metal: number(5)?,
                scale: number(6)?,
                depth: number(7)?,
            })
        }).collect()
    }

    /// How the surface looks at a point in space, facing `normal`, in
    /// a colour of its own where one is given (the person's paint): the
    /// pattern laid from the three sides, each as much as the surface
    /// faces it.
    pub fn surface(&self, colour: [f32; 3], point: [f32; 3], normal: [f32; 3]) -> PSurface {
        let n = unit(normal);
        let weight = n.map(|v| v.abs().powi(4));
        let all = (weight[0] + weight[1] + weight[2]).max(f32::EPSILON);
        let s = self.scale;
        // Each side's own two ways across, and the ways they run in space.
        let side = [([point[2], point[1]], [0usize, 2, 1]), ([point[0], point[2]], [1, 0, 2]), ([point[0], point[1]], [2, 0, 1])];
        let (mut mix, mut tilt) = (0.0, [0.0f32; 3]);
        for (k, (uv, axis)) in side.iter().enumerate() {
            let w = weight[k] / all;
            if w < 0.01 {
                continue;
            }
            let (u, v) = (uv[0] * s, uv[1] * s);
            let e = 0.02;
            let (m, h) = self.pattern.at(u, v);
            let du = (self.pattern.at(u + e, v).1 - h) / e;
            let dv = (self.pattern.at(u, v + e).1 - h) / e;
            mix += m * w;
            // The relief leans the surface against its slope.
            tilt[axis[1]] -= du * self.depth * w;
            tilt[axis[2]] -= dv * self.depth * w;
        }
        let colour = [0, 1, 2].map(|i| colour[i] + (self.second[i] - colour[i]) * mix);
        let normal = unit([0, 1, 2].map(|i| n[i] + tilt[i] * 0.08));
        PSurface { colour, normal, rough: self.rough, metal: self.metal }
    }

    /// A ball of it, so many steps round: what a material's tile shows.
    pub fn ball(step: usize) -> (Vec<[[f32; 3]; 3]>, Vec<[[f32; 3]; 3]>) {
        let at = |i: usize, j: usize| {
            let (a, b) = (std::f32::consts::PI * i as f32 / step as f32, std::f32::consts::TAU * j as f32 / step as f32);
            [a.sin() * b.cos() * 0.5, a.cos() * 0.5, a.sin() * b.sin() * 0.5]
        };
        let (mut triangle, mut normal) = (Vec::new(), Vec::new());
        for i in 0..step {
            for j in 0..step {
                let (p, q, r, s) = (at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j + 1));
                for t in [[p, q, r], [p, r, s]] {
                    normal.push(t.map(|c| c.map(|v| v * 2.0)));
                    triangle.push(t);
                }
            }
        }
        (triangle, normal)
    }
}

/// A colour as a table writes it (#rrggbb, as seen), 0 to 1, as the
/// raster lights colours.
fn seen(hex: &str) -> Option<[f32; 3]> {
    let hex = hex.strip_prefix('#')?;
    let byte = |i: usize| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok();
    Some([byte(0)?, byte(2)?, byte(4)?].map(|c| c as f32 / 255.0))
}

/// A number from 0 to 1 for a cell, the same each time.
fn hash(x: f32, y: f32) -> f32 {
    let h = (x * 127.1 + y * 311.7).sin() * 43_758.547;
    h - h.floor()
}

/// How far a point is from the nearest of points scattered one a cell.
fn cellular(u: f32, v: f32) -> f32 {
    let (cx, cy) = (u.floor(), v.floor());
    let mut near = f32::MAX;
    for dx in -1..=1 {
        for dy in -1..=1 {
            let (x, y) = (cx + dx as f32, cy + dy as f32);
            let (px, py) = (x + hash(x, y), y + hash(y, x));
            near = near.min(((px - u).powi(2) + (py - v).powi(2)).sqrt());
        }
    }
    near
}

/// 0 below `low`, 1 above `high`, smoothly between.
fn smooth(low: f32, high: f32, x: f32) -> f32 {
    let t = ((x - low) / (high - low)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
