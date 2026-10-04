//! Render ▸ Render Audio… — `sound.mixdown` (`sound_ops.cc`): the scene's
//! audio over its frame range, written to the file the browser names.
//! Blender's defaults are a FLAC container and codec, stereo, 16-bit
//! samples at the scene's 48 kHz; a scene without sound — the only kind
//! Phiber reads — comes out as that length of silence, which is what
//! Blender writes for it. The FLAC goes through `ffmpeg` when the machine
//! has one (as the movie writer does, `scene::video`); without it, a WAV
//! of the same samples, beside where the FLAC would have gone.

use std::path::{Path, PathBuf};

/// `sound.mixdown`'s settings as Blender defaults them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BMixdown {
    /// `mixrate` (Hz), `channels` (stereo), the frame range and rate.
    pub rate: u32,
    pub channels: u16,
    pub start: i32,
    pub end: i32,
    pub fps: f32,
}

/// What the mixdown came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BMixdownMade {
    /// Written by FFmpeg in the container the name asks for.
    Ffmpeg(PathBuf),
    /// FFmpeg was not found: a WAV instead, at this path.
    Wav(PathBuf),
}

impl BMixdown {
    pub const RATE: u32 = 48_000;
    pub const CHANNELS: u16 = 2;
    /// `aud::CONTAINER_FLAC`, the default extension the browser appends.
    pub const EXTENSION: &'static str = "flac";

    pub fn of_range(start: i32, end: i32, fps: f32) -> Self {
        Self { rate: Self::RATE, channels: Self::CHANNELS, start, end: end.max(start), fps: fps.max(1e-3) }
    }

    /// The range's length in seconds (`(end − start + 1) / fps`).
    pub fn seconds(&self) -> f32 {
        (self.end - self.start + 1) as f32 / self.fps
    }

    /// Samples a channel over the range.
    pub fn frames(&self) -> u64 {
        (self.seconds() as f64 * f64::from(self.rate)).round() as u64
    }

    /// The name the browser opens on: the `.blend`'s stem (`untitled` for a
    /// new file) with the default container's extension.
    pub fn name_for(blend: &Path) -> String {
        let stem = blend.file_stem().map(|n| n.to_string_lossy().to_string()).filter(|n| !n.is_empty()).unwrap_or_else(|| "untitled".to_string());
        format!("{stem}.{}", Self::EXTENSION)
    }

    /// `target` with the default container's extension when it names none.
    pub fn file_for(target: &Path) -> PathBuf {
        if target.extension().is_some() { target.to_path_buf() } else { target.with_extension(Self::EXTENSION) }
    }

    /// Write the mixdown to `target` (`file_for` it first).
    pub fn write(&self, target: &Path) -> Result<BMixdownMade, String> {
        let file = Self::file_for(target);
        if let Some(folder) = file.parent().filter(|f| !f.as_os_str().is_empty()) {
            std::fs::create_dir_all(folder).map_err(|e| format!("{}: {e}", folder.display()))?;
        }
        if super::video::BMovieJob::ffmpeg_present() && file.extension().is_some_and(|e| e != "wav") {
            let out = std::process::Command::new("ffmpeg").args(self.ffmpeg_args(&file)).output().map_err(|e| format!("ffmpeg: {e}"))?;
            if !out.status.success() {
                return Err(format!("ffmpeg failed: {}", String::from_utf8_lossy(&out.stderr).lines().last().unwrap_or("")));
            }
            return Ok(BMixdownMade::Ffmpeg(file));
        }
        let wav = file.with_extension("wav");
        self.write_wav(&wav).map_err(|e| format!("{}: {e}", wav.display()))?;
        Ok(BMixdownMade::Wav(wav))
    }

    /// Silence of the range's length, 16-bit, in the container `file` names.
    pub fn ffmpeg_args(&self, file: &Path) -> Vec<String> {
        let layout = if self.channels == 1 { "mono" } else { "stereo" };
        vec!["-y".into(), "-f".into(), "lavfi".into(), "-i".into(), format!("anullsrc=channel_layout={layout}:sample_rate={}", self.rate), "-t".into(), format!("{:.6}", self.seconds()), "-sample_fmt".into(), "s16".into(), file.to_string_lossy().to_string()]
    }

    /// A RIFF WAVE of 16-bit silence.
    pub fn write_wav(&self, file: &Path) -> std::io::Result<()> {
        use std::io::Write;
        let block = u32::from(self.channels) * 2;
        let data = (self.frames() * u64::from(block)).min(u64::from(u32::MAX - 44)) as u32;
        let mut out = std::io::BufWriter::new(std::fs::File::create(file)?);
        out.write_all(b"RIFF")?;
        out.write_all(&(36 + data).to_le_bytes())?;
        out.write_all(b"WAVEfmt ")?;
        out.write_all(&16u32.to_le_bytes())?;
        out.write_all(&1u16.to_le_bytes())?;
        out.write_all(&self.channels.to_le_bytes())?;
        out.write_all(&self.rate.to_le_bytes())?;
        out.write_all(&(self.rate * block).to_le_bytes())?;
        out.write_all(&(block as u16).to_le_bytes())?;
        out.write_all(&16u16.to_le_bytes())?;
        out.write_all(b"data")?;
        out.write_all(&data.to_le_bytes())?;
        let zeros = [0u8; 4096];
        let mut left = data as usize;
        while left > 0 {
            let n = left.min(zeros.len());
            out.write_all(&zeros[..n])?;
            left -= n;
        }
        out.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_range_becomes_that_many_seconds_of_silence() {
        let m = BMixdown::of_range(1, 48, 24.0);
        assert_eq!((m.seconds(), m.frames()), (2.0, 96_000));
        assert_eq!(BMixdown::file_for(Path::new("/tmp/mix")), PathBuf::from("/tmp/mix.flac"), "Blender's default container");
        assert_eq!(BMixdown::file_for(Path::new("/tmp/mix.ogg")), PathBuf::from("/tmp/mix.ogg"));
        assert_eq!((BMixdown::name_for(Path::new("/a/shot.blend")), BMixdown::name_for(Path::new(""))), ("shot.flac".to_string(), "untitled.flac".to_string()));
        let args = m.ffmpeg_args(Path::new("out.flac"));
        assert!(args.contains(&"anullsrc=channel_layout=stereo:sample_rate=48000".to_string()) && args.contains(&"2.000000".to_string()));
        let dir = std::env::temp_dir().join(format!("phiber-mixdown-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let wav = dir.join("mix.wav");
        BMixdown::of_range(1, 12, 24.0).write_wav(&wav).unwrap();
        let bytes = std::fs::read(&wav).unwrap();
        // Half a second of 16-bit stereo at 48 kHz: 96 000 bytes after the 44-byte header.
        assert_eq!(bytes.len(), 44 + 96_000);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..16], b"WAVEfmt ");
        assert_eq!(u16::from_le_bytes([bytes[22], bytes[23]]), 2, "stereo");
        assert_eq!(u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]), 48_000);
        assert_eq!(u16::from_le_bytes([bytes[34], bytes[35]]), 16, "S16");
        assert!(bytes[44..].iter().all(|b| *b == 0), "silence");
        // Through `write`: a `.wav` name is written as one whatever the machine has.
        assert_eq!(BMixdown::of_range(1, 1, 24.0).write(&dir.join("short.wav")).unwrap(), BMixdownMade::Wav(dir.join("short.wav")));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
