//! A look: a theme read from a table, so that what an app looks like is a
//! file of the app's and not code.

use super::PTheme;
use crate::pui::core::PColor;

/// A theme read from a table.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PLook {
    color: Vec<(String, PColor)>,
    size: Vec<(String, f32)>,
    word: Vec<(String, String)>,
}

impl PLook {
    /// A look from a table: one token a line, as its kind (`color`,
    /// `size` or `word`), its name and its value, between tabs. A colour
    /// is `#rrggbb` or `#rrggbbaa`. A line that is empty, starts with `#`
    /// or cannot be read is skipped. A token given twice has its last
    /// value, so rows added after a table change it.
    pub fn read(text: &str) -> PLook {
        PLook::default().with(text)
    }

    /// The same look with the rows of another table after its own: what
    /// they name is changed, and everything else is as it was. It is how
    /// a dark look is made from a light one.
    pub fn with(&self, text: &str) -> PLook {
        let mut look = self.clone();
        for line in text.lines().filter(|line| !line.trim().is_empty() && !line.starts_with('#')) {
            let mut cell = line.split('\t').map(str::trim);
            let (kind, token, value) = (cell.next().unwrap_or_default(), cell.next().unwrap_or_default().to_string(), cell.next().unwrap_or_default());
            match kind {
                "color" => look.color.extend(tint(value).map(|color| (token, color))),
                "size" => look.size.extend(value.parse().ok().map(|size| (token, size))),
                "word" => look.word.push((token, value.to_string())),
                _ => {}
            }
        }
        look
    }
}

/// The colour `#rrggbb` or `#rrggbbaa` writes.
fn tint(text: &str) -> Option<PColor> {
    let digit = text.strip_prefix('#')?;
    let pair = |index: usize| Some(u8::from_str_radix(digit.get(index * 2..index * 2 + 2)?, 16).ok()? as f32 / 255.0);
    let alpha = match digit.len() {
        6 => Some(1.0),
        8 => pair(3),
        _ => None,
    };
    Some(PColor::new(pair(0)?, pair(1)?, pair(2)?, alpha?))
}

impl PTheme for PLook {
    fn color(&self, token: &str) -> Option<PColor> {
        self.color.iter().rev().find(|(name, _)| name == token).map(|(_, color)| *color)
    }

    fn size(&self, token: &str) -> Option<f32> {
        self.size.iter().rev().find(|(name, _)| name == token).map(|(_, size)| *size)
    }

    fn word(&self, token: &str) -> Option<&str> {
        self.word.iter().rev().find(|(name, _)| name == token).map(|(_, word)| word.as_str())
    }
}
