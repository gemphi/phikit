//! The quality policy: what a render defaults to when nothing else says.
//!
//! One owner for two rules that used to live only on the command line
//! (`v2::cli::set`), while the app's own F12 took neither of them:
//!
//! - **Phiray is the engine.** A `.blend` saved by Blender names EEVEE or
//!   Cycles and neither is ours to run, so the file's engine is a
//!   *preference* and not an instruction. `BLENDER_WORKBENCH` is the
//!   exception, because Workbench is an engine phiber really does have
//!   (`render_engine.rs`, `workbench_render.cc`) — a file saved under it
//!   still rasterizes.
//! - **2048 samples is the DEFAULT** for a final render — what it takes
//!   when nothing says otherwise. It was a *floor* until 2026-10-19, and
//!   a floor was the wrong shape: `asked.max(2048)` meant Max Samples
//!   could only ever be raised, so typing 64 into the panel rendered
//!   2048 and the row silently did nothing. Blender has no such rule —
//!   Max Samples means Max Samples — and a person who asks for 64 wants
//!   64, whether for a quick look or because they are denoising.
//!
//!   The original reasoning still holds for the *default*: pose 26 asks
//!   for twenty, and twenty is a turntable preview rather than a frame
//!   anyone would keep. So a file that never says gets 2048; a file or a
//!   panel that does say is believed.
//!
//! The floor is a *final render's*. The viewport keeps `preview_samples`
//! and stays as quick as it was (`BRenderSet::preview_samples`).

use super::render_engine::BRenderEngine;

pub struct BRenderQuality;

impl BRenderQuality {
    /// The engine a render uses unless something names another — phiber's
    /// own tracer, named as itself rather than as Cycles.
    pub const ENGINE: &'static str = "PHIRAY";

    /// The samples a render takes when nothing asks for a count.
    ///
    /// Still named `SAMPLE_FLOOR` because that is what it was and what
    /// the command line calls it; it is a default now, not a minimum.
    pub const SAMPLE_FLOOR: u32 = 2048;

    /// The samples a render actually takes: **what was asked for**, and
    /// the default only when nothing asked. A render takes at least one.
    ///
    /// `draft` no longer changes this. It existed to lift a floor that no
    /// longer exists, and is kept as an argument so the command line does
    /// not have to change.
    pub fn samples(asked: Option<u32>, _draft: bool) -> u32 {
        asked.map_or(Self::SAMPLE_FLOOR, |asked| asked.max(1))
    }

    /// The engine to render a file with when the Engine row has never been
    /// touched. Workbench is honoured because phiber has it; every other
    /// name a `.blend` can carry resolves to the tracer.
    pub fn engine_for_file(named: &str) -> BRenderEngine {
        match BRenderEngine::of(named) {
            BRenderEngine::Workbench => BRenderEngine::Workbench,
            _ => BRenderEngine::Phiray,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_count_that_was_asked_for_is_the_count_that_renders() {
        assert_eq!(BRenderQuality::samples(None, false), 2048, "nothing asked: the default");
        // The row is believed. This used to assert the opposite — that
        // twenty was "lifted to the floor" — which is exactly why typing
        // 64 into Max Samples rendered 2048 and looked like a dead
        // control.
        assert_eq!(BRenderQuality::samples(Some(20), false), 20, "pose 26's twenty is honoured");
        assert_eq!(BRenderQuality::samples(Some(64), false), 64, "a 64-sample render takes 64");
        assert_eq!(BRenderQuality::samples(Some(4096), false), 4096, "and a big count is kept");
        assert_eq!(BRenderQuality::samples(Some(0), false), 1, "a render takes at least one sample");
        assert_eq!(BRenderQuality::samples(Some(64), true), 64, "draft changes nothing now");
    }

    #[test]
    fn a_files_engine_resolves_to_the_one_we_have() {
        // The two Blender engines phiber cannot run are rendered by the
        // tracer, and say so.
        for named in ["CYCLES", "BLENDER_EEVEE_NEXT", "BLENDER_EEVEE", "", "SOME_ADDON"] {
            assert_eq!(BRenderQuality::engine_for_file(named), BRenderEngine::Phiray, "{named}");
        }
        // Workbench is ours, so it stays itself and keeps rasterizing.
        assert_eq!(BRenderQuality::engine_for_file("BLENDER_WORKBENCH"), BRenderEngine::Workbench);
        assert!(!BRenderQuality::engine_for_file("BLENDER_WORKBENCH").traces());
        assert!(BRenderQuality::engine_for_file("CYCLES").traces());
    }
}
