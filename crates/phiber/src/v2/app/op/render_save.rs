//! The 'save finished shot' half of `op::render` — `save_view` for a
//! turnaround view's file, the FFmpeg `movie_job` + `movie_made` pair,
//! and `save_frame` for one still image. Split from `render.rs` so each
//! file sits under the 240-line read cap (`tests/level_structure.rs`);
//! the start-a-render path (`render_now` → `render_from` → `render_look`)
//! stays with its own render-related state.

use super::super::BPhiberApp;
use crate::v2::state::{BKey as _, log};
use crate::v2::part::text::info::{self, BReportLevel};
use crate::v2::state::CtxKey;
use crate::v2::scene::render_job;

impl BPhiberApp {
/// Write the finished shot as turnaround view `name`'s file.
pub(super) fn save_view(&mut self, name: &str) -> bool {
    let (shot, _) = render_job::shot();
    let Some(shot) = shot else { return false };
    let output = crate::v2::scene::output_set::as_still(&crate::v2::scene::output_set(&self.state));
    let pass = crate::v2::scene::pass_set(&self.state, Default::default());
    let blend = self.state.get(&crate::v2::scene::loaded_path_key()).unwrap_or_default().to_string();
    let folder = std::path::Path::new(&blend).parent().map(std::path::Path::to_path_buf).unwrap_or_default();
    let file = crate::v2::scene::render_save::view_file(&output, &folder, name);
    match crate::v2::scene::save_render_to(&shot, &output, pass, &file) {
        Ok(()) => {
            info::push(&mut self.state, BReportLevel::Info, format!("Saved: '{}'", file.display()));
            true
        }
        Err(why) => {
            info::push(&mut self.state, BReportLevel::Error, format!("Cannot save the {name} view: {why}"));
            false
        }
    }
}

/// The movie an FFmpeg Video output gathers its frames into, when that
/// is the format asked for.
pub(super) fn movie_job(&self, start: i32, end: i32) -> Option<crate::v2::scene::video::BMovieJob> {
    let output = crate::v2::scene::output_set(&self.state);
    if !crate::v2::scene::output_set::wants_movie(&output) {
        return None;
    }
    let blend = self.state.get(&crate::v2::scene::loaded_path_key()).unwrap_or_default().to_string();
    let folder = std::path::Path::new(&blend).parent().map(std::path::Path::to_path_buf).unwrap_or_default();
    let still = crate::v2::scene::output_set::as_still(&output).file_of(&folder);
    let movie = crate::v2::scene::output_set::movie_set(&self.state, self.scene_cache.as_ref().map(|(_, m)| m.render.movie).unwrap_or_default());
    let fps = crate::v2::state::key::float_or(&self.state, CtxKey::Fps, 24.0);
    match crate::v2::scene::video::BMovieJob::begin(&still, &output, movie, fps, start, end) {
        Ok(job) => Some(job),
        Err(why) => {
            log::push(format!("Cannot gather the movie's frames: {why}"));
            None
        }
    }
}

/// What the encode came to, told in the Info editor.
pub(super) fn movie_made(&mut self, made: Result<crate::v2::scene::video::BMovieMade, String>) {
    use crate::v2::scene::video::BMovieMade;
    match made {
        Ok(BMovieMade::Ffmpeg(file)) => info::push(&mut self.state, BReportLevel::Info, format!("Saved movie: '{}'", file.display())),
        Ok(BMovieMade::Avi(file)) => info::push(
            &mut self.state,
            BReportLevel::Warning,
            format!("ffmpeg is not installed: saved a Motion-JPEG AVI instead, '{}' (install ffmpeg for the H.264 / container the Output panel asks for)", file.display()),
        ),
        Err(why) => info::push(&mut self.state, BReportLevel::Error, format!("Cannot write the movie: {why}")),
    }
}

/// The finished shot written as frame `frame` of the output path — or,
/// `into` a movie's frame file, as a PNG on its way to the encoder.
pub(super) fn save_frame(&mut self, frame: i32, into: Option<std::path::PathBuf>) -> bool {
    let (shot, _) = render_job::shot();
    let Some(shot) = shot else { return false };
    let output = crate::v2::scene::output_set::as_still(&crate::v2::scene::output_set(&self.state));
    let pass = crate::v2::scene::pass_set(&self.state, Default::default());
    let blend = self.state.get(&crate::v2::scene::loaded_path_key()).unwrap_or_default().to_string();
    let folder = std::path::Path::new(&blend).parent().map(std::path::Path::to_path_buf).unwrap_or_default();
    let movie = into.is_some();
    let file = into.unwrap_or_else(|| crate::v2::scene::render_save::frame_file(&output, &folder, frame));
    match crate::v2::scene::save_render_to(&shot, &output, pass, &file) {
        Ok(()) => {
            if !movie {
                info::push(&mut self.state, BReportLevel::Info, format!("Saved: '{}'", file.display()));
            }
            true
        }
        Err(why) => {
            info::push(&mut self.state, BReportLevel::Error, format!("Cannot save frame {frame}: {why}"));
            false
        }
    }
}}
