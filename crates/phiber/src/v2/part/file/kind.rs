//! File type classification — Blender's `FILE_TYPE_*` bits, reduced to the
//! kinds the browser actually filters and icons on.
//! Blender ref: `DNA_space_enums.h:669` and `filelist_filter.cc`.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BFileKind {
    Folder,
    /// `.blend`
    Blend,
    /// `.blend1`, `.blend2`, … — Blender's backup saves.
    BlendBackup,
    Image,
    Movie,
    Script,
    Font,
    Sound,
    Text,
    /// OpenVDB and friends.
    Volume,
    Archive,
    /// glTF / USD / OBJ / FBX / PLY — importable scene formats.
    Model,
    Other,
}

impl BFileKind {
    /// Classify by extension. Directories short-circuit to `Folder`.
    pub fn of(name: &str, is_dir: bool) -> Self {
        if is_dir {
            return Self::Folder;
        }
        let ext = name
            .rsplit_once('.')
            .map(|(_, e)| e.to_ascii_lowercase())
            .unwrap_or_default();
        // `.blend1` … `.blend32` are backups; bare `.blend` is not.
        if ext.starts_with("blend") && ext != "blend" {
            return Self::BlendBackup;
        }
        match ext.as_str() {
            "blend" => Self::Blend,
            "png" | "jpg" | "jpeg" | "tga" | "tif" | "tiff" | "exr" | "hdr" | "bmp" | "webp"
            | "gif" | "psd" | "dds" => Self::Image,
            "mp4" | "mov" | "avi" | "mkv" | "webm" | "mpg" | "mpeg" | "ogv" => Self::Movie,
            "py" | "rs" | "js" | "ts" | "sh" | "ps1" | "bat" | "toml" | "json" | "yaml" | "yml" => {
                Self::Script
            }
            "ttf" | "otf" | "woff" | "woff2" | "pfb" => Self::Font,
            "wav" | "mp3" | "ogg" | "flac" | "aiff" | "aac" => Self::Sound,
            "txt" | "md" | "log" | "csv" | "rst" => Self::Text,
            "vdb" => Self::Volume,
            "zip" | "7z" | "gz" | "tar" | "bz2" | "xz" | "rar" => Self::Archive,
            "gltf" | "glb" | "usd" | "usda" | "usdc" | "usdz" | "obj" | "fbx" | "ply" | "abc"
            | "stl" | "phx" => Self::Model,
            _ => Self::Other,
        }
    }

    /// Icon name from the shipped `assets/icons` set.
    pub fn icon(self) -> &'static str {
        match self {
            Self::Folder => "folder",
            Self::Blend => "blend",
            Self::BlendBackup => "file_blend",
            Self::Image => "image",
            Self::Movie => "film",
            Self::Script => "file_code",
            Self::Font => "type",
            Self::Sound => "audio",
            Self::Text => "file_text",
            Self::Volume => "cloud",
            Self::Archive => "package",
            Self::Model => "cube",
            Self::Other => "file",
        }
    }

    /// The `ctx.file.filter_*` flag that hides this kind, if any.
    /// `Other` has no toggle — Blender hides unrecognised files whenever any
    /// type filter is active.
    pub fn filter_flag(self) -> Option<&'static str> {
        Some(match self {
            Self::Folder => "filter_folder",
            Self::Blend => "filter_blender",
            Self::BlendBackup => "filter_backup",
            Self::Image => "filter_image",
            Self::Movie => "filter_movie",
            Self::Script => "filter_script",
            Self::Font => "filter_font",
            Self::Sound => "filter_sound",
            Self::Text => "filter_text",
            Self::Volume => "filter_volume",
            Self::Archive | Self::Model | Self::Other => return None,
        })
    }
}
