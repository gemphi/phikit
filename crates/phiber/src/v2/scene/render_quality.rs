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
//! - **2048 samples is the floor** for a final render, so the default is
//!   the quality render rather than whatever count a file happened to be
//!   saved with. Pose 26 asks for twenty; twenty is a turntable preview,
//!   not a frame anyone would keep.
//!
//! The floor is a *final render's*. The viewport keeps `preview_samples`
//! and stays as quick as it was (`BRenderSet::preview_samples`).

use super::render_engine::BRenderEngine;

pub struct BRenderQuality;

impl BRenderQuality {
    /// The engine a render uses unless something names another — phiber's
    /// own tracer, named as itself rather than as Cycles.
    pub const ENGINE: &'static str = "PHIRAY";

    /// The sample floor. A render takes at least this many samples;
    /// `--draft` on the command line lifts it.
    pub const SAMPLE_FLOOR: u32 = 2048;

    /// The samples a render actually takes: what was asked for, never
    /// below the floor unless the floor was lifted. `None` asked for
    /// means the floor itself.
    pub fn samples(asked: Option<u32>, draft: bool) -> u32 {
        match (asked, draft) {
            (Some(asked), true) => asked.max(1),
            (Some(asked), false) => asked.max(Self::SAMPLE_FLOOR),
            (None, _) => Self::SAMPLE_FLOOR,
        }
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
    fn the_floor_holds_unless_draft_lifts_it() {
        assert_eq!(BRenderQuality::samples(None, false), 2048, "nothing asked: the floor");
        assert_eq!(BRenderQuality::samples(Some(20), false), 2048, "pose 26's twenty is lifted to the floor");
        assert_eq!(BRenderQuality::samples(Some(4096), false), 4096, "more than the floor is kept");
        assert_eq!(BRenderQuality::samples(Some(20), true), 20, "--draft renders what was asked");
        assert_eq!(BRenderQuality::samples(Some(0), true), 1, "a render takes at least one sample");
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
