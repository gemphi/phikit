//! The records search finds that are not Phiogy's own: Phiegg's worlds,
//! clubs, shows (`data/search.tsv` says which), the very records Phiegg
//! reads. Asked of Phiora where it gave the site; else read from
//! Phiegg's own files by the manifest Phiora seeds from (`tree.json`).
//! And what is sought most often, from Phiegg's site file.

use std::path::Path;

use serde_json::Value;

use crate::data;
use crate::kept::Kept;

/// A record found: its kind, its id, its name, its line, its picture by
/// its key (its own, else its world's; none where it has neither), and
/// the record whole, for its page.
#[derive(Debug, Clone, PartialEq)]
pub struct Found {
    pub kind: String,
    pub id: String,
    pub name: String,
    pub line: String,
    pub picture: String,
    pub row: Value,
}

/// Every record of each kind search asks for, what is sought most, and
/// the kinds Phiora gave (whose ids are its keys).
#[derive(Debug, Clone, Default)]
pub struct Book {
    pub kind: Vec<(String, Vec<Found>)>,
    pub popular: Vec<String>,
    pub keyed: Vec<String>,
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
        let phiora = |kind: &str| asked.as_ref().and_then(|tree| tree.get("sources")?.as_array()?.iter().find(|s| s.get("kind").and_then(Value::as_str) == Some(kind))?.get("rows")?.as_array().cloned());
        let rows = |kind: &str| -> Vec<Value> { phiora(kind).unwrap_or_else(|| Book::files(kind)) };
        let given: Vec<String> = kinds.iter().filter(|kind| phiora(kind).is_some()).map(|kind| kind.to_string()).collect();
        let words = |row: &str| cells(row).iter().flat_map(|cell| cell.split_whitespace()).collect::<Vec<_>>();
        let (name, line, picture, borrow) = (words("name"), words("line"), words("picture"), words("borrow"));
        let word = |row: &Value, fields: &[&str]| fields.iter().find_map(|field| row.get(*field).and_then(Value::as_str).filter(|text| !text.is_empty())).unwrap_or_default().to_string();
        // A picture's key: the table's start, then its path less what the
        // table says to leave off it.
        let (key, strip) = (cells("key").first().copied().unwrap_or_default(), cells("strip").first().copied().unwrap_or_default());
        let keyed = |path: String| match path.is_empty() {
            true => path,
            false => format!("{key}{}", path.strip_prefix(strip).unwrap_or(&path)),
        };
        // The worlds' pictures, which a record with none of its own wears.
        let world: Vec<(String, String)> = rows(cells("world").first().copied().unwrap_or("world")).iter().map(|row| (word(row, &["id"]), keyed(word(row, &picture)))).collect();
        let worn = |row: &Value| borrow.iter().find_map(|field| match row.get(*field)? {
            Value::String(id) => Some(id.clone()),
            Value::Array(list) => list.first()?.as_str().map(str::to_string),
            _ => None,
        }).and_then(|id| world.iter().find(|(known, _)| *known == id).map(|(_, picture)| picture.clone())).unwrap_or_default();
        let found = |kind: &str, row: &Value| {
            let own = keyed(word(row, &picture));
            Found { kind: kind.to_string(), id: word(row, &["id"]), name: word(row, &name), line: word(row, &line), picture: if own.is_empty() { worn(row) } else { own }, row: row.clone() }
        };
        // A record with no name (a sale) is kept for the profile's trail; search never finds it.
        let kind = kinds.iter().map(|kind| (kind.to_string(), rows(kind).iter().map(|row| found(kind, row)).filter(|found| !found.id.is_empty()).collect())).collect();
        let popular = cells("popular");
        let popular = popular.first().and_then(|path| file(path)).and_then(|site| site.get(*popular.get(1)?)?.as_array().cloned()).unwrap_or_default();
        Book { kind, popular: popular.iter().filter_map(Value::as_str).map(str::to_string).collect(), keyed: given }
    }

    /// The kinds of Phiora's records search's tabs find.
    pub fn sought() -> Vec<String> {
        data::row(data::SEARCH).into_iter().filter(|row| row.first() == Some(&"tab")).filter_map(|row| row.get(3)?.strip_prefix("tree:").map(str::to_string)).collect()
    }

    /// Every kind read: those search finds, and those a person's profile
    /// shows (`data/profile.tsv`: its tabs' kinds and a person's own).
    pub fn kinds() -> Vec<String> {
        let profile = data::row(data::PROFILE).into_iter().filter_map(|row| match row.first() {
            Some(&"tab") => row.get(4).filter(|kind| !kind.is_empty()).map(|kind| kind.to_string()),
            Some(&"own") => row.get(1).map(|kind| kind.to_string()),
            _ => None,
        });
        let named = data::row(data::PROFILE).into_iter().filter(|row| row.first() == Some(&"name")).flat_map(|row| row.get(1).map(|kinds| kinds.split_whitespace().map(str::to_string).collect::<Vec<_>>()).unwrap_or_default());
        let profile = profile.chain(named);
        let mut every = Book::sought();
        profile.for_each(|kind| if !every.contains(&kind) { every.push(kind) });
        every
    }

    /// A record's id as this book has it: Phiegg's own, or, for a kind
    /// Phiora gave, Phiora's key for it (the md5 of `kind:id`).
    pub fn key(&self, kind: &str, id: &str) -> String {
        use md5::{Digest, Md5};
        match self.keyed.iter().any(|known| known == kind) {
            true => format!("{:x}", Md5::digest(format!("{kind}:{id}").as_bytes())),
            false => id.to_string(),
        }
    }

    /// The records of a kind.
    pub fn of(&self, kind: &str) -> &[Found] {
        self.kind.iter().find(|(name, _)| name == kind).map(|(_, found)| found.as_slice()).unwrap_or_default()
    }

    /// The record of any kind read that has that id.
    pub fn named(&self, id: &str) -> Option<&Found> {
        self.kind.iter().flat_map(|(_, found)| found).find(|found| found.id == id)
    }

    /// A record by its kind and id.
    pub fn record(&self, kind: &str, id: &str) -> Option<&Found> {
        self.kind.iter().find(|(name, _)| name == kind)?.1.iter().find(|found| found.id == id)
    }

    /// The records related to one: those that name it, and those it names,
    /// by the fields `link` lists (its world, its club).
    pub fn related(&self, one: &Found) -> Vec<&Found> {
        let field = cells("link").iter().flat_map(|cell| cell.split_whitespace()).collect::<Vec<_>>();
        let names = |row: &Value, id: &str| field.iter().any(|field| match row.get(*field) {
            Some(Value::String(own)) => own == id,
            Some(Value::Array(list)) => list.iter().any(|own| own.as_str() == Some(id)),
            _ => false,
        });
        let sought = Book::sought();
        let every = self.kind.iter().filter(|(kind, _)| sought.contains(kind)).flat_map(|(_, found)| found);
        every.filter(|other| other.id != one.id && (names(&other.row, &one.id) || names(&one.row, &other.id))).collect()
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
        // A world has its own picture, a club and a show wear their world's.
        let of = |kind: &str| book.kind.iter().find(|(name, _)| name == kind).map(|(_, found)| found.clone()).unwrap_or_default();
        assert!(of("world").iter().any(|found| found.name == "Opalia" && found.picture.starts_with("phiegg:worlds/")));
        assert!(of("club").iter().filter(|found| !found.picture.is_empty()).count() > 100 && of("show").iter().any(|found| !found.picture.is_empty()));
    }

    #[test]
    fn a_record_is_found_by_its_phiegg_id_where_phiora_keyed_it() {
        // Phiora's key for a record is the md5 of its kind and its id.
        let keyed = Book { keyed: vec!["person".to_string()], ..Book::default() };
        assert_eq!(keyed.key("person", "u_noor"), "b551533bfd7df0be9bfc4880c03eb288");
        // A kind read from Phiegg's own files keeps Phiegg's id.
        assert_eq!(keyed.key("club", "c_opal"), "c_opal");
    }
}
