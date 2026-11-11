//! What a file asked for and could not be given, said once.
//!
//! Blender prints `Cannot read '<path>': No such file or directory` per
//! image, and that is what phiphx printed too — which is right for the one
//! or two a tidy file is missing and useless for the case that actually
//! turns up. CW-POSES pose 26 asks for **304 images, 289 of them not on
//! this machine** (they are at `E:\Graphigem\…`, the paths of the computer
//! the file was made on), so the Info editor filled with 289 identical-
//! looking lines and the one thing a reader needs — *which folders do I
//! copy?* — was not among them.
//!
//! So the tally leads with the answer and keeps Blender's lines as
//! evidence behind it. **A recorded divergence**: Blender would print all
//! 289.
//!
//! The grouping is by the folder the file asks for, because that is the
//! unit someone copies. Find Missing Files (`texture_find.rs`) already
//! searches the `.blend`'s own folder and a `textures/` beside it, so the
//! instruction the summary gives is one a reader can actually carry out.

use std::collections::BTreeMap;

/// How many of Blender's own per-image lines are kept before the rest are
/// counted instead. Enough to recognise the names, short enough to read.
const EVIDENCE: usize = 6;

/// How many folders the summary names before it stops. A file asking for
/// more than this many has a bigger problem than a list would fix.
const FOLDERS: usize = 10;

/// Every image asked for, and which of them were not found.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BMissingTexture {
    asked: usize,
    /// The keys not found, deduped, in the order they were asked for.
    missing: Vec<String>,
}

impl BMissingTexture {
    /// One image asked for, and whether it was there.
    pub fn note(&mut self, key: &str, found: bool) {
        self.asked += 1;
        if found || key.is_empty() || self.missing.iter().any(|k| k == key) {
            return;
        }
        self.missing.push(key.to_string());
    }

    pub fn is_empty(&self) -> bool {
        self.missing.is_empty()
    }

    /// How many distinct images were not found.
    pub fn len(&self) -> usize {
        self.missing.len()
    }

    /// The folder a key asks for, as the file spells it — the unit someone
    /// copies. An image with no folder at all (a packed name) groups under
    /// the empty string and is reported as "beside the file".
    fn folder(key: &str) -> &str {
        match key.rfind(['/', '\\']) {
            Some(cut) => &key[..cut],
            None => "",
        }
    }

    /// A folder shortened to its last three parts, so a path from another
    /// machine fits a line without losing the part that identifies it.
    fn short(folder: &str) -> String {
        if folder.is_empty() {
            return "(beside the file)".to_string();
        }
        let parts: Vec<&str> = folder.split(['/', '\\']).filter(|p| !p.is_empty() && *p != "..").collect();
        match parts.len() > 3 {
            true => format!("…/{}", parts[parts.len() - 3..].join("/")),
            false => parts.join("/"),
        }
    }

    /// The report: the count and what to do about it, a line per folder
    /// with how many images it owes, then Blender's own lines as evidence.
    /// Empty when nothing is missing.
    pub fn lines(&self) -> Vec<String> {
        if self.is_empty() {
            return Vec::new();
        }
        let mut out = vec![format!(
            "{} of {} images not found — copy their folders beside the .blend, or into a `textures` folder next to it, and reopen (both are searched by name)",
            self.len(),
            self.asked,
        )];
        let mut by_folder: BTreeMap<String, usize> = BTreeMap::new();
        for key in &self.missing {
            *by_folder.entry(Self::short(Self::folder(key))).or_default() += 1;
        }
        let mut ranked: Vec<(String, usize)> = by_folder.into_iter().collect();
        // Most owed first; the name breaks ties so the order is stable
        // across reads of the same file.
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let shown = ranked.len().min(FOLDERS);
        for (folder, n) in &ranked[..shown] {
            out.push(format!("  {n} from {folder}"));
        }
        if ranked.len() > shown {
            out.push(format!("  … and {} more folders", ranked.len() - shown));
        }
        // Blender's own line, for the first few: the evidence behind the
        // count, in the words anyone who has used Blender will recognise.
        for key in self.missing.iter().take(EVIDENCE) {
            out.push(format!("Cannot read '{key}': No such file or directory"));
        }
        if self.len() > EVIDENCE {
            out.push(format!("… and {} more images", self.len() - EVIDENCE));
        }
        out
    }
}

#[cfg(test)]
#[path = "test/texture_miss.rs"]
mod texture_miss_test;
