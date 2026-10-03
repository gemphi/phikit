//! Reading a model's answer: `kind | other | weight` lines out of whatever
//! the model wrapped them in.
#[allow(unused_imports)]
use super::*;

/// One parsed line of an answer.
#[derive(Clone, PartialEq, Debug)]
pub struct Claim {
    pub kind: String,
    pub other: String,
    pub weight: f64,
}

/// Parse `kind | other | weight` lines, skipping anything that is not one.
/// `offered` are the kinds the prompt named (empty: any kind).
///
/// Models add preambles, bullets, numbering, code fences, markdown tables,
/// bold, and apologies. The parser strips what surrounds a relation and drops
/// every line that is not one, rather than the quest asking twice:
///
/// - a line's outer pipes are removed, so a markdown table row reads like a
///   plain one, and separator rows (`|---|---|`) have no letters and go;
/// - a leading bullet or list number (`-`, `*`, `•`, `1.`, `2)`) is removed,
///   so numbering cannot invent a kind per line;
/// - a kind is normalised to lower-case words joined by `-` (`Used For`,
///   `used_for` → `used-for`), and a kind one `s` away from an offered one is
///   that one (`likes` → `like`), so one relation keeps one channel;
/// - a weight is the number a third column starts with (`0.9 (strong)` →
///   0.9); a third column with no number is a header, and the line goes;
/// - an `other` of two or three words is one compound (`pet store` →
///   `pet-store`); four or more is a phrase, and the line goes;
/// - a line with no pipes is read by tabs, and failing that by spaces when it
///   ends in a number — Phi-4's own shape on 2026-09-28 (`is-a mammal 0.9`,
///   `does give birth 0.7`): the kind is the longest offered one the line
///   starts with, the rest is the other.
///
/// Two more of Phi-4's shapes are read too: labelled fields
/// (`kind: is-a, other word: aquatic, weight: 0.9`), and a relation stacked
/// one field per line — read only when no line of the answer is a relation.
pub fn parse_relations(answer: &str, offered: &[String]) -> Vec<Claim> {
    let first = parse_lines(answer.lines(), offered);
    match first.is_empty() {
        true => parse_lines(unstack(answer).iter().map(String::as_str), offered),
        false => first,
    }
}

/// Lines holding one field each, in threes ending in a number, joined back
/// into `kind | other | weight`.
fn unstack(answer: &str) -> Vec<String> {
    let fields: Vec<&str> = answer.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    fields
        .chunks(3)
        .filter(|t| t.len() == 3 && leading_number(t[2]).is_some() && leading_number(t[1]).is_none())
        .map(|t| t.join(" | "))
        .collect()
}

fn parse_lines<'a>(lines: impl Iterator<Item = &'a str>, offered: &[String]) -> Vec<Claim> {
    let mut out = Vec::new();
    for line in lines {
        let line = strip_list_marker(line.trim().trim_matches('|').trim());
        let parts = columns(line, offered);
        if parts.len() < 2 {
            continue;
        }
        let kind = offered_kind(&kind_name(&parts[0]), offered);
        // A kind the prompt did not offer is dropped (Phi-4, 27 questions:
        // 15 such names — `protects`, `found-in`, `similar-to` — 20 rows of
        // 248, each alone on its channel with nothing to learn from).
        if !offered.is_empty() && !offered.contains(&kind) && !glued(&kind, &parts[1], offered) {
            continue;
        }
        let other = clean(&parts[1]);
        let words = other.split_whitespace().count();
        if !kind.chars().any(|c| c.is_alphabetic()) || words == 0 || words > 3 {
            continue;
        }
        let other = other.split_whitespace().collect::<Vec<_>>().join("-");
        // `has-bark | bark`: an offered kind with its answer glued on.
        let kind = match kind.strip_suffix(&format!("-{other}")) {
            Some(base) if offered.iter().any(|o| o == base) => base.to_string(),
            _ => kind,
        };
        let weight = match parts.get(2).filter(|w| !w.is_empty()) {
            None => 1.0,
            Some(w) => match leading_number(w) {
                Some(x) => x.clamp(0.0, 1.0),
                None => continue,
            },
        };
        if weight == 0.0 {
            continue;
        }
        out.push(Claim { kind, other, weight });
    }
    out
}

