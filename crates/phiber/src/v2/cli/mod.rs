//! Rendering from the command line, the way Blender does it.
//!
//! `phiber -b scene.blend -f 12 -o //render/` reads the file, renders the
//! frame and writes it, with no window — the flags, their short and long
//! spellings and the frame grammar are
//! [Blender's own](set::ARG) (`source/creator/creator_args.cc`), so a
//! script written against `blender -b` runs here unchanged.
//!
//! Two things differ, both deliberate and both stated by `--help`:
//!
//! - **Phiray is the engine** unless `-E` names another
//!   ([`BRenderQuality::ENGINE`]). A `.blend` saved by Blender names EEVEE
//!   or Cycles, neither of which is ours to run, so the file's engine is a
//!   preference here rather than an instruction.
//! - **2048 samples is the floor** ([`BRenderQuality::SAMPLE_FLOOR`]), so
//!   the default is the quality render rather than whatever the file was
//!   saved with. `--draft` lifts the floor for a quick look.
//!
//! Parsing is separated from doing: [`BCli::parse`] turns an argument list
//! into a [`BCli`] and nothing else, which is why the grammar is testable
//! without a window, a GPU or a file.

pub mod frame;
pub mod help;
pub mod render;
pub mod set;

use frame::{BFrameError, BFrameLimit, BFrameSpec};
use set::{BArgTake, BRenderQuality, spec_of};

/// What the command line asked for.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BCli {
    /// `-b`: no window. A render without it opens the app instead.
    pub background: bool,
    /// The `.blend` to open — the first bare word, as Blender takes it.
    pub file: Option<String>,
    /// `-f`: the frames to render, unparsed until the scene's own range is
    /// known (the relative forms need it).
    pub frame: Option<String>,
    /// `-a`, **after** a file: render the scene's whole range.
    pub animation: bool,
    /// `-a`, **before** a file: play back a rendered sequence instead.
    pub playback: bool,
    /// `-S`, `-s`, `-e`, `-j`.
    pub scene: Option<String>,
    pub first: Option<String>,
    pub last: Option<String>,
    pub step: Option<u32>,
    /// `-o`.
    pub output: Option<String>,
    /// `-E`, defaulting to Phiray.
    pub engine: Option<String>,
    /// `-F`.
    pub format: Option<String>,
    /// `-x`: `0` / `1`.
    pub extension: Option<bool>,
    /// `-t`.
    pub threads: Option<u32>,
    pub quiet: bool,
    pub help: bool,
    pub version: bool,
    /// `--samples` and `--draft`.
    pub samples: Option<u32>,
    pub draft: bool,
}

/// Why a command line was refused, in Blender's own wording.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BCliError {
    /// `"Error: <flag> requires one argument"`.
    MissingValue(String),
    /// A flag that takes a number got something else.
    NotANumber(String, String),
    /// `-E` or `-F` naming something that is not on the list.
    Unknown(String, String),
    /// The frame expression did not parse.
    Frame(BFrameError),
}

impl BCliError {
    pub fn message(&self) -> String {
        match self {
            Self::MissingValue(flag) => format!("Error: {flag} requires one argument"),
            Self::NotANumber(flag, word) => format!("Error: not a number '{flag} {word}'"),
            Self::Unknown(flag, word) => format!("Error: unknown '{flag} {word}'"),
            Self::Frame(error) => format!("Error: {} '-f'", error.message()),
        }
    }
}

impl BCli {
    /// Parse an argument list — **without** the program name.
    ///
    /// Blender registers `-a` twice, as playback before a file argument
    /// and as `--render-anim` after one, so which it means depends on
    /// whether a `.blend` has been seen yet. That is the only positional
    /// rule in the grammar; everything else is order-free.
    pub fn parse<S: AsRef<str>>(argv: &[S]) -> Result<Self, BCliError> {
        let mut cli = Self::default();
        let mut index = 0;
        while index < argv.len() {
            let word = argv[index].as_ref();
            index += 1;
            let Some(spec) = spec_of(word) else {
                // A bare word is the file, as Blender takes the first one.
                if !word.starts_with('-') && cli.file.is_none() {
                    cli.file = Some(word.to_string());
                }
                continue;
            };
            let value = match spec.take {
                BArgTake::Nothing => None,
                BArgTake::Word => {
                    let next = argv.get(index).map(|s| s.as_ref().to_string());
                    let Some(next) = next else {
                        return Err(BCliError::MissingValue(cli.flag_name(spec)));
                    };
                    index += 1;
                    Some(next)
                }
            };
            cli.apply(spec, value)?;
        }
        Ok(cli)
    }

