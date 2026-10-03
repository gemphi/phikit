#[allow(unused_imports)]
use super::*;

/// An oracle with a hidden structure, so the loop can be measured before a
/// model is installed: `n` invented words in families of eight, each standing
/// in five relations decided by arithmetic on its index. It is not a language;
/// it is a *known* graph, which is what a recoverability measurement needs —
/// the quest walks it, writes `relations.tsv`, and phiano's ingestion is then
/// judged against the truth the oracle was built from, not against prose.
///
/// The kinds it uses are five of [`crate::relation::KIND_NAMES`], so a quest
/// declares nothing new.
pub struct Synthetic {
    n: usize,
    pub asked: Vec<String>,
}

/// The kinds the hidden structure is drawn under.
pub const SYNTHETIC_KINDS: [&str; 5] = ["is-a", "part-of", "near", "used-for", "unlike"];

impl Synthetic {
    /// `n` words — rounded up to a multiple of eight.
    pub fn new(n: usize) -> Self {
        Self { n: n.max(8).div_ceil(8) * 8, asked: Vec::new() }
    }

    /// The spelling of word `i`.
    pub fn word(i: usize) -> String {
        format!("w{i}")
    }

    /// The word a spelling names, if it is one of ours.
    pub fn index(spelling: &str) -> Option<usize> {
        spelling.strip_prefix('w').and_then(|s| s.parse().ok())
    }

    /// Every relation word `i` stands in: `(kind, other, weight)`.
    ///
    /// A family head (`f{k}`) is what the word *is*; `part-of` points across
    /// the vocabulary by a stride coprime to `n`; `near` is the next word in
    /// the family; `used-for` names one of five tools; `opposite` is the word
    /// half a vocabulary away.
    pub fn truth(&self, i: usize) -> Vec<(&'static str, String, f64)> {
        let n = self.n;
        let fam = i / 8;
        vec![
            ("is-a", format!("f{fam}"), 0.9),
            ("part-of", Self::word((i * 7 + 3) % n), 0.6),
            ("near", Self::word(fam * 8 + (i + 1) % 8), 0.7),
            ("used-for", format!("t{}", i % 5), 0.5),
            ("unlike", Self::word((i + n / 2) % n), 0.8),
        ]
    }

    /// The word the prompt asks about.
    fn subject(prompt: &str) -> Option<String> {
        prompt
            .lines()
            .find_map(|l| l.strip_prefix("Word:"))
            .map(|s| s.trim().to_string())
    }
}

impl Oracle for Synthetic {
    fn ask(&mut self, prompt: &str) -> Result<String, OracleError> {
        let about = Self::subject(prompt).ok_or(OracleError::Empty)?;
        self.asked.push(about.clone());
        // Family heads and tools are leaves: the structure says nothing about
        // them, and an oracle that says nothing is a recorded step.
        let Some(i) = Self::index(&about) else { return Err(OracleError::Empty) };
        if i >= self.n {
            return Err(OracleError::Empty);
        }
        let lines: Vec<String> = self
            .truth(i)
            .into_iter()
            .map(|(k, o, w)| format!("{k} | {o} | {w:.1}"))
            .collect();
        Ok(lines.join("\n"))
    }

    fn name(&self) -> String {
        format!("synthetic({})", self.n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walks_the_hidden_structure() {
        let mut oracle = Synthetic::new(64);
        let mut quest = Quest::new("w0").with_depth(4).with_budget(200);
        let asked = quest.run(&mut oracle);
        // Leaves (family heads, tools) are asked and answer nothing.
        let structure = quest.asked().iter().filter(|a| Synthetic::index(a).is_some()).count();
        assert!(asked as usize > structure && structure >= 32, "asked {asked}, {structure} in structure");
        let w = &quest.world;
        assert_eq!(w.edges.len(), structure * 5);
        for k in SYNTHETIC_KINDS {
            assert!(w.relations.iter().any(|(_, n)| n == k));
        }
    }
}
