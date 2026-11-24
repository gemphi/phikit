//! The records search finds that are not Phiogy's own: Phiegg's worlds,
//! clubs, shows (`data/search.tsv` says which), the very records Phiegg
//! reads. Asked of Phiora where it gave the site; else read from
//! Phiegg's own files by the manifest Phiora seeds from (`tree.json`).
//! And what is sought most often, from Phiegg's site file.

use std::path::Path;

use serde_json::Value;

use crate::data;
use crate::kept::Kept;

/// A record found: its name and its line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub name: String,
    pub line: String,
}

/// Every record of each kind search asks for, and what is sought most.
#[derive(Debug, Clone, Default)]
pub struct Book {
    pub kind: Vec<(String, Vec<Found>)>,
    pub popular: Vec<String>,
}

/// The cells of the first row of that name in `data/search.tsv`.
fn cells(name: &str) -> Vec<&'static str> {
    data::row(data::SEARCH).into_iter().find(|row| row.first() == Some(&name)).map(|row| row[1..].to_vec()).unwrap_or_default()
}

/// A path of `data/search.tsv`, from this crate's folder.
fn file(relative: &str) -> Option<Value> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

impl Book {
    /// The records of these kinds, from Phiora where `kept` reaches it,
    /// else from Phiegg's files.
    pub fn read(kept: &Kept, kinds: &[&str]) -> Book {
        let ask = cells("ask").first().copied().unwrap_or_default().replace("{kind}", &kinds.join(","));
        let asked: Option<Value> = kept.read(&ask).and_then(|text| serde_json::from_str(&text).ok());
        let rows = |kind: &str| -> Vec<Value> {
            let from_phiora = asked.as_ref().and_then(|tree| tree.get("sources")?.as_array()?.iter().find(|s| s.get("kind").and_then(Value::as_str) == Some(kind))?.get("rows")?.as_array().cloned());
            from_phiora.unwrap_or_else(|| Book::files(kind))
        };
        let words = |row: &str| cells(row).iter().flat_map(|cell| cell.split_whitespace()).collect::<Vec<_>>();
        let (name, line) = (words("name"), words("line"));
        let word = |row: &Value, fields: &[&str]| fields.iter().find_map(|field| row.get(*field).and_then(Value::as_str).filter(|text| !text.is_empty())).unwrap_or_default().to_string();
        let kind = kinds.iter().map(|kind| (kind.to_string(), rows(kind).iter().map(|row| Found { name: word(row, &name), line: word(row, &line) }).filter(|found| !found.name.is_empty()).collect())).collect();
        let popular = cells("popular");
        let popular = popular.first().and_then(|path| file(path)).and_then(|site| site.get(*popular.get(1)?)?.as_array().cloned()).unwrap_or_default();
        Book { kind, popular: popular.iter().filter_map(Value::as_str).map(str::to_string).collect() }
    }

    /// A kind's rows from Phiegg's own files, by the manifest: the list
    /// it names in the file it names.
    fn files(kind: &str) -> Vec<Value> {
        let tree = cells("tree").first().and_then(|path| file(path)).unwrap_or_default();
        let folder = cells("tree").first().map(|path| Path::new(path).parent().unwrap_or(Path::new("")).to_path_buf()).unwrap_or_default();
        let source = tree.get("sources").and_then(Value::as_array).and_then(|list| list.iter().find(|s| s.get("kind").and_then(Value::as_str) == Some(kind) && s.get("each").is_none()).cloned());
        let Some(source) = source else { return Vec::new() };
        let name = source.get("file").and_then(Value::as_str).unwrap_or_default();
        let list = source.get("list").and_then(Value::as_str).unwrap_or_default();
        file(&folder.join(format!("{name}.json")).to_string_lossy()).and_then(|whole| whole.get(list)?.as_array().cloned()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::Book;
    use crate::kept::Kept;

    #[test]
    fn phiegg_s_records_are_read_from_its_own_files_where_phiora_does_not_answer() {
        let book = Book::read(&Kept::none(), &["world", "club", "show"]);
        let count = |kind: &str| book.kind.iter().find(|(name, _)| name == kind).map_or(0, |(_, found)| found.len());
        assert!(count("world") > 100 && count("club") > 100 && count("show") > 100, "{:?}", book.kind.iter().map(|(k, f)| (k, f.len())).collect::<Vec<_>>());
        assert!(book.kind[0].1.iter().any(|found| found.name == "Opalia"));
        assert!(book.popular.contains(&"Once City".to_string()));
    }
}
