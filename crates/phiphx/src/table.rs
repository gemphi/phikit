//! The reader for phiphx's data tables (`data/*.tsv`): tab-separated
//! rows under a header line, `#` comment lines and blank lines skipped.
//! Code names columns by their header; the rows are the data.

/// A tab-separated table: its header and its rows of cells.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TsvTable {
    header: Vec<String>,
    row: Vec<Vec<String>>,
}

/// One row of a [`TsvTable`], read by column name.
#[derive(Clone, Copy, Debug)]
pub struct TsvRow<'a> {
    header: &'a [String],
    cell: &'a [String],
}

impl TsvTable {
    /// Read a table from its text: the first line that is neither blank
    /// nor a comment is the header.
    pub fn parse(text: &str) -> Self {
        let mut lines = text
            .lines()
            .map(|l| l.trim_end_matches('\r'))
            .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'));
        let split = |l: &str| l.split('\t').map(|c| c.trim().to_string()).collect::<Vec<_>>();
        let header = lines.next().map(split).unwrap_or_default();
        let row = lines.map(split).collect();
        Self { header, row }
    }

    /// Every row, in file order.
    pub fn row(&self) -> impl Iterator<Item = TsvRow<'_>> {
        self.row.iter().map(|cell| TsvRow { header: &self.header, cell })
    }

    /// The value in column `value` of the row whose column `key` is `name`
    /// (a key-value table such as `data/usd.tsv`).
    pub fn lookup(&self, key: &str, name: &str, value: &str) -> Option<&str> {
        self.row().find(|r| r.get(key) == name).map(|r| r.get(value))
    }
}

impl<'a> TsvRow<'a> {
    /// The cell under `column`; empty when the column or cell is missing.
    pub fn get(&self, column: &str) -> &'a str {
        self.header
            .iter()
            .position(|h| h == column)
            .and_then(|i| self.cell.get(i))
            .map(String::as_str)
            .unwrap_or("")
    }

    /// The cell as a number; `-`, empty or unreadable is 0.
    pub fn number(&self, column: &str) -> f32 {
        self.get(column).parse().unwrap_or(0.0)
    }

    /// The cell as a count; `-`, empty or unreadable is 0.
    pub fn count(&self, column: &str) -> usize {
        self.get(column).parse().unwrap_or(0)
    }

    /// The cell's space-separated words.
    pub fn word_list(&self, column: &str) -> Vec<&'a str> {
        self.get(column).split_whitespace().collect()
    }

    /// The cell's space-separated numbers (unreadable ones are 0).
    pub fn number_list(&self, column: &str) -> Vec<f32> {
        self.word_list(column).iter().map(|w| w.parse().unwrap_or(0.0)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_reads_rows_by_header() {
        let t = TsvTable::parse("# note\nname\tsize\tpart\n\ncube\t0.1\tbody pip\n");
        let row = t.row().next().unwrap();
        assert_eq!(row.get("name"), "cube");
        assert_eq!(row.number("size"), 0.1);
        assert_eq!(row.word_list("part"), ["body", "pip"]);
        assert_eq!(row.get("missing"), "");
        assert_eq!(t.lookup("name", "cube", "size"), Some("0.1"));
    }
}
