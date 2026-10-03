//! A persona, written as a markdown file: who it is, what the quest asks for
//! it, and the laws it keeps.
//!
//! In Phient's world, a word (w) is nothing but what it relates to, under what
//! conditions (c). A persona is a set of conditions — the words it starts
//! from, the kinds of relation it cares about, the voice it speaks in, the
//! laws its answers must obey — so a persona file is a quest's plan. The
//! frontmatter is `key: value` lines between `---` fences; lists are comma
//! separated; an inverse pair is `has>part-of`. The body is for people.
//!
//! ```text
//! ---
//! name: Rustacean
//! voice: the way a patient Rust teacher explains it
//! seeds: ownership, borrow, trait
//! kinds: is-a, has, part-of, used-for, does
//! per_seed: 110
//! depth: 2
//! relations: 64
//! inherit: has, does, used-for
//! inverse: has>part-of
//! transitive: is-a, part-of
//! sources: data/rust_book_corpus.txt
//! ---
//! ```

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Persona {
    pub name: String,
    /// How the persona speaks: appended to the voice question.
    pub voice: String,
    pub seeds: Vec<String>,
    pub kinds: Vec<String>,
    pub per_seed: u32,
    pub depth: u32,
    /// Relations per word in the persona's relation model: the one size
    /// knob (docs/100). Default 64; any value, 128 and past it included.
    pub relations: u32,
    /// Kinds whose answers a word inherits from what it `is-a` (docs/97 §1).
    pub inherit: Vec<String>,
    /// `(k, k′)`: `w k v` holds exactly when `v k′ w` does.
    pub inverse: Vec<(String, String)>,
    /// Kinds that chain: `a k b`, `b k c` ⇒ `a k c`.
    pub transitive: Vec<String>,
    /// Corpora and code the persona also reads, relative to phiano.
    pub sources: Vec<String>,
    /// Everything after the frontmatter.
    pub body: String,
}

impl Persona {
    /// Parse a persona file; the error names the missing or malformed field.
    pub fn from_md(text: &str) -> Result<Persona, String> {
        let text = text.trim_start_matches('\u{feff}');
        let rest = text.trim_start().strip_prefix("---").ok_or("a persona file starts with a --- line")?;
        let (front, body) = rest.split_once("\n---").ok_or("the frontmatter needs a closing --- line")?;
        let mut f: HashMap<String, String> = HashMap::new();
        for line in front.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
            let (k, v) = line.split_once(':').ok_or_else(|| format!("not `key: value`: {line}"))?;
            f.insert(k.trim().to_lowercase(), v.trim().to_string());
        }
        let list = |k: &str| -> Vec<String> {
            f.get(k).map_or(Vec::new(), |v| v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        };
        let num = |k: &str, d: u32| -> Result<u32, String> {
            f.get(k).map_or(Ok(d), |v| v.parse().map_err(|_| format!("{k} must be a whole number, not {v}")))
        };
        let name = f.get("name").cloned().filter(|s| !s.is_empty()).ok_or("name is missing")?;
        let seeds = list("seeds");
        let kinds = list("kinds");
        if seeds.is_empty() || kinds.is_empty() {
            return Err(format!("{name}: seeds and kinds are both needed"));
        }
        let mut inverse = Vec::new();
        for pair in list("inverse") {
            let (a, b) = pair.split_once('>').ok_or_else(|| format!("inverse pairs are written a>b, not {pair}"))?;
            inverse.push((a.trim().to_string(), b.trim().to_string()));
        }
        Ok(Persona {
            voice: f.get("voice").cloned().unwrap_or_default(),
            per_seed: num("per_seed", 110)?,
            depth: num("depth", 2)?,
            relations: num("relations", 64)?,
            inherit: list("inherit"),
            transitive: list("transitive"),
            sources: list("sources"),
            body: body.trim_start_matches(|c| c == '-').trim().to_string(),
            name,
            seeds,
            kinds,
            inverse,
        })
    }

    /// A file-system name: lower case, dashes.
    pub fn slug(&self) -> String {
        self.name.to_lowercase().chars().map(|c| if c.is_alphanumeric() { c } else { '-' }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RUST: &str = "---\nname: Rustacean\nvoice: the way a patient Rust teacher explains it\nseeds: ownership, borrow\nkinds: is-a, has, used-for\nper_seed: 40\ninherit: has, used-for\ninverse: has>part-of\ntransitive: is-a\n---\n\n# Rustacean\n\nWho it is.\n";

    #[test]
    fn a_persona_file_is_a_quest_plan() {
        let p = Persona::from_md(RUST).unwrap();
        assert_eq!(p.name, "Rustacean");
        assert_eq!(p.seeds, vec!["ownership", "borrow"]);
        assert_eq!(p.kinds, vec!["is-a", "has", "used-for"]);
        assert_eq!((p.per_seed, p.depth, p.relations), (40, 2, 64));
        let wide = Persona::from_md(&RUST.replace("per_seed: 40", "per_seed: 40\nrelations: 128")).unwrap();
        assert_eq!(wide.relations, 128);
        assert_eq!(p.inverse, vec![("has".to_string(), "part-of".to_string())]);
        assert!(p.body.starts_with("# Rustacean"));
        assert_eq!(p.slug(), "rustacean");
    }

    #[test]
    fn a_broken_file_says_what_is_wrong() {
        assert!(Persona::from_md("name: x").unwrap_err().contains("starts with"));
        assert!(Persona::from_md("---\nname: x\nseeds: a\n---\n").unwrap_err().contains("kinds"));
        assert!(Persona::from_md("---\nname: x\nseeds: a\nkinds: b\nper_seed: many\n---\n").unwrap_err().contains("whole number"));
        assert!(Persona::from_md("---\nname: x\nseeds: a\nkinds: b\ninverse: has part-of\n---\n").unwrap_err().contains("a>b"));
    }
}
