//! A movie from a rendered animation — Output → File Format: FFmpeg Video.
//!
//! Blender links libavcodec and encodes as the frames land
//! (`imbuf/movie/intern/movie_write.cc`); Phiber writes each frame as it
//! lands and encodes once the range is done, with the same settings —
//! the file's container, codec, constant rate factor, preset and keyframe
//! interval (`phiphx::BlendMovie`) — through the `ffmpeg` program when
//! the machine has one. Without it, a Motion-JPEG AVI (`avi.rs`), which
//! every player opens, so an animation always comes out as a movie.

mod avi;

use crate::phiphx::blend::{BlendMovie, BlendOutput};
use std::path::{Path, PathBuf};
use std::process::Command;

/// A movie being gathered, frame by frame.
pub struct BMovieJob {
    /// Where the frames land until they are encoded.
    pub frames_dir: PathBuf,
    pub frames: Vec<PathBuf>,
    pub movie: BlendMovie,
    pub fps: f32,
    /// The movie file (`BlendMovie::file_of`).
    pub file: PathBuf,
    pub quality: u8,
}

/// What came of the encode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BMovieMade {
    /// Written by FFmpeg as the settings asked.
    Ffmpeg(PathBuf),
    /// FFmpeg was not found: a Motion-JPEG AVI instead, at this path.
    Avi(PathBuf),
}

impl BMovieJob {
    /// Begin a movie whose stills would land at `still` (the Output path
    /// resolved), for frames `start..=end`.
    pub fn begin(still: &Path, output: &BlendOutput, movie: BlendMovie, fps: f32, start: i32, end: i32) -> std::io::Result<Self> {
        let file = movie.file_of(still, start, end);
        let frames_dir = std::env::temp_dir().join(format!("phiber-movie-{}", std::process::id()));
        std::fs::create_dir_all(&frames_dir)?;
        Ok(Self { frames_dir, frames: Vec::new(), movie, fps, file, quality: output.quality })
    }

    /// Where the next frame is kept until the encode (a PNG, the lossless
    /// hand-off Blender's pipeline makes in memory), numbered in sequence
    /// as the encoder reads them.
    pub fn next_frame_file(&mut self) -> PathBuf {
        let path = self.frames_dir.join(format!("frame{:04}.png", self.frames.len() + 1));
        self.frames.push(path.clone());
        path
    }

    /// Whether an `ffmpeg` program answers on this machine.
    pub fn ffmpeg_present() -> bool {
        Command::new("ffmpeg").arg("-version").output().is_ok_and(|o| o.status.success())
    }

    /// The command line `movie_write.cc` amounts to for these settings.
    pub fn ffmpeg_args(&self) -> Vec<String> {
        let mut args: Vec<String> = vec!["-y".into(), "-framerate".into(), format!("{}", self.fps), "-i".into(), self.frames_dir.join("frame%04d.png").to_string_lossy().to_string()];
        args.extend(["-c:v".to_string(), self.movie.codec_name().to_string()]);
        match self.movie.crf_value() {
            Some(crf) => args.extend(["-crf".to_string(), crf.to_string()]),
            None => args.extend(["-b:v".to_string(), format!("{}k", self.movie.bitrate.max(1))]),
        }
        if let Some((preset, deadline)) = self.movie.preset_name() {
            if self.movie.codec == 167 {
                args.extend(["-deadline".to_string(), deadline.to_string()]);
            } else if matches!(self.movie.codec, BlendMovie::H264 | 173) {
                args.extend(["-preset".to_string(), preset.to_string()]);
            }
        }
        if self.movie.gop > 0 {
            args.extend(["-g".to_string(), self.movie.gop.to_string()]);
        }
        // Players want 4:2:0 for H.264 / H.265; a PNG or FFV1 stream keeps its pixels.
        if matches!(self.movie.codec, BlendMovie::H264 | 173 | 226 | 167 | 12) {
            args.extend(["-pix_fmt".to_string(), "yuv420p".to_string()]);
        }
        args.push(self.file.to_string_lossy().to_string());
        args
    }

