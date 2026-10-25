//! Dresses the players' figure eight times over — eight uniforms, eight
//! shoes, eight of the hairstyles, eight teams — and writes them in a row
//! for `tools/preview.py`, from a body dumped by `tools/body_dump.py`.
//!
//! `cargo run -p phiplay --example figure -- <body.bin> <sport> <page> <out>`:
//! `page` 0 shows hairstyles 1–8, 1 shows 9–16.

use phiplay::build::body::{BBody, BBone, BPiece};
use phiplay::build::{hair, kit, shoe};
use phiplay::colour::Rgb;
use phiplay::kit::sneaker::BSneaker;
use phiplay::kit::uniform::BUniform;
use phiplay::look::BLook;
use phiplay::look::hair::BHair;
use phiplay::sport::BSport;
use phiplay::team::BTeam;
use std::io::Write;

fn read(path: &str) -> BBody {
    let data = std::fs::read(path).expect("body dump");
    let mut at = 0usize;
    let mut b = BBody::default();
    let take_u32 = |at: &mut usize| {
        let v = u32::from_le_bytes(data[*at..*at + 4].try_into().unwrap());
        *at += 4;
        v
    };
    let take_f32 = |at: &mut usize| {
        let v = f32::from_le_bytes(data[*at..*at + 4].try_into().unwrap());
        *at += 4;
        v
    };
    let take_text = |at: &mut usize| {
        let n = take_u32(at) as usize;
        let s = String::from_utf8(data[*at..*at + n].to_vec()).unwrap();
        *at += n;
        s
    };
    let nv = take_u32(&mut at);
    for _ in 0..nv {
        b.positions.push([take_f32(&mut at), take_f32(&mut at), take_f32(&mut at)]);
    }
    let nf = take_u32(&mut at);
    for _ in 0..nf {
        let n = take_u32(&mut at);
        b.faces.push((0..n).map(|_| take_u32(&mut at)).collect());
    }
    let ng = take_u32(&mut at);
    for _ in 0..ng {
        b.groups.push(take_text(&mut at));
    }
    for _ in 0..nv {
        let n = take_u32(&mut at);
        b.weights.push((0..n).map(|_| (take_u32(&mut at), take_f32(&mut at))).collect());
    }
    let nb = take_u32(&mut at);
    for _ in 0..nb {
        let name = take_text(&mut at);
        b.bones.push(BBone { name, head: [take_f32(&mut at), take_f32(&mut at), take_f32(&mut at)], tail: [take_f32(&mut at), take_f32(&mut at), take_f32(&mut at)] });
    }
    b
}

/// One part of the preview dump.
struct Out {
    name: String,
    verts: Vec<[f32; 5]>,
    tris: Vec<u32>,
    mat: phiplay::build::BMat,
}

fn split(name: &str, piece: &BPiece, mats: &[phiplay::build::BMat], dx: f32) -> Vec<Out> {
    mats.iter()
        .enumerate()
        .map(|(slot, mat)| {
            let mut tris = Vec::new();
            for (f, m) in piece.faces.iter().zip(&piece.face_materials) {
                if *m as usize == slot {
                    for k in 1..f.len() - 1 {
                        tris.extend([f[0], f[k], f[k + 1]]);
                    }
                }
            }
            let verts = piece.positions.iter().zip(&piece.uvs).map(|(p, uv)| [p[0] + dx, p[1], p[2], uv[0], uv[1]]).collect();
            Out { name: format!("{name} {slot}"), verts, tris, mat: mat.clone() }
        })
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let body = read(&args[1]);
    let sport = BSport::ALL[args[2].parse::<usize>().unwrap()].spec();
    let page: usize = args[3].parse().unwrap();
    let mut parts = Vec::new();
    for i in 0..8 {
        let dx = (i as f32 - 3.5) * 0.75;
        let team = BTeam::ALL[i * 2 + page].spec();
        let look = BLook::ALL[i].spec();
        let skin = phiplay::build::BMat::plain("Skin", look.skin, 0.45);
        let whole = BPiece { positions: body.positions.clone(), faces: body.faces.clone(), uvs: vec![[0.0; 2]; body.positions.len()], weights: Vec::new(), face_materials: vec![0; body.faces.len()] };
        parts.extend(split(&format!("Body {i}"), &whole, &[skin], dx));
        for g in kit::kit(&body, sport.cut, BUniform::ALL[i].spec(), team.home, team.trim, [7, 11, 23, 3, 14, 30, 5, 9][i]) {
            parts.extend(split(&format!("{} {i}", g.name), &g.piece, &g.materials, dx));
        }
        if let Some(g) = shoe::shoes(&body, sport.footwear, BSneaker::ALL[i].spec(), team.home, team.trim) {
            parts.extend(split(&format!("Shoes {i}"), &g.piece, &g.materials, dx));
        }
        let style = BHair::ALL[i + page * 8].spec();
        if let Some(g) = hair::hair(&body, style, look.hair[i % 3]) {
            parts.extend(split(&format!("Hair {i}"), &g.piece, &g.materials, dx));
        }
    }
    let mut f = std::io::BufWriter::new(std::fs::File::create(&args[4]).unwrap());
    let w32 = |f: &mut std::io::BufWriter<std::fs::File>, b: [u8; 4]| f.write_all(&b).unwrap();
    w32(&mut f, (parts.len() as u32).to_le_bytes());
    for p in &parts {
        w32(&mut f, (p.name.len() as u32).to_le_bytes());
        f.write_all(p.name.as_bytes()).unwrap();
        w32(&mut f, (p.verts.len() as u32).to_le_bytes());
        for v in &p.verts {
            for x in v {
                w32(&mut f, x.to_le_bytes());
            }
        }
        w32(&mut f, (p.tris.len() as u32).to_le_bytes());
        for i in &p.tris {
            w32(&mut f, i.to_le_bytes());
        }
        let m = &p.mat;
        let Rgb(r, g, b) = m.base;
        let Rgb(er, eg, eb) = m.emission;
        for x in [r, g, b, m.roughness, m.metallic, er, eg, eb, m.strength] {
            w32(&mut f, x.to_le_bytes());
        }
        match &m.image {
            Some(img) => {
                w32(&mut f, img.width.to_le_bytes());
                w32(&mut f, img.height.to_le_bytes());
                f.write_all(&img.rgba).unwrap();
            }
            None => {
                w32(&mut f, 0u32.to_le_bytes());
                w32(&mut f, 0u32.to_le_bytes());
            }
        }
    }
    eprintln!("{} parts", parts.len());
}
