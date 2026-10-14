//! Render a range of an animation the way Ctrl F12 does — the scene posed at
//! each frame, through the scene's own camera, at the file's resolution —
//! and write one PNG per frame beside a `times.txt` of per-frame timings.
//! The Blender side of the same comparison is
//! `phiphx/tests/truth/blender_animation.py`; `render_compare` reads a pair
//! of directories.
//!
//!   cargo run --release -p phiber --example animation_render -- FILE OUTDIR [start] [end] [step] [samples]
//!
//! The loop itself is `v2::cli::render::BHeadlessRender`, which the
//! `phiber -b … -a` command line runs too — one renderer, two front doors.
//! The env switches below are the measurement aids it reads from
//! `BLookOverride::from_env`:
//!
//!   PHIBER_SCALE=<n>    divides the resolution, for a quick pass
//!   PHIBER_NO_WORLD=1   the world contributes nothing (lamps only)
//!   PHIBER_NO_LIGHTS=1  every lamp is dropped (the world only)
//!   PHIBER_VIEW=…       Standard | Filmic | AgX, over the file's
//!   PHIBER_OVERRIDE=<a> one grey Lambertian surface in place of every
//!                       material, so the render measures the light
//!   PHIRAY_DENOISE=0    the filter off, which is what a measurement wants
//!
//! Samples default to the quality floor (`BRenderQuality::SAMPLE_FLOOR`),
//! so a run that names none is the good render; pass a count to override.
use phiber::phiphx::Phx;
use phiber::v2::cli::render::{BHeadlessRender, BLookOverride};

fn main() {
    let mut args = std::env::args().skip(1);
    let (path, dir) = (args.next().expect("file"), args.next().expect("outdir"));
    let mut num = |d: f32| -> f32 { args.next().and_then(|s| s.parse().ok()).unwrap_or(d) };
    let (start, end, step) = (num(1.0), num(0.0), num(1.0));
    drop(num);
    let samples: Option<u32> = args.next().and_then(|s| s.parse().ok());

    // The file's own end is what an unset `end` means, so the range needs
    // the timeline before the render does.
    let file_end = Phx::open(std::path::Path::new(&path)).expect("open").timeline.frame_end as f32;
    let render = BHeadlessRender {
        file: path,
        out: dir,
        frame: BHeadlessRender::range(start, end, step, file_end),
        samples,
        draft: samples.is_some(),
        look: BLookOverride::from_env(),
    };
    let first = render.frame.first().copied().unwrap_or(start);
    let last = render.frame.last().copied().unwrap_or(start);
    println!("frames {first}..={last} step {step}, {} samples", render.sample_count());

    match render.run() {
        Ok(cost) => {
            for line in cost.iter().map(|c| c.line()) {
                print!("{line}");
            }
            println!("-> {}", render.out);
        }
        Err(why) => {
            eprintln!("Error: {why}");
            std::process::exit(1);
        }
    }
}
