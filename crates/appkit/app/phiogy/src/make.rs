//! Making clubs and PFTs, and publishing them on Phiora, which is what
//! Phiegg's web reads. A sheet of rows, one record to a row, is read
//! (`rows`) and each is made on Phiora or, where Phiora already has a
//! record of that `ref`, changed to match it (`publish`): the same sheet
//! run again makes nothing twice. A guest cannot publish, as Phiora
//! says; the person signs in first. What a sheet is, the kinds it may
//! make and every word said are `data/make.tsv`.
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use appkit::service::PTable;
use serde_json::{Map, Value, json};

use crate::kept::Kept;
use crate::phiora::Verb;

/// How many rows of a kind are asked for one by one; more, and the list is asked.
const FEW: usize = 20;

/// One record a sheet asks for.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// Where it is in the sheet, counting from 1.
    pub line: usize,
    pub kind: String,
    pub name: String,
    /// Every field, `ref` among them.
    pub fields: Map<String, Value>,
}

/// What the cells of a column are.
#[derive(Clone, Copy, PartialEq)]
enum How {
    Words,
    Number,
    Bool,
    List,
}

/// A column of the header: the field it is, the fields it is inside,
/// and what its cells are.
struct Column {
    path: Vec<String>,
    how: How,
}

/// What the end of a column's name says its cells are, and the name
/// without it.
fn column(head: &str, word: &PTable) -> Column {
    let head = head.trim();
    let ends = [(word.get("suffix.number"), How::Number), (word.get("suffix.bool"), How::Bool), (word.get("suffix.list"), How::List), (word.get("suffix.words"), How::Words)];
    let (name, how) = ends.iter().find_map(|(end, how)| head.strip_suffix(end).map(|name| (name, *how))).unwrap_or((head, How::Words));
    Column { path: name.split(word.get("nest")).map(str::to_string).collect(), how }
}

/// A cell as the value it is.
fn value(cell: &str, how: How, word: &PTable) -> Result<Value, String> {
    match how {
        How::Words => Ok(Value::String(cell.to_string())),
        How::List => Ok(Value::Array(cell.split(word.get("split.list")).map(|item| Value::String(item.trim().to_string())).filter(|item| item.as_str() != Some("")).collect())),
        How::Number => cell.parse::<i64>().map(Value::from).or_else(|_| cell.parse::<f64>().ok().filter(|number| number.is_finite()).map(|number| json!(number)).ok_or(())).map_err(|_| word.fill("why.number", "value", cell)),
        How::Bool => match cell {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ => Err(word.fill("why.bool", "value", cell)),
        },
    }
}

/// Put a value inside the fields, at the end of a path of names.
fn put(into: &mut Map<String, Value>, path: &[String], value: Value) {
    match path {
        [] => {}
        [name] => _ = into.insert(name.clone(), value),
        [name, rest @ ..] => {
            let inner = into.entry(name.clone()).or_insert_with(|| Value::Object(Map::new()));
            if !inner.is_object() {
                *inner = Value::Object(Map::new());
            }
            if let Value::Object(inner) = inner {
                put(inner, rest, value);
            }
        }
    }
}

/// The rows of a sheet, and what is wrong with the ones that are not
/// rows (a line number, then why). A row with a problem is left out.
pub fn rows(sheet: &str, word: &PTable) -> (Vec<Row>, Vec<String>) {
    let (mut rows, mut bad) = (Vec::new(), Vec::new());
    let mut lines = sheet.lines().enumerate().map(|(at, line)| (at + 1, line.trim_end_matches('\r'))).filter(|(_, line)| !line.trim().is_empty() && !line.starts_with('#'));
    let Some((_, head)) = lines.next() else { return (rows, bad) };
    let head: Vec<&str> = head.split('\t').map(str::trim).collect();
    let (kind, name) = (word.get("column.kind"), word.get("column.ref"));
    let (Some(at_kind), Some(at_ref)) = (head.iter().position(|one| *one == kind), head.iter().position(|one| *one == name)) else {
        bad.push(word.fill("say.bad", "line", "1").replace("{why}", word.get("why.header")));
        return (rows, bad);
    };
    let columns: Vec<Column> = head.iter().map(|one| column(one, word)).collect();
    let kinds: HashSet<&str> = word.get("kinds").split_whitespace().collect();
    let mut seen = HashSet::new();
    for (line, text) in lines {
        let cells: Vec<&str> = text.split('\t').map(str::trim).collect();
        let why = (|| {
            if cells.len() > columns.len() && cells[columns.len()..].iter().any(|cell| !cell.is_empty()) {
                return Err(word.get("why.cells").to_string());
            }
            let cell = |at: usize| cells.get(at).copied().unwrap_or_default();
            let (kind, name) = (cell(at_kind), cell(at_ref));
            if !kinds.contains(kind) {
                return Err(word.fill("why.kind", "kind", kind));
            }
            if name.is_empty() {
                return Err(word.get("why.ref").to_string());
            }
            if !seen.insert((kind.to_string(), name.to_string())) {
                return Err(word.fill("why.twice", "ref", name));
            }
            let mut fields = Map::new();
            for (at, one) in columns.iter().enumerate() {
                let cell = cell(at);
                if !cell.is_empty() {
                    put(&mut fields, &one.path, value(cell, one.how, word)?);
                }
            }
            // `kind` says what is made; it is not a field of it.
            fields.remove(word.get("column.kind"));
            Ok(Row { line, kind: kind.to_string(), name: name.to_string(), fields })
        })();
        match why {
            Ok(row) => rows.push(row),
            Err(why) => bad.push(word.fill("say.bad", "line", &line.to_string()).replace("{why}", &why)),
        }
    }
    (rows, bad)
}

