//! `Scene.r.ffcodecdata` — Properties → Output → Encoding, read when the
//! file format is FFmpeg Video (`R_IMF_IMTYPE_FFMPEG`): the container, the
//! video codec, the constant rate factor, the encoding speed preset and
//! the keyframe interval (`DNA_scene_types.h`'s `FFMpegCodecData`,
//! `imbuf/movie/MOV_enums.hh`), and the name a movie file takes
//! (`BKE_movie_filepath_get`: the path, the frame range, the container's
//! extension).

use super::super::{BlendBlock, BlendFile, DnaType};

/// `FFMpegCodecData`: what the encoder is told.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlendMovie {
    /// The container (`FFMPEG_MPEG4` 2, `AVI` 3, `MOV` 4, `MKV` 9, `OGG` 10,
    /// `WEBM` 12, …).
    pub container: i32,
    /// `IMB_Ffmpeg_Codec_ID`: H.264 27, H.265 173, AV1 226, VP9 167, MPEG-4
    /// 12, PNG 61, FFV1 33, Theora 30, ProRes 147, DNxHD 99, QT RLE 55.
    pub codec: i32,
    /// `eFFMpegCrf`: 0 lossless … 23 medium … 32 lowest; 128 custom
    /// (`custom_crf` then), −1 none (bitrate).
    pub crf: i32,
    pub custom_crf: i32,
    /// `eFFMpegPreset`: 10 Good ("medium"), 11 Best ("slower"), 12 Realtime
    /// ("superfast").
    pub preset: i32,
    /// `gop_size`: frames between keyframes (0 lets the encoder choose).
    pub gop: i32,
    pub bitrate: i32,
}

impl Default for BlendMovie {
    /// A new file's (`BKE_scene_init`): Matroska, H.264, medium quality,
    /// the good preset, a keyframe every 18 frames.
    fn default() -> Self {
        Self { container: Self::MKV, codec: Self::H264, crf: 23, custom_crf: 23, preset: 10, gop: 18, bitrate: 6000 }
    }
}

impl BlendMovie {
    pub const MPEG4: i32 = 2;
    pub const AVI: i32 = 3;
    pub const MOV: i32 = 4;
    pub const MKV: i32 = 9;
    pub const OGG: i32 = 10;
    pub const WEBM: i32 = 12;
    pub const H264: i32 = 27;

    /// `get_file_extensions`: the container's extension.
    pub fn extension(&self) -> &'static str {
        match self.container {
            0 => ".mpg",
            1 => ".dvd",
            Self::AVI | 7 => ".avi",
            Self::MOV => ".mov",
            5 => ".dv",
            8 => ".flv",
            Self::MKV => ".mkv",
            Self::OGG => ".ogv",
            Self::WEBM => ".webm",
            _ => ".mp4",
        }
    }

    /// FFmpeg's name for the video codec (`MOV_codec_…` → `avcodec_find_encoder`).
    pub fn codec_name(&self) -> &'static str {
        match self.codec {
            Self::H264 => "libx264",
            173 => "libx265",
            226 => "libaom-av1",
            167 => "libvpx-vp9",
            12 => "mpeg4",
            61 => "png",
            33 => "ffv1",
            25 => "huffyuv",
            30 => "libtheora",
            147 => "prores_ks",
            99 => "dnxhd",
            55 => "qtrle",
            1 => "mpeg1video",
            2 => "mpeg2video",
            _ => "libx264",
        }
    }

    /// The constant rate factor the encoder gets, or `None` for a bitrate.
    pub fn crf_value(&self) -> Option<i32> {
        match self.crf {
            -1 => None,
            128 => Some(self.custom_crf),
            crf => Some(crf),
        }
    }

    /// `movie_write.cc`'s preset names: `preset` for x264, `deadline` for VP9.
    pub fn preset_name(&self) -> Option<(&'static str, &'static str)> {
        match self.preset {
            10 => Some(("medium", "good")),
            11 => Some(("slower", "best")),
            12 => Some(("superfast", "realtime")),
            _ => None,
        }
    }

    /// `BKE_movie_filepath_get`: the still's path with its image extension
    /// dropped, the frame range appended (`BLI_path_frame_range`, four
    /// digits each) and the container's extension.
    pub fn file_of(&self, still: &std::path::Path, start: i32, end: i32) -> std::path::PathBuf {
        let name = still.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "Untitled".into());
        let stem = match name.rfind('.') {
            Some(dot) if dot > 0 => name[..dot].to_string(),
            _ => name,
        };
        still.with_file_name(format!("{stem}{start:04}-{end:04}{}", self.extension()))
    }
}

impl BlendFile {
    /// Read Output → Encoding off the scene block.
    pub(crate) fn extract_movie(&self, sc: &BlendBlock, dna: &DnaType) -> BlendMovie {
        let ps = self.header.pointer_size as usize;
        let d = &sc.data;
        let dflt = BlendMovie::default();
        let Some(r) = Self::struct_field_offset_exact("Scene", "r", dna, ps) else { return dflt };
        let Some(ff) = Self::struct_field_offset_exact("RenderData", "ffcodecdata", dna, ps) else { return dflt };
        let int = |field: &str, or: i32| match Self::struct_field_offset_exact("FFMpegCodecData", field, dna, ps) {
            Some(o) if r + ff + o + 4 <= d.len() => self.read_i32(&d[r + ff + o..r + ff + o + 4]),
            _ => or,
        };
        BlendMovie {
            container: int("type", dflt.container),
            codec: int("codec", dflt.codec),
            crf: int("constant_rate_factor", dflt.crf),
            custom_crf: int("custom_constant_rate_factor", dflt.custom_crf),
            preset: int("ffmpeg_preset", dflt.preset),
            gop: int("gop_size", dflt.gop),
            bitrate: int("video_bitrate", dflt.bitrate),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_movie_is_named_after_its_range_and_container() {
        let m = BlendMovie::default();
        assert_eq!(m.file_of(std::path::Path::new("/renders/shot_.png"), 1, 250), std::path::PathBuf::from("/renders/shot_0001-0250.mkv"));
        assert_eq!(BlendMovie { container: BlendMovie::MPEG4, ..m }.file_of(std::path::Path::new("C:/r/Untitled"), 8, 12), std::path::PathBuf::from("C:/r/Untitled0008-0012.mp4"));
        assert_eq!((m.codec_name(), m.crf_value(), m.preset_name()), ("libx264", Some(23), Some(("medium", "good"))));
        assert_eq!(BlendMovie { crf: 128, custom_crf: 17, ..m }.crf_value(), Some(17));
        assert_eq!(BlendMovie { crf: -1, ..m }.crf_value(), None, "a bitrate instead");
    }
}
