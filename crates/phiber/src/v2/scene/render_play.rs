//! View Animation — `render.play_rendered_anim`
//! (`screen_play_rendered_anim.py`): the frames an animation render wrote
//! where Output points, played back at the scene's frame rate. Blender
//! hands them to a player — its own by default; Phiber's player is the
//! render window, which shows each frame in turn in place of the Render
//! Result and puts the result back when it stops. A movie output goes to
//! the system's player, as Blender's external-player presets do. The
//! operator looks for the first frame's file and warns, as Blender does,
//! when it is not there: "File '…' not found".

use super::render_job::BRenderShot;
use crate::phiphx::blend::BlendOutput;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// Whether the player is running — one at a time, as one window plays.
static PLAYING: AtomicBool = AtomicBool::new(false);

/// What Ctrl F11 plays.
#[derive(Clone, Debug, PartialEq)]
pub enum BRenderedPlayback {
    /// The frames `start ..= end` as files, at `fps`.
    Frames { files: Vec<PathBuf>, fps: f32 },
    /// One movie file (Output ▸ File Format: a video).
    Movie(PathBuf),
}

impl BRenderedPlayback {
    /// What the Output settings would have written for `start ..= end`,
    /// relative to the `.blend`'s folder; `movie` is the file a video
    /// output wrote. Errs with Blender's warning when the first frame (or
    /// the movie) is not on disk.
    pub fn plan(output: &BlendOutput, folder: &Path, movie: Option<PathBuf>, start: i32, end: i32, fps: f32) -> Result<Self, String> {
        let first = match &movie {
            Some(file) => file.clone(),
            None => super::render_save::frame_file(output, folder, start),
        };
        if !first.is_file() {
            return Err(format!("File '{}' not found", first.display()));
        }
        Ok(match movie {
            Some(file) => Self::Movie(file),
            None => Self::Frames { files: (start..=end.max(start)).map(|f| super::render_save::frame_file(output, folder, f)).collect(), fps: fps.max(1e-3) },
        })
    }

    /// A frame's file as the render window shows a shot: BGRA rows, done.
    pub fn load(file: &Path) -> Option<BRenderShot> {
        let img = image::open(file).ok()?.to_rgba8();
        let (w, h) = img.dimensions();
        let mut bgra = img.into_raw();
        for px in bgra.chunks_exact_mut(4) {
            px.swap(0, 2);
        }
        Some(BRenderShot { w, h, bgra, taken: 1, target: 1, done: true, ..Default::default() })
    }

    /// The pause between frames.
    pub fn frame_time(&self) -> std::time::Duration {
        match self {
            Self::Frames { fps, .. } => std::time::Duration::from_secs_f32(1.0 / fps.max(1e-3)),
            Self::Movie(_) => std::time::Duration::ZERO,
        }
    }

    /// Take the player: `false` when one is already running.
    pub fn begin() -> bool {
        !PLAYING.swap(true, Ordering::SeqCst)
    }

    pub fn end() {
        PLAYING.store(false, Ordering::SeqCst);
    }

    pub fn playing() -> bool {
        PLAYING.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_plan_names_the_frames_and_wants_the_first_on_disk() {
        let dir = std::env::temp_dir().join(format!("phiber-play-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let output = BlendOutput { path: dir.join("shot_####").to_string_lossy().to_string(), ..Default::default() };
        let missing = BRenderedPlayback::plan(&output, &dir, None, 3, 5, 24.0).unwrap_err();
        assert!(missing.starts_with("File '") && missing.ends_with("shot_0003.png' not found"), "{missing}");
        let img = image::RgbaImage::from_fn(4, 2, |x, _| image::Rgba([x as u8 * 60, 10, 200, 255]));
        img.save(dir.join("shot_0003.png")).unwrap();
        let plan = BRenderedPlayback::plan(&output, &dir, None, 3, 5, 24.0).unwrap();
        let BRenderedPlayback::Frames { files, fps } = &plan else { panic!("frames") };
        assert_eq!(files.len(), 3);
        assert!(files[2].ends_with("shot_0005.png") && *fps == 24.0);
        assert!((plan.frame_time().as_secs_f32() - 1.0 / 24.0).abs() < 1e-6);
        let shot = BRenderedPlayback::load(&files[0]).expect("a frame");
        assert_eq!((shot.w, shot.h, shot.done), (4, 2, true));
        // BGRA: the red 60 of pixel 1 lands third.
        assert_eq!(&shot.bgra[4..8], &[200, 10, 60, 255]);
        assert!(BRenderedPlayback::load(&files[1]).is_none(), "a frame not written is skipped");
        let movie = dir.join("shot.mp4");
        std::fs::write(&movie, b"").unwrap();
        assert_eq!(BRenderedPlayback::plan(&output, &dir, Some(movie.clone()), 1, 2, 24.0).unwrap(), BRenderedPlayback::Movie(movie));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
