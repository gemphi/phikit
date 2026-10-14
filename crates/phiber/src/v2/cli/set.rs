//! The arguments, as data — one row per flag, in the order and with the
//! spellings `setupArguments` registers them (`creator_args.cc`).
//!
//! Blender's own render flags are kept verbatim, short and long form, so a
//! script written for `blender -b` runs against `phiber -b` unchanged.
//! Phiber's additions sit after them and are long-form only, so they can
//! never collide with a short flag Blender adds later.

/// What a flag does with the word after it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BArgTake {
    /// A flag on its own — `-b`, `-a`, `-x`.
    Nothing,
    /// One word follows — `-f 12`, `-o /tmp/out`.
    Word,
}

/// One flag the command line accepts.
#[derive(Clone, Copy, Debug)]
pub struct BArgSpec {
    /// The short form, as Blender spells it (`"-f"`), or empty for a
    /// long-only flag.
    pub short: &'static str,
    /// The long form (`"--render-frame"`), or empty where Blender has
    /// none (`-a` as *playback* has no long form; `-a` as `--render-anim`
    /// does — see [`PLAYBACK_IS_BEFORE_A_FILE`]).
    pub long: &'static str,
    pub take: BArgTake,
    /// One line for `--help`, in Blender's wording where it has one.
    pub help: &'static str,
}

const fn arg(short: &'static str, long: &'static str, take: BArgTake, help: &'static str) -> BArgSpec {
    BArgSpec { short, long, take, help }
}

/// `-a` is registered **twice** in Blender: once before a file argument as
/// animation *playback*, and once after as `--render-anim`. Which one a
/// given `-a` means therefore depends on whether a `.blend` has been named
/// yet, and that is the one piece of this grammar that is positional.
pub const PLAYBACK_IS_BEFORE_A_FILE: bool = true;

/// Blender's render arguments, plus the general ones that change what a
/// render does.
pub static ARG: &[BArgSpec] = &[
    arg("-b", "--background", BArgTake::Nothing, "Run in the background (often used for UI-less rendering)."),
    arg("-f", "--render-frame", BArgTake::Word, "Render frame <frame> and save it. +<frame> and -<frame> are relative to the scene's range; <a>..<b> is a range; a comma lists several."),
    arg("-a", "--render-anim", BArgTake::Nothing, "Render frames from start to end (inclusive)."),
    arg("-S", "--scene", BArgTake::Word, "Set the active scene <name> for rendering."),
    arg("-s", "--frame-start", BArgTake::Word, "Set start to frame <frame>, supports +/- for relative frames too."),
    arg("-e", "--frame-end", BArgTake::Word, "Set end to frame <frame>, supports +/- for relative frames too."),
    arg("-j", "--frame-jump", BArgTake::Word, "Set number of frames to step forward after each rendered frame."),
    arg("-o", "--render-output", BArgTake::Word, "Set the render path and file name. Use '//' for a relative path."),
    arg("-E", "--engine", BArgTake::Word, "Specify the render engine. Use `-E help` to list available engines."),
    arg("-F", "--render-format", BArgTake::Word, "Set the render format. Use `-F help` to list available formats."),
    arg("-x", "--use-extension", BArgTake::Word, "Set option to add the file extension to the end of the file."),
    arg("-t", "--threads", BArgTake::Word, "Use amount of <threads> for rendering in background."),
    arg("-q", "--quiet", BArgTake::Nothing, "Print only errors and warnings."),
    arg("-h", "--help", BArgTake::Nothing, "Print this help text and exit."),
    arg("-v", "--version", BArgTake::Nothing, "Print Phiber version and exit."),
    // ── Phiber's own, long-form only.
    arg("", "--samples", BArgTake::Word, "Set the sample count for the render. Clamped up to the quality floor (2048)."),
    arg("", "--draft", BArgTake::Nothing, "Lift the quality floor, so --samples below it is honoured."),
];

/// The engines `-E` accepts, in the order `-E help` lists them. Phiray is
/// first because it is the default (`BRenderQuality::ENGINE`).
pub static ENGINE: &[&str] = &["PHIRAY", "CYCLES", "BLENDER_EEVEE_NEXT", "BLENDER_WORKBENCH"];

/// The formats `-F` accepts, in the order `-F help` lists them
/// (`arg_handle_image_type_set`'s `imtype_from_arg`).
pub static FORMAT: &[&str] = &[
    "TGA", "RAWTGA", "JPEG", "IRIS", "PNG", "BMP", "HDR", "TIFF", "OPEN_EXR", "OPEN_EXR_MULTILAYER", "CINEON", "DPX",
    "JP2", "WEBP", "AVIJPEG", "AVIRAW", "FFMPEG",
];

impl BArgSpec {
    /// Whether `word` is this flag, in either spelling.
    pub fn names(&self, word: &str) -> bool {
        (!self.short.is_empty() && self.short == word) || (!self.long.is_empty() && self.long == word)
    }
}

/// The flag `word` names, if any.
pub fn spec_of(word: &str) -> Option<&'static BArgSpec> {
    ARG.iter().find(|spec| spec.names(word))
}

/// The quality policy: what a render defaults to when the command line and
/// the file are both silent.
pub struct BRenderQuality;

impl BRenderQuality {
    /// The engine a render uses unless `-E` says otherwise — Phiber's own
    /// tracer, not whatever engine the `.blend` was saved with. A file from
    /// Blender names EEVEE or Cycles and neither is ours to run, so the
    /// file's engine is a *preference* here and not an instruction.
    pub const ENGINE: &'static str = "PHIRAY";

    /// The sample floor. A render takes at least this many samples, so the
    /// default is the quality one rather than whatever a file happened to
    /// be saved with; `--draft` lifts it for a quick look.
    pub const SAMPLE_FLOOR: u32 = 2048;

    /// The samples a render actually takes: what was asked for, never
    /// below the floor unless the floor was lifted. `None` asked for means
    /// the floor itself.
    pub fn samples(asked: Option<u32>, draft: bool) -> u32 {
        match (asked, draft) {
            (Some(asked), true) => asked.max(1),
            (Some(asked), false) => asked.max(Self::SAMPLE_FLOOR),
            (None, _) => Self::SAMPLE_FLOOR,
        }
    }
}