    /// Encode the frames gathered and remove them. The movie file's folder
    /// is made; a frame that fails to read is a failure.
    pub fn finish(self) -> Result<BMovieMade, String> {
        if self.frames.is_empty() {
            return Err("no frames were rendered".to_string());
        }
        if let Some(folder) = self.file.parent().filter(|f| !f.as_os_str().is_empty()) {
            std::fs::create_dir_all(folder).map_err(|e| format!("{}: {e}", folder.display()))?;
        }
        let made = if Self::ffmpeg_present() {
            let out = Command::new("ffmpeg").args(self.ffmpeg_args()).output().map_err(|e| format!("ffmpeg: {e}"))?;
            if !out.status.success() {
                return Err(format!("ffmpeg failed: {}", String::from_utf8_lossy(&out.stderr).lines().last().unwrap_or("")));
            }
            BMovieMade::Ffmpeg(self.file.clone())
        } else {
            BMovieMade::Avi(self.write_avi()?)
        };
        let _ = std::fs::remove_dir_all(&self.frames_dir);
        Ok(made)
    }

    /// Every frame re-encoded as a JPEG at the Output quality, into an AVI
    /// beside where the FFmpeg movie would have gone.
    fn write_avi(&self) -> Result<PathBuf, String> {
        let file = self.file.with_extension("avi");
        let fail = |e: std::io::Error| format!("{}: {e}", file.display());
        let mut avi: Option<avi::BAvi<std::fs::File>> = None;
        for frame in &self.frames {
            let img = image::open(frame).map_err(|e| format!("{}: {e}", frame.display()))?.to_rgb8();
            let (w, h) = img.dimensions();
            let mut jpeg = Vec::new();
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, self.quality.clamp(1, 100)).encode(img.as_raw(), w, h, image::ExtendedColorType::Rgb8).map_err(|e| e.to_string())?;
            if avi.is_none() {
                avi = Some(avi::BAvi::begin(std::fs::File::create(&file).map_err(fail)?, w, h, self.fps).map_err(fail)?);
            }
            if let Some(a) = avi.as_mut() {
                a.frame(&jpeg).map_err(fail)?;
            }
        }
        if let Some(a) = avi {
            a.finish().map_err(fail)?;
        }
        Ok(file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_command_line_is_blenders_defaults_and_the_avi_is_the_fallback() {
        let dir = std::env::temp_dir().join(format!("phiber-video-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let still = dir.join("shot_.png");
        let mut job = BMovieJob::begin(&still, &BlendOutput::default(), BlendMovie::default(), 24.0, 1, 3).unwrap();
        assert_eq!(job.file, dir.join("shot_0001-0003.mkv"), "`BKE_movie_filepath_get`: the range and the container");
        let args = job.ffmpeg_args();
        let has = |a: &str, b: &str| args.windows(2).any(|w| w[0] == a && w[1] == b);
        assert!(has("-c:v", "libx264") && has("-crf", "23") && has("-preset", "medium") && has("-g", "18") && has("-framerate", "24") && has("-pix_fmt", "yuv420p"), "{args:?}");
        for k in 1..=3 {
            let path = job.next_frame_file();
            image::RgbImage::from_fn(8, 4, |x, _| image::Rgb([x as u8 * 30, 0, k as u8 * 60])).save(&path).unwrap();
        }
        // Without ffmpeg the fallback writes a Motion-JPEG AVI of the frames.
        let made = job.write_avi().unwrap();
        assert_eq!(made, dir.join("shot_0001-0003.avi"));
        let bytes = std::fs::read(&made).unwrap();
        assert_eq!(&bytes[8..12], b"AVI ");
        assert_eq!(bytes.windows(4).filter(|w| *w == b"00dc").count(), 3 + 3, "three frames, each once in `movi` and once in `idx1`");
        // With ffmpeg on the machine: the AVI reads back as three MJPEG
        // frames, and the real encode writes the H.264 Matroska asked for.
        if BMovieJob::ffmpeg_present() {
            let probe = |file: &Path| {
                let out = Command::new("ffprobe").args(["-v", "error", "-count_frames", "-show_entries", "stream=codec_name,nb_read_frames", "-of", "csv=p=0"]).arg(file).output().unwrap();
                String::from_utf8_lossy(&out.stdout).trim().to_string()
            };
            assert_eq!(probe(&made), "mjpeg,3", "ffprobe reads the AVI");
            let encoded = job.finish().unwrap();
            assert_eq!(encoded, BMovieMade::Ffmpeg(dir.join("shot_0001-0003.mkv")));
            assert_eq!(probe(&dir.join("shot_0001-0003.mkv")), "h264,3", "ffprobe reads the Matroska");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
