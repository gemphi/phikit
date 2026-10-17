//! A project's name, from the asset it was made from.
//!
//! The rule is the file stem, and the interesting part is everything the
//! stem can be that a directory name cannot. The old module took
//! `file_stem().unwrap_or_default()` and carried on, so an asset at `/` or
//! one whose stem was `..` produced a bundle at a path nobody meant — and
//! a stem with a `/` or a `:` in it produced one that could not be created
//! at all, reported as an I/O error from two calls later.
//!
//! So the name is a type that is either valid or an explained refusal.

use std::path::Path;

/// A name a bundle can be created under.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BProjectName(pub String);

/// Why a path gives no project name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BNameError {
    /// The path has no file name at all — `/`, or one ending in `..`.
    NoStem,
    /// The stem is `.` or `..`, which name a directory rather than a file.
    Relative(String),
    /// The stem holds a character a path separator or a device name claims.
    Unusable(String, char),
}

impl BNameError {
    pub fn message(&self) -> String {
        match self {
            Self::NoStem => "the asset path has no file name to take a project name from".to_string(),
            Self::Relative(stem) => format!("{stem:?} names a directory, not a project"),
            Self::Unusable(stem, bad) => format!("{stem:?} cannot be a directory name: it holds {bad:?}"),
        }
    }
}

impl BProjectName {
    /// Characters no bundle directory may carry. The set is Windows'
    /// (`<>:"/\|?*`), because it is the strict one and a project written on
    /// one machine is opened on another.
    const FORBIDDEN: &'static [char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

    /// The project name `asset` gives, or why it gives none.
    pub fn of(asset: &Path) -> Result<Self, BNameError> {
        let stem = asset.file_stem().and_then(|stem| stem.to_str()).ok_or(BNameError::NoStem)?;
        let stem = stem.trim();
        if stem.is_empty() {
            return Err(BNameError::NoStem);
        }
        if stem == "." || stem == ".." {
            return Err(BNameError::Relative(stem.to_string()));
        }
        if let Some(bad) = stem.chars().find(|c| Self::FORBIDDEN.contains(c) || c.is_control()) {
            return Err(BNameError::Unusable(stem.to_string(), bad));
        }
        Ok(Self(stem.to_string()))
    }
}

#[cfg(test)]
#[path = "name_tests.rs"]
mod name_tests;