/// A word is not its own relation: `cat | cat`, or Phi-4's
/// `mammal | mammal | 1.0` when asked about cat, which names a kind by its
/// answer. Both are dropped.
pub(crate) fn is_self_edge(c: &Claim, about: &str) -> bool {
    c.other == about || c.kind == c.other
}

/// The offered kind a model meant when it bent the name by one `s`
/// (`likes` → `like`, `cause` → `causes`) or lost a dash (`belongsto`);
/// any other name is returned as the model wrote it.
pub fn offered_kind(kind: &str, offered: &[String]) -> String {
    // Phi-4's own synonyms for offered kinds (full `cat` quest, 64 questions).
    const SAME: [(&str, &str); 4] = [("opposite", "unlike"), ("opposite-of", "unlike"), ("has-a", "has"), ("similar-to", "like")];
    let kind = SAME.iter().find(|(a, b)| *a == kind && offered.iter().any(|o| o == b)).map_or(kind, |(_, b)| *b);
    if let Some(o) = offered.iter().find(|o| o.replace('-', "") == kind.replace('-', "")) {
        return o.clone();
    }
    let is = |k: &str| offered.iter().any(|o| o == k);
    let singular = kind.strip_suffix('s').filter(|k| is(k));
    let plural = format!("{kind}s");
    match (is(kind), singular) {
        (true, _) => kind.to_string(),
        (false, Some(k)) => k.to_string(),
        (false, None) if is(&plural) => plural,
        _ => kind.to_string(),
    }
}

/// `has-bark | bark`: an offered kind with its answer glued on.
fn glued(kind: &str, other: &str, offered: &[String]) -> bool {
    let other = clean(other).split_whitespace().collect::<Vec<_>>().join("-");
    kind.strip_suffix(&format!("-{other}")).is_some_and(|b| offered.iter().any(|o| o == b))
}

/// A line's columns: by `|`, else by tab, else by spaces when the last word
/// is a weight. Spaced, the kind is the longest offered kind (two words or
/// one) the line starts with; with none offered to match, only
/// `kind other weight` is read. Anything else is prose and has none.
fn columns(line: &str, offered: &[String]) -> Vec<String> {
    let by = |sep: char| line.split(sep).map(|p| p.trim().to_string()).collect::<Vec<_>>();
    if line.contains('|') {
        return by('|');
    }
    if line.to_lowercase().starts_with("kind:") {
        return line.split(',').map(|f| f.split_once(':').map_or(f, |(_, v)| v).trim().to_string()).collect();
    }
    if line.contains('\t') {
        return by('\t');
    }
    let w: Vec<&str> = line.split_whitespace().collect();
    let Some((weight, middle)) = w.split_last().filter(|(x, _)| leading_number(x).is_some()) else {
        return Vec::new();
    };
    let offers = |k: &[&str]| offered.iter().any(|o| *o == offered_kind(&kind_name(&k.join(" ")), offered));
    let split = (1..=2.min(middle.len().saturating_sub(1))).rev().find(|&k| offers(&middle[..k]));
    match (split, middle.len()) {
        (Some(k), _) => vec![middle[..k].join(" "), middle[k..].join(" "), weight.to_string()],
        (None, 2) => vec![middle[0].into(), middle[1].into(), weight.to_string()],
        _ => Vec::new(),
    }
}

/// Remove a leading bullet or list number: `- `, `* `, `• `, `1. `, `12) `.
fn strip_list_marker(s: &str) -> &str {
    let s = s.trim_start_matches(['-', '*', '•']).trim_start();
    let digits = s.chars().take_while(|c| c.is_ascii_digit()).count();
    match digits > 0 && s[digits..].starts_with(['.', ')']) {
        true => s[digits + 1..].trim_start(),
        false => s,
    }
}

/// A kind as lower-case words joined by `-`.
fn kind_name(s: &str) -> String {
    clean(&s.replace('_', " ")).split_whitespace().collect::<Vec<_>>().join("-")
}

/// The number a string starts with, if it starts with one.
fn leading_number(s: &str) -> Option<f64> {
    let end = s.char_indices().find(|(_, c)| !(c.is_ascii_digit() || *c == '.')).map(|(i, _)| i).unwrap_or(s.len());
    s[..end].parse().ok()
}

/// Lower case, letters, digits, dashes and spaces only.
fn clean(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == ' ')
        .collect::<String>()
        .trim()
        .to_string()
}