/// What a run did, as lines to say.
#[derive(Debug, Default)]
pub struct Done {
    pub said: Vec<String>,
    pub made: usize,
    pub changed: usize,
    pub same: usize,
    pub refused: usize,
}

/// Every string of a value that names another row, replaced by that
/// record's id. The name of the one that is nobody's, when there is
/// one.
fn linked(value: &mut Value, mark: &str, ids: &HashMap<String, String>) -> Result<(), String> {
    match value {
        Value::String(text) => {
            if let Some(name) = text.strip_prefix(mark) {
                *text = ids.get(name).cloned().ok_or_else(|| name.to_string())?;
            }
            Ok(())
        }
        Value::Array(all) => all.iter_mut().try_for_each(|one| linked(one, mark, ids)),
        Value::Object(all) => all.values_mut().try_for_each(|one| linked(one, mark, ids)),
        _ => Ok(()),
    }
}

/// What Phiora has of a kind, by `ref`: each record's id and the record.
type Had = HashMap<String, (String, Map<String, Value>)>;

/// A text as a part of an address: what is not a letter, a digit or one of `/-_.` is written `%XX`.
pub fn enc(text: &str) -> String {
    text.bytes().map(|byte| if byte.is_ascii_alphanumeric() || b"/-_.".contains(&byte) { (byte as char).to_string() } else { format!("%{byte:02X}") }).collect()
}

