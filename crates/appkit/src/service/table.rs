//! A table of names and what they stand for, read from a file: an app's
//! labels, its settings. What a person reads is then a file of the app's
//! and not code.

/// Names and what they stand for.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PTable {
    row: Vec<(String, String)>,
}

impl PTable {
    /// A table from text: one name a line, then a tab, then what it stands
    /// for. A line that is empty, starts with `#` or has no tab is
    /// skipped. A name given twice keeps its first row.
    pub fn read(text: &str) -> PTable {
        let line = text.lines().filter(|line| !line.trim().is_empty() && !line.starts_with('#'));
        let row = line.filter_map(|line| line.split_once('\t')).map(|(name, value)| (name.trim().to_string(), value.trim().to_string()));
        PTable { row: row.collect() }
    }

    /// What a name stands for; nothing for a name the table does not have.
    pub fn get(&self, name: &str) -> &str {
        self.row.iter().find(|(known, _)| known == name).map_or("", |(_, value)| value.as_str())
    }

    /// Every row, in the order the text gave them.
    pub fn list(&self) -> &[(String, String)] {
        &self.row
    }

    /// What a name stands for, with `{word}` in it replaced by `with`.
    pub fn fill(&self, name: &str, word: &str, with: &str) -> String {
        self.get(name).replace(&format!("{{{word}}}"), with)
    }
}
