//! Writes a choice's arena as a flat binary file for `tools/preview.py`,
//! which builds it in Blender and renders it — how the generators are
//! looked at before Phiber draws them.
//!
//! `cargo run -p phiplay --example preview -- <venue> <sport> <theme> <out>`,
//! the first three by their index in their set's `ALL`.

use phiplay::build::{BPart, ball, board, field, goal, venue};
use phiplay::{BChoice, sport::BSport, theme::BTheme, venue::BVenue};
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let pick = |i: usize| args.get(i).and_then(|a| a.parse::<usize>().ok()).unwrap_or(0);
    let choice = BChoice { venue: BVenue::ALL[pick(1)], sport: BSport::ALL[pick(2)], theme: BTheme::ALL[pick(3)], ..BChoice::default() };
    let out = args.get(4).cloned().unwrap_or_else(|| "preview.bin".into());
    let sport = choice.sport.spec();
    let mut parts: Vec<BPart> = vec![field::floor(sport, choice.turf().spec(), choice.theme.spec())];
    parts.extend(goal::goals(sport, choice.theme.spec()));
    let mut ball = ball::ball(&sport.ball, "Ball");
    ball.at = glam::Mat4::from_translation(glam::Vec3::new(0.0, 0.0, 1.5));
    parts.push(ball);
    // The board as it reads at 23–7 after 1:05.
    let board = board::board(&choice);
    parts.extend(board.parts);
    parts.extend(board.digits.into_iter().filter(|d| d.slot.shows([23, 7], 65 * 24) == Some(d.value)).map(|d| d.part));
    let built = venue::venue(&choice);
    parts.extend(built.parts);
    // Seats and fans merged per list: the preview does not need instances.
    for s in built.seats.iter().chain(&built.crowd) {
        let mut m = phiplay::build::BMesh::default();
        for at in &s.at {
            m.merge(&s.mesh, *at);
        }
        parts.push(BPart { name: s.name.clone(), mesh: std::sync::Arc::new(m), material: s.material.clone(), at: glam::Mat4::IDENTITY });
    }
    let mut f = std::io::BufWriter::new(std::fs::File::create(&out).expect("out"));
    let u32le = |f: &mut std::io::BufWriter<std::fs::File>, v: u32| f.write_all(&v.to_le_bytes()).unwrap();
    let f32le = |f: &mut std::io::BufWriter<std::fs::File>, v: f32| f.write_all(&v.to_le_bytes()).unwrap();
    u32le(&mut f, parts.len() as u32);
    for p in &parts {
        let name = p.name.as_bytes();
        u32le(&mut f, name.len() as u32);
        f.write_all(name).unwrap();
        let m = &p.mesh;
        u32le(&mut f, m.positions.len() as u32);
        for (v, uv) in m.positions.iter().zip(&m.uvs) {
            let w = p.at.transform_point3(glam::Vec3::from(*v));
            for x in [w.x, w.y, w.z, uv[0], uv[1]] {
                f32le(&mut f, x);
            }
        }
        u32le(&mut f, m.indices.len() as u32);
        for i in &m.indices {
            u32le(&mut f, *i);
        }
        let mat = &p.material;
        for x in [mat.base.0, mat.base.1, mat.base.2, mat.roughness, mat.metallic, mat.emission.0, mat.emission.1, mat.emission.2, mat.strength] {
            f32le(&mut f, x);
        }
        match &mat.image {
            Some(img) => {
                u32le(&mut f, img.width);
                u32le(&mut f, img.height);
                f.write_all(&img.rgba).unwrap();
            }
            None => {
                u32le(&mut f, 0);
                u32le(&mut f, 0);
            }
        }
    }
    eprintln!("{} parts, {} seats -> {out}", parts.len(), built.places.len());
}
