//! The frame expression `-f` / `--render-frame` takes, ported from
//! `parse_int_range_relative_clamp_n` and the three parsers under it
//! (`source/creator/creator_args.cc`).
//!
//! It is not a number. It is a comma-separated list whose items are each
//! either one frame or a `..` range, and whose values may be written
//! relative to the scene's own range:
//!
//! | written | means |
//! | --- | --- |
//! | `12` | frame 12 |
//! | `+5` | `frame_start + 5` |
//! | `-5` | `(frame_end − 5) + 1` — note the **+1** |
//! | `1..10` | every frame from 1 to 10 |
//! | `1,5,9` | three single frames |
//! | `1..3,10,-1` | all three forms at once |
//!
//! Every value is clamped to [`MINAFRAME`] ..= [`MAXFRAME`] *after* the
//! relative form is resolved, so `-f -9999999` is the first frame rather
//! than an error. A range whose end precedes its start is not an error
//! either: Blender warns and renders nothing for it.

/// The scene's own frame range, which the relative forms are relative to
/// — `scene->r.sfra` and `scene->r.efra`, which Blender passes in as `pos`
/// and `neg`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BFrameLimit {
    pub first: i32,
    pub last: i32,
}

/// `MINAFRAME` / `MAXFRAME` (`DNA_scene_types.h`).
pub const MINAFRAME: i32 = -1_048_574;
pub const MAXFRAME: i32 = 1_048_574;

/// What went wrong, in Blender's own words — the message it prints after
/// `Error:` so a script that greps for it still matches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BFrameError {
    /// `"not a number"`: trailing characters, or nothing where a number
    /// should be.
    NotANumber,
    /// `"exceeds range"`: outside what an `int` holds. Note this is *not*
    /// the frame limit, which clamps instead.
    ExceedsRange,
    /// `"incorrect comma use"`: an empty item — a leading, trailing or
    /// doubled comma.
    IncorrectCommaUse,
}

impl BFrameError {
    pub fn message(self) -> &'static str {
        match self {
            Self::NotANumber => "not a number",
            Self::ExceedsRange => "exceeds range",
            Self::IncorrectCommaUse => "incorrect comma use",
        }
    }
}

/// One item of the list: an inclusive range. A single frame is a range
/// whose ends match, which is how Blender stores it too.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BFrameSpan {
    pub first: i32,
    pub last: i32,
}

impl BFrameSpan {
    /// The frames this span renders, in order. Empty when the end precedes
    /// the start — the case Blender warns about and skips.
    pub fn list(self) -> impl Iterator<Item = i32> {
        (self.first..=self.last).step_by(1)
    }

    /// Whether Blender would warn about this span rather than render it.
    pub fn reversed(self) -> bool {
        self.first > self.last
    }
}

/// A whole `-f` expression, parsed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BFrameSpec {
    pub span: Vec<BFrameSpan>,
}

impl BFrameSpec {
    /// `parse_int_relative`: `+n` from the start, `-n` from the end, or a
    /// plain number. `limit` carries the scene's own range, which is what
    /// the relative forms are relative to.
    fn value(text: &str, limit: BFrameLimit) -> Result<i32, BFrameError> {
        let (base, digits) = match text.as_bytes().first() {
            // `value = pos + strtol(str + 1)`
            Some(b'+') => (i64::from(limit.first), &text[1..]),
            // `value = (neg - strtol(str + 1)) + 1` — the off-by-one is
            // Blender's, and `-1` therefore means the last frame itself.
            Some(b'-') => (i64::from(limit.last) + 1, &text[1..]),
            _ => (0, text),
        };
        let signed = text.starts_with('-');
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return Err(BFrameError::NotANumber);
        }
        let magnitude: i64 = digits.parse().map_err(|_| BFrameError::ExceedsRange)?;
        let value = if signed { base - magnitude } else { base + magnitude };
        if value < i64::from(i32::MIN) || value > i64::from(i32::MAX) {
            return Err(BFrameError::ExceedsRange);
        }
        // `CLAMP(*r_value, min, max)` — after the relative form resolves.
        Ok((value as i32).clamp(MINAFRAME, MAXFRAME))
    }

    /// `parse_int_range_sep_search`: the `..` in an item, when it has one.
    /// A `..` with nothing after it is not a separator, so `1..` falls
    /// through to the number parser and fails as "not a number".
    fn separator(item: &str) -> Option<usize> {
        let at = item.find("..")?;
        (at + 2 < item.len()).then_some(at)
    }

    /// `parse_int_range_relative_clamp_n`: the comma-separated list.
    pub fn parse(text: &str, limit: BFrameLimit) -> Result<Self, BFrameError> {
        let mut span = Vec::new();
        for item in text.split(',') {
            // `if (ELEM(*str, sep, '\0'))` — an empty item.
            if item.is_empty() {
                return Err(BFrameError::IncorrectCommaUse);
            }
            match Self::separator(item) {
                Some(at) => {
                    let first = Self::value(&item[..at], limit)?;
                    let last = Self::value(&item[at + 2..], limit)?;
                    span.push(BFrameSpan { first, last });
                }
                None => {
                    let only = Self::value(item, limit)?;
                    span.push(BFrameSpan { first: only, last: only });
                }
            }
        }
        Ok(Self { span })
    }

    /// Every frame the expression names, in the order Blender renders them
    /// — each span in turn, and a reversed span contributing nothing.
    pub fn list(&self) -> Vec<i32> {
        self.span.iter().flat_map(|span| span.list()).collect()
    }
}

#[cfg(test)]
#[path = "frame_tests.rs"]
mod frame_tests;
