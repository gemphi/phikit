#[allow(unused_imports)]
use super::*;

use std::path::Path;

/// One row of `relations.tsv`: `about \t kind \t other \t weight`.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub about: String,
    pub kind: String,
    pub other: String,
    pub weight: f64,
}

/// Parse the table a quest writes. Blank lines and a header row (one whose
/// weight does not parse) are skipped; nothing else is forgiven, so a
/// malformed file is noticed rather than half-read.
pub fn parse_tsv(text: &str) -> Vec<Row> {
    text.lines()
        .filter_map(|line| {
            let mut f = line.split('\t').map(str::trim);
            let (about, kind, other, weight) = (f.next()?, f.next()?, f.next()?, f.next()?);
            let weight = weight.parse::<f64>().ok()?;
            if about.is_empty() || kind.is_empty() || other.is_empty() {
                return None;
            }
            Some(Row {
                about: about.to_string(),
                kind: kind.to_string(),
                other: other.to_string(),
                weight: weight.clamp(0.0, 1.0),
            })
        })
        .collect()
}

impl Quest {
    /// Rebuild a world from a written `relations.tsv`: every row becomes an
    /// edge and a coupling, exactly as it did when the oracle said it. The
    /// steps are not recovered — the table is the graph, the pages are the
    /// transcript — so the result is a quest with a world and no history,
    /// ready to be continued from any word.
    pub fn from_rows(rows: &[Row]) -> Self {
        let seed = rows.first().map(|r| r.about.as_str()).unwrap_or("word");
        let mut quest = Quest::new(seed);
        for r in rows {
            let Some(kind) = quest.world.declare(&r.kind) else { continue };
            let head = quest.world.coin_bare(&r.about);
            let other = quest.world.coin_bare(&r.other);
            let rate = crate::world::COUPLE_RATE * r.weight;
            quest.world.relate_by(head, kind, other, r.weight, rate);
            if !quest.asked.iter().any(|a| a == &r.about) {
                quest.asked.push(r.about.clone());
            }
        }
        quest.frontier.clear();
        quest
    }

    /// [`Quest::from_rows`] on a file.
    pub fn from_tsv(path: &Path) -> std::io::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self::from_rows(&parse_tsv(&text)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_synthetic_quest() {
        let mut oracle = Synthetic::new(16);
        let mut quest = Quest::new("w0").with_depth(3).with_budget(64);
        quest.run(&mut oracle);
        let tsv: String = quest.steps.iter().map(Step::to_tsv).collect();
        let rows = parse_tsv(&tsv);
        assert_eq!(rows.len(), quest.world.edges.len());
        let back = Quest::from_rows(&rows);
        assert_eq!(back.world.edges.len(), quest.world.edges.len());
        assert_eq!(back.world.lexicon.len(), quest.world.lexicon.len());
        // Leaves answered nothing and left no rows, so only words with edges
        // count as asked in the rebuilt quest.
        let with_edges = quest.steps.iter().filter(|s| !s.kept.is_empty()).count();
        assert_eq!(back.asked().len(), with_edges);
    }

    #[test]
    fn skips_the_header() {
        let rows = parse_tsv("about\tkind\tother\tweight\ncat\tis-a\tanimal\t0.9\n");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].other, "animal");
    }
}
