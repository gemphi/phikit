//! Reading a quest's transcript again with today's parser. An answer is
//! evidence and is kept verbatim on its step page, so a better reader should
//! not cost another hour of Phi-4: `reparse_dir` rebuilds `relations.tsv`
//! from the pages alone.
#[allow(unused_imports)]
use super::*;

use std::path::Path;

/// One step page read back: the word, the kinds its prompt offered, the answer.
#[derive(Clone, Debug, PartialEq)]
pub struct Page {
    pub about: String,
    pub offered: Vec<String>,
    pub answer: String,
}

impl Page {
    /// Read a page written by [`Quest::write`]. `None` if it is not one.
    pub fn read(md: &str) -> Option<Page> {
        let about = md.lines().next()?.strip_prefix("# ")?.trim().to_string();
        // A one-kind page names its kind on a `Relation:` line; a relations
        // page lists what it offered on `- kind: one of:`.
        let offered = match md.lines().find_map(|l| l.trim().strip_prefix("Relation:")) {
            Some(k) => vec![k.trim().to_string()],
            None => md
                .lines()
                .find_map(|l| l.trim().strip_prefix("- kind: one of:"))
                .map(|k| k.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
                .unwrap_or_default(),
        };
        let (start, end) = (md.find("## Answered")?, md.find("## Kept")?);
        let block = md.get(start + "## Answered".len()..end)?.trim();
        let answer = block.strip_prefix("```")?.strip_suffix("```")?.trim().to_string();
        Some(Page { about, offered, answer })
    }

    /// What today's parser keeps from this answer, reading kinds against
    /// `offered` (a page from before 2026-09-28 lists invented kinds too).
    pub fn claims(&self, offered: &[String]) -> Vec<Claim> {
        parse_relations(&self.answer, offered).into_iter().filter(|c| !parse::is_self_edge(c, &self.about)).collect()
    }
}

/// Re-read every `steps/*.md` under `dir`, in name order, and rewrite
/// `relations.tsv` from them, offering the [`base_kinds`]. Returns each
/// page's word and rows kept.
pub fn reparse_dir(dir: &Path) -> std::io::Result<Vec<(String, usize)>> {
    let mut pages: Vec<_> = std::fs::read_dir(dir.join("steps"))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    pages.sort();
    let (mut tsv, mut out, offered) = (String::new(), Vec::new(), base_kinds());
    for p in pages {
        let Some(page) = Page::read(&std::fs::read_to_string(&p)?) else { continue };
        let kinds = match page.offered.len() == 1 {
            true => &page.offered,
            false => &offered,
        };
        let claims = page.claims(kinds);
        for c in &claims {
            tsv.push_str(&format!("{}\t{}\t{}\t{:.3}\n", page.about, c.kind, c.other, c.weight));
        }
        out.push((page.about, claims.len()));
    }
    std::fs::write(dir.join("relations.tsv"), tsv)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_written_quest_reparses_to_the_same_rows() {
        let said = "mammal | mammal | 1.0\nhas fur 0.9  \ndoes give birth 0.7\nlikes fish 0.6";
        let mut q = Quest::new("cat").with_depth(0).with_budget(1);
        q.run(&mut Scripted::new(&[("Word: cat", said)]));
        let dir = std::env::temp_dir().join(format!("phiecs-reparse-{}", std::process::id()));
        q.write(&dir, "scripted").unwrap();
        let before = std::fs::read_to_string(dir.join("relations.tsv")).unwrap();
        let counts = reparse_dir(&dir).unwrap();
        let after = std::fs::read_to_string(dir.join("relations.tsv")).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(before, after);
        assert_eq!(counts, vec![("cat".to_string(), 3)]);
        assert!(after.contains("cat\tdoes\tgive-birth\t0.700") && after.contains("cat\tlike\tfish"));
        assert!(!after.contains("mammal\tmammal"), "a kind named by its answer is a self-edge");
    }
}
