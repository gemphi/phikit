#[allow(unused_imports)]
use super::*;

use std::path::Path;

impl Quest {
    /// Ask until the budget runs out or the frontier empties — or the oracle
    /// cannot be reached, or fails [`MAX_FAILURES_IN_A_ROW`] times running.
    /// A word the oracle answers with nothing is data and does not stop the
    /// quest; an oracle that will not start, or keeps exiting with an error,
    /// is not, and asking it the rest of the budget would only write that
    /// many identical failure pages. [`Quest::stopped`] says why a run ended
    /// early.
    pub fn run(&mut self, oracle: &mut dyn Oracle) -> u32 {
        self.run_watching(oracle, |_, _| {})
    }

    /// [`Quest::run`], calling `after(quest, step)` once each answer is in —
    /// to print progress, or to write the quest so far, so that a slow model
    /// stopped half-way still leaves every answer it gave on disk.
    pub fn run_watching(&mut self, oracle: &mut dyn Oracle, mut after: impl FnMut(&Quest, &Step)) -> u32 {
        let mut asked = 0u32;
        let mut failed_in_a_row = 0u32;
        while asked < self.budget {
            let Some(step) = self.ask_one(oracle) else { break };
            asked += 1;
            after(self, &step);
            let text = step.answer.text();
            if !step.answer.is_said() && text.starts_with("oracle unreachable") {
                self.stopped = Some(format!("{text} — is the program installed and on PATH?"));
                break;
            }
            failed_in_a_row = match !step.answer.is_said() && text.starts_with("oracle failed") {
                true => failed_in_a_row + 1,
                false => 0,
            };
            if failed_in_a_row >= MAX_FAILURES_IN_A_ROW {
                self.stopped = Some(format!("{failed_in_a_row} failures in a row; the last: {text}"));
                break;
            }
        }
        asked
    }

    /// Write the quest out: one page and one table per step, plus an index.
    /// This is the auto-generated corpus — re-readable by a person, and
    /// re-ingestable by phiano as `word\tkind\tother\tweight`.
    pub fn write(&self, dir: &Path, oracle_name: &str) -> std::io::Result<()> {
        std::fs::create_dir_all(dir.join("steps"))?;
        let mut index = String::new();
        let mut tsv = String::new();
        index.push_str(&format!(
            "# Quest\n\n_oracle `{}` · {} questions · {} words · {} relation kinds · {} edges_\n\n\
             | # | word | depth | kept | opened |\n|---|---|---|---|---|\n",
            oracle_name,
            self.steps.len(),
            self.world.lexicon.len(),
            self.world.relations.len(),
            self.world.edges.len()
        ));
        for (i, step) in self.steps.iter().enumerate() {
            let name = format!("steps/{:03}-{}.md", i + 1, slug(&step.about));
            std::fs::write(dir.join(&name), step.to_markdown())?;
            tsv.push_str(&step.to_tsv());
            index.push_str(&format!(
                "| {} | [{}]({}) | {} | {} | {} |\n",
                i + 1,
                step.about,
                name,
                step.depth,
                step.kept.len(),
                step.opened.len()
            ));
        }
        std::fs::write(dir.join("relations.tsv"), tsv)?;
        if !self.sentences.is_empty() {
            let lines: String = self.sentences.iter().map(|(w, l)| format!("{w}\t{l}\n")).collect();
            std::fs::write(dir.join("sentences.tsv"), lines)?;
        }
        std::fs::write(dir.join("README.md"), index)?;
        Ok(())
    }
}

fn slug(s: &str) -> String {
    s.chars().map(|c| if c.is_alphanumeric() { c } else { '-' }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oracle() -> Scripted {
        Scripted::new(&[
            ("Word: cat", "is-a | animal | 0.9\nhas | fur | 0.8\ndoes | sleep | 0.6"),
            ("Word: animal", "is-a | thing | 0.7\nnot a relation line at all"),
            ("Word: fur", "part-of | cat | 0.9"),
            ("Word: sleep", "does | rest | 0.5"),
            ("Word: thing", "is-a | thing | 0.1"),
            ("Word: rest", "like | sleep | 0.4"),
        ])
    }

    #[test]
    fn a_quest_walks_outward_and_stops_at_its_budget() {
        let mut q = Quest::new("cat").with_depth(1).with_budget(10);
        let asked = q.run(&mut oracle());
        assert_eq!(asked, 4, "cat, then its three words, then depth 1 is spent");
        assert_eq!(q.asked()[0], "cat");
        assert!(q.world.lexicon.get("animal").is_some());

        // The graph, not the transcript, is the point: cat is-a animal.
        let is_a = q.world.relations.get("is-a").unwrap();
        let cat = q.world.lexicon.get("cat").unwrap();
        let animal = q.world.lexicon.get("animal").unwrap();
        assert_eq!(q.world.calls_for(cat, is_a).first().map(|(w, _)| *w), Some(animal));
        // And the two words moved toward each other on that channel alone.
        let p = q.world.read::<crate::phasor::Phasor>(cat).unwrap();
        let o = q.world.read::<crate::phasor::Phasor>(animal).unwrap();
        assert!(p.agreement(o, is_a).is_some());
    }

    #[test]
    fn watching_sees_every_answer_as_it_arrives() {
        let mut q = Quest::new("cat").with_depth(1).with_budget(10);
        let mut seen = Vec::new();
        let asked = q.run_watching(&mut oracle(), |q, s| seen.push((q.steps.len(), s.about.clone())));
        assert_eq!(seen.len() as u32, asked);
        assert_eq!(seen[0], (1, "cat".to_string()));
    }

    #[test]
    fn phi4s_first_answer_is_kept_whole() {
        let said = "is-a mammal 0.9  \nhas whiskers 0.8  \ndoes purr 0.7  \nlikes fish 0.6  \n\
                    has fur 0.8  \nmade-of cells 0.9  \nbelongs-to Felidae 0.8  \ncauses allergy 0.5";
        let mut q = Quest::new("cat").with_budget(1);
        q.run(&mut Scripted::new(&[("Word: cat", said)]));
        let kept = &q.steps[0].kept;
        assert_eq!(kept.len(), 8);
        assert!(kept.contains(&("like".to_string(), "fish".to_string(), 0.6)));
        assert!(q.world.relations.get("likes").is_none(), "no ninth kind for a plural");
    }

    #[test]
    fn a_silent_oracle_is_recorded_not_fatal() {
        let mut q = Quest::new("unknown-word").with_budget(3);
        let asked = q.run(&mut Scripted::new(&[]));
        assert_eq!(asked, 1);
        assert!(!q.steps[0].answer.is_said());
        assert!(q.steps[0].kept.is_empty());
    }
}
