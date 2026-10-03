//! One kind at a time: a quest shaped for laws.
//!
//! A law `k₁ ⇒ k₂` (phinet `relate::find_laws`) can only be counted on a told
//! `w k₁ v` when both `w` and `v` were asked `k₂`. The first full Phi-4 quest
//! asked 64 of 442 words, so each kind pair had 11–19 testable triples and no
//! law reached 80% (phiano docs/93). Here a question is a (word, kind) pair,
//! the answer is short, and **every word an answer names is asked every
//! kind** — so every told `w k₁ v` is testable under every `k₂`.

#[allow(unused_imports)]
use super::*;

impl Quest {
    /// Ask `kinds` one at a time, for the seed and then for every word any
    /// answer names, breadth first to the quest's depth.
    pub fn one_kind_at_a_time(mut self, kinds: &[&str]) -> Self {
        let kinds: Vec<String> = kinds.iter().map(|k| k.to_string()).collect();
        let seed = self.frontier.pop_front().map(|(w, _)| w).unwrap_or_default();
        for k in &kinds {
            self.kind_frontier.push_back((seed.clone(), k.clone(), 0));
        }
        self.queued.insert(seed);
        self.per_kind = Some(kinds);
        self
    }

    /// Also ask every word for two sentences in `voice` (a persona's way of
    /// speaking), after its kinds. Call after [`Quest::one_kind_at_a_time`].
    pub fn with_voice(mut self, voice: &str) -> Self {
        if !voice.trim().is_empty() {
            self.voice = Some(voice.trim().to_string());
            if let Some((seed, _, _)) = self.kind_frontier.front().cloned() {
                self.kind_frontier.push_back((seed, VOICE_KIND.to_string(), 0));
            }
        }
        self
    }

    /// The quest a persona file plans, from one of its seeds.
    pub fn for_persona(p: &crate::persona::Persona, seed: &str) -> Self {
        let kinds: Vec<&str> = p.kinds.iter().map(String::as_str).collect();
        Quest::new(seed).with_depth(p.depth).with_budget(p.per_seed).one_kind_at_a_time(&kinds).with_voice(&p.voice)
    }

    /// Questions still waiting in one-kind-at-a-time mode.
    pub fn pending_questions(&self) -> usize {
        self.kind_frontier.len()
    }

    pub(crate) fn ask_one_kind(&mut self, oracle: &mut dyn Oracle) -> Option<Step> {
        let kinds = self.per_kind.clone()?;
        let (about, kind, depth) = loop {
            let (w, k, d) = self.kind_frontier.pop_front()?;
            if d <= self.depth {
                break (w, k, d);
            }
        };
        if !self.asked.contains(&about) {
            self.asked.push(about.clone());
        }
        if kind == VOICE_KIND {
            return Some(self.ask_voice(oracle, &about, depth));
        }
        let step = self.ask_about(oracle, &about, depth, &template::ONE_KIND, &[kind]);
        for (_, other, _) in &step.kept {
            if depth < self.depth && self.queued.insert(other.clone()) {
                for k in &kinds {
                    self.kind_frontier.push_back((other.clone(), k.clone(), depth + 1));
                }
                if self.voice.is_some() {
                    self.kind_frontier.push_back((other.clone(), VOICE_KIND.to_string(), depth + 1));
                }
            }
        }
        self.steps.push(step.clone());
        Some(step)
    }
}

impl Quest {
    /// Ask only the voice question, for each of `words` — for a persona
    /// whose relations were already asked (the night's tables), so its
    /// sentences cost one question a word. `after` sees every step.
    pub fn ask_voices(&mut self, oracle: &mut dyn Oracle, words: &[String], mut after: impl FnMut(&Quest, &Step)) -> usize {
        let mut failed = 0;
        for w in words {
            let step = self.ask_voice(oracle, w, 0);
            after(self, &step);
            failed = match step.answer.is_said() {
                true => 0,
                false => failed + 1,
            };
            if failed >= MAX_FAILURES_IN_A_ROW {
                self.stopped = Some(format!("{failed} failures in a row; the last: {}", step.answer.text()));
                break;
            }
        }
        self.steps.len()
    }
}

/// The frontier's marker for a voice question.
pub(crate) const VOICE_KIND: &str = "⟨voice⟩";

impl Quest {
    /// Ask for sentences with `about` in the persona's voice; keep each line
    /// that reads as a sentence (3 to 40 words) and names the word.
    fn ask_voice(&mut self, oracle: &mut dyn Oracle, about: &str, depth: u32) -> Step {
        let voice = self.voice.clone().unwrap_or_default();
        let prompt = template::VOICE.fill(about, &[voice.as_str()]);
        let answer = match oracle.ask(&prompt) {
            Ok(text) => {
                for line in text.lines() {
                    let l = line.trim().trim_start_matches(|c: char| c.is_ascii_digit() || "-*•.) ".contains(c)).trim_matches('"').trim();
                    let n = l.split_whitespace().count();
                    if (3..=40).contains(&n) && l.to_lowercase().contains(&about.to_lowercase()) {
                        self.sentences.push((about.to_string(), l.to_string()));
                    }
                }
                Answer::Said(text)
            }
            Err(e) => Answer::Nothing(e.to_string()),
        };
        let step = Step { about: about.to_string(), depth, template: template::VOICE.name, prompt, answer, kept: Vec::new(), opened: Vec::new() };
        self.steps.push(step.clone());
        step
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_answer_is_asked_every_kind() {
        let mut q = Quest::new("w0").with_depth(1).with_budget(100).one_kind_at_a_time(&["is-a", "near"]);
        let asked = q.run(&mut Synthetic::new(16));
        // w0 is-a f0, w0 near w1: then f0 and w1 are each asked both kinds.
        assert_eq!(asked, 2 + 2 * 2);
        let about_kind: Vec<(String, String)> = q.steps.iter().map(|s| (s.about.clone(), s.kept.first().map_or(String::new(), |k| k.0.clone()))).collect();
        assert!(about_kind.contains(&("w1".into(), "is-a".into())));
        assert!(about_kind.contains(&("w1".into(), "near".into())));
        // Each step keeps only its own kind, though the oracle answered all five.
        assert!(q.steps.iter().all(|s| s.kept.len() <= 1));
    }

    #[test]
    fn a_voice_asks_every_word_for_its_sentences() {
        let said = "is-a | mammal | 0.9";
        let voice = "A cat curls up where the sun is.\n2. The cat is proud of its whiskers.\nnothing here";
        let mut o = Scripted::new(&[("Relation:", said), ("Write 2 short sentences", voice)]);
        let mut q = Quest::new("cat").with_depth(0).with_budget(10).one_kind_at_a_time(&["is-a"]).with_voice("as a poet");
        q.run(&mut o);
        assert_eq!(q.steps.len(), 2, "one kind and one voice question for cat");
        assert_eq!(q.sentences.len(), 2);
        assert_eq!(q.sentences[1].1, "The cat is proud of its whiskers.");
        let mut v = Quest::new("x").with_voice("as a poet");
        v.ask_voices(&mut Scripted::new(&[("Write 2 short sentences", voice)]), &["cat".to_string(), "dog".to_string()], |_, _| {});
        assert_eq!((v.steps.len(), v.sentences.len()), (2, 2), "dog is never named in the canned answer");
    }
}
