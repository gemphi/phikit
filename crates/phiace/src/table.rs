//! A table of data, as a robot's facts are kept: one row a line, cells
//! apart by tabs, a line that begins with `#` a comment. Nothing a robot
//! is or does is written in the code; it is read from its tables.

use std::path::Path;

/// The rows of a table.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Table {
    pub row: Vec<Vec<String>>,
}

impl Table {
    /// Read a table from its file; an empty table where there is none.
    pub fn read(file: &Path) -> Table {
        let text = std::fs::read_to_string(file).unwrap_or_default();
        let row = text.lines().filter(|line| !line.trim().is_empty() && !line.starts_with('#')).map(|line| line.split('\t').map(|cell| cell.trim().to_string()).collect()).collect();
        Table { row }
    }

    /// The rows whose first cell is `key`.
    pub fn rows<'a>(&'a self, key: &str) -> impl Iterator<Item = &'a Vec<String>> + 'a {
        let key = key.to_string();
        self.row.iter().filter(move |row| row.first().is_some_and(|first| *first == key))
    }

    /// The second cell of the row of `key`, or nothing.
    pub fn get(&self, key: &str) -> &str {
        self.rows(key).next().and_then(|row| row.get(1)).map_or("", String::as_str)
    }

    /// The number in the row of `key`, or nothing.
    pub fn num(&self, key: &str) -> f32 {
        self.get(key).parse().unwrap_or_default()
    }

    /// The first cells of every row, in order.
    pub fn keys(&self) -> Vec<&str> {
        self.row.iter().filter_map(|row| row.first()).map(String::as_str).collect()
    }
}
