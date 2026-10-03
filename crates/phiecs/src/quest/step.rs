#[allow(unused_imports)]
use super::*;

/// What one question did to the world.
#[derive(Clone, Debug)]
pub struct Step {
    /// The word asked about.
    pub about: String,
    /// How far from the seed it was.
    pub depth: u32,
    /// Which template produced the prompt.
    pub template: &'static str,
    /// The prompt exactly as it was sent: a transcript records what was asked
    /// then, not what the template would ask now.
    pub prompt: String,
    /// What the oracle said, verbatim — a transcript is evidence.
    pub answer: Answer,
    /// The relations that survived parsing and were written into the world.
    pub kept: Vec<(String, String, f64)>,
    /// Words this step put on the frontier.
    pub opened: Vec<String>,
}

/// An oracle's reply, or the reason there wasn't one.
#[derive(Clone, Debug)]
pub enum Answer {
    Said(String),
    Nothing(String),
}

impl Answer {
    pub fn text(&self) -> &str {
        match self {
            Answer::Said(s) => s,
            Answer::Nothing(s) => s,
        }
    }

    pub fn is_said(&self) -> bool {
        matches!(self, Answer::Said(_))
    }
}

impl Step {
    /// The step as a page: the prompt's subject, what was said, what was kept.
    /// One file per step is what makes a quest re-readable — and re-runnable
    /// against a different oracle for comparison.
    pub fn to_markdown(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("# {}\n\n", self.about));
        s.push_str(&format!(
            "_depth {} · template `{}` · {} relations kept · {} words opened_\n\n",
            self.depth,
            self.template,
            self.kept.len(),
            self.opened.len()
        ));
        s.push_str("## Asked\n\n```\n");
        s.push_str(self.prompt.trim());
        s.push_str("\n```\n\n## Answered\n\n```\n");
        s.push_str(self.answer.text().trim());
        s.push_str("\n```\n\n## Kept\n\n");
        if self.kept.is_empty() {
            s.push_str("Nothing parsed from this answer.\n");
        } else {
            s.push_str("| kind | word | weight |\n|---|---|---|\n");
            for (k, o, w) in &self.kept {
                s.push_str(&format!("| {k} | {o} | {w:.2} |\n"));
            }
        }
        s
    }

    /// The same step as rows a program reads: `about\tkind\tother\tweight`.
    pub fn to_tsv(&self) -> String {
        self.kept
            .iter()
            .map(|(k, o, w)| format!("{}\t{}\t{}\t{:.3}\n", self.about, k, o, w))
            .collect()
    }
}

/// Which words a step opened, given what it kept.
pub(crate) fn opened(kept: &[(String, String, f64)], asked: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (_, other, _) in kept {
        if !asked.iter().any(|a| a == other) && !out.contains(other) {
            out.push(other.clone());
        }
    }
    out
}
