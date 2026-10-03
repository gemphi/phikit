#[allow(unused_imports)]
use super::*;

use std::process::Command;

/// Anything that answers a prompt with text: Phi-4 through llama.cpp or
/// ollama, an HTTP endpoint behind a shell script, or a scripted stub in a
/// test. The quest does not care which, and says so in its transcript.
pub trait Oracle {
    /// Answer `prompt`, or say why not.
    fn ask(&mut self, prompt: &str) -> Result<String, OracleError>;

    /// What to record in the transcript as the source of these answers.
    fn name(&self) -> String {
        "oracle".to_string()
    }
}

#[derive(Debug)]
pub enum OracleError {
    /// The oracle could not be reached or run.
    Unreachable(String),
    /// It ran and said nothing usable.
    Empty,
    /// It ran and failed.
    Failed(String),
}

impl std::fmt::Display for OracleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OracleError::Unreachable(s) => write!(f, "oracle unreachable: {s}"),
            OracleError::Empty => write!(f, "oracle answered nothing"),
            OracleError::Failed(s) => write!(f, "oracle failed: {s}"),
        }
    }
}

impl std::error::Error for OracleError {}

/// An oracle that is a command: the prompt goes in on stdin, the answer comes
/// back on stdout. This is how Phi-4 is reached without linking a runtime —
/// `llama-cli -m phi-4.gguf -f /dev/stdin`, `ollama run phi4`, or a script of
/// your own. Everything about the model stays outside this crate.
pub struct Spawned {
    program: String,
    args: Vec<String>,
}

impl Spawned {
    pub fn new(program: &str, args: &[&str]) -> Self {
        Self {
            program: program.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// `ollama run <model>` — the shortest path to a local Phi-4.
    pub fn ollama(model: &str) -> Self {
        Self::new("ollama", &["run", model])
    }
}

impl Oracle for Spawned {
    fn ask(&mut self, prompt: &str) -> Result<String, OracleError> {
        use std::io::Write;
        let mut child = Command::new(&self.program)
            .args(&self.args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| OracleError::Unreachable(format!("{}: {e}", self.program)))?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(prompt.as_bytes())
                .map_err(|e| OracleError::Failed(e.to_string()))?;
        }
        let out = child.wait_with_output().map_err(|e| OracleError::Failed(e.to_string()))?;
        if !out.status.success() {
            return Err(OracleError::Failed(String::from_utf8_lossy(&out.stderr).into_owned()));
        }
        let text = String::from_utf8_lossy(&out.stdout).into_owned();
        match text.trim().is_empty() {
            true => Err(OracleError::Empty),
            false => Ok(text),
        }
    }

    fn name(&self) -> String {
        format!("{} {}", self.program, self.args.join(" "))
    }
}

/// An oracle with its answers written in advance: what the tests use, and what
/// a dry run uses to prove the loop works before a model is installed.
pub struct Scripted {
    answers: Vec<(String, String)>,
    pub asked: Vec<String>,
}

impl Scripted {
    /// `(contains, answer)` — the first entry whose key appears in the prompt.
    pub fn new(answers: &[(&str, &str)]) -> Self {
        Self {
            answers: answers.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            asked: Vec::new(),
        }
    }
}

impl Oracle for Scripted {
    fn ask(&mut self, prompt: &str) -> Result<String, OracleError> {
        self.asked.push(prompt.to_string());
        self.answers
            .iter()
            .find(|(k, _)| prompt.contains(k.as_str()))
            .map(|(_, v)| v.clone())
            .ok_or(OracleError::Empty)
    }

    fn name(&self) -> String {
        "scripted".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_program_stops_the_quest_at_once() {
        let mut oracle = Spawned::new("phiecs-no-such-program", &[]);
        let mut quest = Quest::new("cat").with_budget(64);
        let asked = quest.run(&mut oracle);
        assert_eq!(asked, 1);
        assert!(quest.stopped.as_deref().unwrap_or("").contains("unreachable"));
    }

    #[test]
    fn an_empty_answer_does_not_stop_the_quest() {
        // The synthetic oracle answers nothing for leaves; the walk goes on.
        let mut oracle = Synthetic::new(16);
        let mut quest = Quest::new("w0").with_depth(3).with_budget(64);
        let asked = quest.run(&mut oracle);
        assert!(asked > 16 && quest.stopped.is_none(), "asked {asked}, stopped {:?}", quest.stopped);
    }
}
