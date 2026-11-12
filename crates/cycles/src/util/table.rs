// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! The reader of the tab-separated files in `data/`. Names, labels and
//! report text live there, not in the code.

/// The rows of a data file, each cut at its tabs. Blank lines and lines that
/// start with `#` are left out.
pub fn row_list(text: &'static str) -> impl Iterator<Item = Vec<&'static str>> {
    text.lines().filter(|line| !line.trim().is_empty() && !line.starts_with('#')).map(|line| line.split('\t').collect())
}

#[cfg(test)]
mod test;
