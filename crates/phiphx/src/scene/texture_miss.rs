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
mod tests {
    use super::*;

    fn tally(keys: &[(&str, bool)]) -> BMissingTexture {
        let mut m = BMissingTexture::default();
        for (key, found) in keys {
            m.note(key, *found);
        }
        m
    }

    #[test]
    fn a_file_with_every_image_says_nothing() {
        let m = tally(&[("a.png", true), ("b.png", true)]);
        assert!(m.is_empty() && m.lines().is_empty());
    }

    #[test]
    fn the_count_leads_and_the_folders_are_ranked_by_what_they_owe() {
        // Pose 26's shape: many images, a few folders, one owing most.
        let mut keys: Vec<(String, bool)> = Vec::new();
        for i in 0..20 {
            keys.push((format!("//../../../Female-01-Texture-File/skin/map{i}.png"), false));
        }
        for i in 0..5 {
            keys.push((format!("E:/Graphigem/Outfits/Textures/cloth{i}.png"), false));
        }
        keys.push(("packed.png".into(), true));
        let m = tally(&keys.iter().map(|(k, f)| (k.as_str(), *f)).collect::<Vec<_>>());
        let lines = m.lines();
        assert!(lines[0].starts_with("25 of 26 images not found"), "{}", lines[0]);
        assert!(lines[0].contains("textures"), "it says what to do: {}", lines[0]);
        // The folder owing 20 comes before the one owing 5.
        assert!(lines[1].starts_with("  20 from") && lines[1].ends_with("Female-01-Texture-File/skin"), "{}", lines[1]);
        assert!(lines[2].starts_with("  5 from"), "{}", lines[2]);
        // Blender's own line survives as evidence, and the rest are counted.
        assert!(lines.iter().any(|l| l.starts_with("Cannot read '")), "{lines:?}");
        assert!(lines.iter().any(|l| l == "… and 19 more images"), "{lines:?}");
        // The whole report is a screenful, not 25 lines.
        assert!(lines.len() <= 3 + FOLDERS + EVIDENCE, "{} lines", lines.len());
    }

    #[test]
    fn one_image_asked_for_twice_is_one_image_missing() {
        // `attach` walks material slots, so the same key arrives repeatedly.
        let m = tally(&[("a.png", false), ("a.png", false), ("b.png", false)]);
        assert_eq!(m.len(), 2);
        assert!(m.lines()[0].starts_with("2 of 3 images not found"));
    }

    #[test]
    fn a_long_path_keeps_the_end_that_names_it_and_a_bare_name_says_beside_the_file() {
        assert_eq!(BMissingTexture::short("E:/a/b/c/d/e"), "…/c/d/e");
        assert_eq!(BMissingTexture::short("a/b"), "a/b");
        assert_eq!(BMissingTexture::short(""), "(beside the file)");
        // `//..\..` is Blender's relative prefix; the `..` parts carry no
        // information and are dropped rather than shown.
        assert_eq!(BMissingTexture::short("//../../Patterns"), "Patterns");
    }
}
