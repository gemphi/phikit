//! Writes every picker card as one contact sheet (raw RGBA rows, a row of
//! cards per picker row) for a look: `cargo run -p phiplay --example cards -- <out.ppm>`.
use phiplay::pick::{BPick, thumb};
use std::io::Write;

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "cards.ppm".into());
    let choice = phiplay::BChoice::default();
    let n = thumb::SIZE as usize;
    let cols = 16;
    let rows = BPick::ALL.len();
    let (w, h) = (cols * n, rows * n);
    let mut px = vec![0u8; w * h * 3];
    for (r, pick) in BPick::ALL.iter().enumerate() {
        for i in 0..pick.count() {
            let img = thumb::thumb(*pick, i, &choice);
            for y in 0..n {
                for x in 0..n {
                    let s = (y * n + x) * 4;
                    let d = ((r * n + y) * w + i * n + x) * 3;
                    px[d..d + 3].copy_from_slice(&img.rgba[s..s + 3]);
                }
            }
        }
    }
    let mut f = std::fs::File::create(&out).unwrap();
    write!(f, "P6\n{w} {h}\n255\n").unwrap();
    f.write_all(&px).unwrap();
}