    /// How a flag is named in an error — the long form when it has one, as
    /// Blender prints `"-f / --render-frame"`.
    fn flag_name(&self, spec: &set::BArgSpec) -> String {
        match (spec.short.is_empty(), spec.long.is_empty()) {
            (false, false) => format!("{} / {}", spec.short, spec.long),
            (true, false) => spec.long.to_string(),
            _ => spec.short.to_string(),
        }
    }

    fn number(&self, spec: &set::BArgSpec, word: &str) -> Result<u32, BCliError> {
        word.parse().map_err(|_| BCliError::NotANumber(self.flag_name(spec), word.to_string()))
    }

    fn apply(&mut self, spec: &set::BArgSpec, value: Option<String>) -> Result<(), BCliError> {
        let word = value.unwrap_or_default();
        match (spec.short, spec.long) {
            ("-b", _) => self.background = true,
            ("-f", _) => self.frame = Some(word),
            // The one positional rule: before a file this is playback.
            ("-a", _) if self.file.is_none() && set::PLAYBACK_IS_BEFORE_A_FILE => self.playback = true,
            ("-a", _) => self.animation = true,
            ("-S", _) => self.scene = Some(word),
            ("-s", _) => self.first = Some(word),
            ("-e", _) => self.last = Some(word),
            ("-j", _) => self.step = Some(self.number(spec, &word)?),
            ("-o", _) => self.output = Some(word),
            ("-E", _) => {
                let upper = word.to_uppercase();
                if word != "help" && !set::ENGINE.contains(&upper.as_str()) {
                    return Err(BCliError::Unknown("-E".into(), word));
                }
                self.engine = Some(upper);
            }
            ("-F", _) => {
                let upper = word.to_uppercase();
                if word != "help" && !set::FORMAT.contains(&upper.as_str()) {
                    return Err(BCliError::Unknown("-F".into(), word));
                }
                self.format = Some(upper);
            }
            // `arg_handle_extension_set`: "0" off, anything else on.
            ("-x", _) => self.extension = Some(word != "0"),
            ("-t", _) => self.threads = Some(self.number(spec, &word)?),
            ("-q", _) => self.quiet = true,
            ("-h", _) => self.help = true,
            ("-v", _) => self.version = true,
            (_, "--samples") => self.samples = Some(self.number(spec, &word)?),
            (_, "--draft") => self.draft = true,
            _ => {}
        }
        Ok(())
    }

    /// Whether this invocation renders at all, rather than opening the app.
    pub fn renders(&self) -> bool {
        self.frame.is_some() || self.animation
    }

    /// The engine the render runs on: `-E`, else Phiray.
    pub fn engine_name(&self) -> &str {
        self.engine.as_deref().unwrap_or(BRenderQuality::ENGINE)
    }

    /// The samples the render takes, floor applied.
    pub fn sample_count(&self) -> u32 {
        BRenderQuality::samples(self.samples, self.draft)
    }

    /// The frames to render, given the scene's own range. `-f` decides it
    /// when present; otherwise `-a` renders `first ..= last`, with `-s` and
    /// `-e` overriding the scene's own ends and `-j` the step.
    pub fn frame_list(&self, scene: BFrameLimit) -> Result<Vec<i32>, BCliError> {
        if let Some(text) = &self.frame {
            return BFrameSpec::parse(text, scene).map(|spec| spec.list()).map_err(BCliError::Frame);
        }
        if !self.animation {
            return Ok(Vec::new());
        }
        let end = |text: &Option<String>, fallback: i32| -> Result<i32, BCliError> {
            match text {
                Some(text) => BFrameSpec::parse(text, scene)
                    .map(|spec| spec.span.first().map_or(fallback, |span| span.first))
                    .map_err(BCliError::Frame),
                None => Ok(fallback),
            }
        };
        let (first, last) = (end(&self.first, scene.first)?, end(&self.last, scene.last)?);
        let step = self.step.unwrap_or(1).max(1) as usize;
        Ok((first..=last).step_by(step).collect())
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