/// What Phiora already has of a kind, by `ref`: its id and the record. Of every
/// record of the kind, or, given the refs, of those only (one ask each): a few
/// records are not worth the list of all there are.
fn had(kept: &Kept, kind: &str, word: &PTable, only: Option<&[&str]>) -> Option<Had> {
    let list = word.fill("list", "kind", kind);
    let paths: Vec<String> = match only {
        Some(refs) => refs.iter().map(|one| format!("{list}&where={}:{}", word.get("column.ref"), enc(one))).collect(),
        None => vec![list],
    };
    let mut all = Had::new();
    for path in paths {
        let tree: Value = serde_json::from_str(&kept.read(&path)?).ok()?;
        let sources = tree.get(word.get("answer.sources")).and_then(Value::as_array)?;
        let rows = sources.iter().filter(|source| source.get(word.get("answer.kind")).and_then(Value::as_str) == Some(kind)).filter_map(|source| source.get(word.get("answer.rows")).and_then(Value::as_array));
        let one = |row: &Value| {
            let record = row.as_object()?;
            Some((record.get(word.get("column.ref"))?.as_str()?.to_string(), (record.get(word.get("id"))?.as_str()?.to_string(), record.clone())))
        };
        all.extend(rows.flatten().filter_map(one));
    }
    Some(all)
}
/// Publish a sheet's rows on Phiora, in order, so a club is there for
/// the PFTs that name it. With `dry` nothing is written and each line
/// says what would be. `Err` is why nothing was tried: nobody signed
/// in, or Phiora not answering.
pub fn publish(kept: &Kept, rows: &[Row], dry: bool, word: &PTable) -> Result<Done, String> {
    if kept.me().is_null() {
        return Err(word.get("say.nowhere").to_string());
    }
    if kept.guest() {
        return Err(word.get("say.nobody").to_string());
    }
    let mut on = HashMap::new();
    let mut ids: HashMap<String, String> = HashMap::new();
    for kind in rows.iter().map(|row| row.kind.as_str()).collect::<HashSet<_>>() {
        // A few rows ask for those records; many ask for the list.
        let refs: Vec<&str> = rows.iter().filter(|row| row.kind == kind).map(|row| row.name.as_str()).collect();
        let only = Some(refs.as_slice()).filter(|refs| refs.len() <= FEW);
        let all = had(kept, kind, word, only).ok_or_else(|| word.get("say.nowhere").to_string())?;
        ids.extend(all.iter().map(|(name, (id, _))| (name.clone(), id.clone())));
        on.insert(kind.to_string(), all);
    }
    let (wave, pause) = (word.get("wave").parse::<usize>().unwrap_or(28), Duration::from_millis(word.get("pause").parse().unwrap_or(61000)));
    let (mut done, mut written) = (Done::default(), 0);
    for row in rows {
        let mut fields = row.fields.clone();
        let mut mine = Value::Object(std::mem::take(&mut fields));
        let said = |name: &str| word.fill(name, "kind", &row.kind).replace("{ref}", &row.name);
        if let Err(name) = linked(&mut mine, word.get("link"), &ids) {
            done.refused += 1;
            done.said.push(said("say.refused").replace("{why}", &word.fill("why.link", "name", &name)));
            continue;
        }
        let Value::Object(fields) = mine else { continue };
        match on.get(&row.kind).and_then(|all| all.get(&row.name)) {
            Some((id, record)) => {
                let set: Map<String, Value> = fields.iter().filter(|(name, value)| record.get(*name) != Some(*value)).map(|(name, value)| (name.clone(), value.clone())).collect();
                if set.is_empty() {
                    done.same += 1;
                    done.said.push(said("say.same"));
                    continue;
                }
                let names = set.keys().cloned().collect::<Vec<_>>().join(", ");
                if dry {
                    done.changed += 1;
                    done.said.push(said("say.would.change").replace("{fields}", &names));
                    continue;
                }
                written = pace(written, wave, pause, &mut done);
                let path = word.get("item").replace("{kind}", &row.kind).replace("{id}", id);
                match kept.record(Verb::Patch, &path, &json!({ "set": set }).to_string()) {
                    Some((true, _)) => {
                        done.changed += 1;
                        done.said.push(said("say.changed").replace("{fields}", &names));
                    }
                    other => refuse(&mut done, &said("say.refused"), other),
                }
            }
            None => {
                if dry {
                    done.made += 1;
                    done.said.push(said("say.would"));
                    ids.insert(row.name.clone(), format!("{}{}", word.get("link"), row.name));
                    continue;
                }
                written = pace(written, wave, pause, &mut done);
                match kept.record(Verb::Post, word.get("tree"), &json!({ "kind": row.kind, "fields": fields }).to_string()) {
                    Some((true, text)) => {
                        let item = serde_json::from_str::<Value>(&text).ok().and_then(|answer| answer.get(word.get("answer.item")).cloned());
                        let id = item.as_ref().and_then(|item| item.get(word.get("id"))).and_then(Value::as_str).unwrap_or_default().to_string();
                        ids.insert(row.name.clone(), id.clone());
                        done.made += 1;
                        done.said.push(said("say.made").replace("{id}", &id));
                    }
                    other => refuse(&mut done, &said("say.refused"), other),
                }
            }
        }
    }
    done.said.push(word.fill("say.done", "made", &done.made.to_string()).replace("{changed}", &done.changed.to_string()).replace("{same}", &done.same.to_string()).replace("{refused}", &done.refused.to_string()));
    Ok(done)
}

/// Count a write; after a wave of them, wait out Phiora's limit.
fn pace(written: usize, wave: usize, pause: Duration, done: &mut Done) -> usize {
    if written > 0 && written.is_multiple_of(wave) {
        done.said.push(PTable::read(crate::data::MAKE).get("say.wait").to_string());
        std::thread::sleep(pause);
    }
    written + 1
}

/// A write Phiora did not take: its own words if it said any.
fn refuse(done: &mut Done, line: &str, answer: Option<(bool, String)>) {
    let word = PTable::read(crate::data::MAKE);
    let why = match answer {
        Some((_, text)) => serde_json::from_str::<Value>(&text).ok().and_then(|said| said.get(PTable::read(crate::data::MAKE).get("answer.message")).and_then(Value::as_str).map(str::to_string)).unwrap_or(text),
        None => word.get("say.nowhere").to_string(),
    };
    done.refused += 1;
    done.said.push(line.replace("{why}", &why));
}
