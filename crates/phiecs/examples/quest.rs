//! Run a quest against a real oracle and write what it learned.
//!
//! ```text
//! cargo run --release --example quest -- cat 3 64 ollama run phi4
//! cargo run --release --example quest -- cat 2 16          # dry run, scripted
//! cargo run --release --example quest -- w0 4 200 synthetic 64   # hidden structure
//! PHIQUEST_KINDS=is-a,has,part-of cargo run --release --example quest -- cat 2 300 ollama run phi4
//! ```
//! Arguments: seed word, depth, budget, then the command that answers prompts
//! on stdin. With no command it runs the scripted oracle, which proves the
//! loop and the files without a model installed.

use phiecs::prelude::*;
use phiecs::quest::{Scripted, Synthetic};
use std::path::Path;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let seed = a.first().cloned().unwrap_or_else(|| "cat".to_string());
    let depth: u32 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(2);
    let budget: u32 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(24);
    let out = std::env::var("PHIQUEST_OUT").unwrap_or_else(|_| "quest".to_string());

    let mut scripted;
    let mut spawned;
    let mut synthetic;
    let oracle: &mut dyn Oracle = match a.len() > 3 {
        true if a[3] == "synthetic" => {
            let n = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(64);
            synthetic = Synthetic::new(n);
            &mut synthetic
        }
        true => {
            let args: Vec<&str> = a[4..].iter().map(String::as_str).collect();
            spawned = Spawned::new(&a[3], &args);
            &mut spawned
        }
        false => {
            scripted = Scripted::new(&[(
                "Word:",
                "is-a | thing | 0.6\nnear | word | 0.4\nused-for | naming | 0.5",
            )]);
            &mut scripted
        }
    };
    let name = oracle.name();

    let mut quest = Quest::new(&seed).with_depth(depth).with_budget(budget);
    // PHIQUEST_KINDS=is-a,has,part-of asks one kind at a time, and every word
    // an answer names is asked every kind — the shape laws need (docs/94).
    if let Ok(k) = std::env::var("PHIQUEST_KINDS") {
        let kinds: Vec<&str> = k.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
        quest = quest.one_kind_at_a_time(&kinds);
        println!("one kind at a time: {}", kinds.join(", "));
    }
    println!("quest from `{seed}` · depth {depth} · budget {budget} · oracle {name}");
    // Each answer is printed and the quest so far is written as it arrives:
    // a model on a CPU takes seconds to minutes a question, and a run stopped
    // with Ctrl-C keeps everything it was told.
    let dir = Path::new(&out).to_path_buf();
    let mut clock = std::time::Instant::now();
    let asked = quest.run_watching(oracle, |q, step| {
        let said = match step.answer.is_said() {
            true => format!("{} relations kept, {} words opened", step.kept.len(), step.opened.len()),
            false => step.answer.text().to_string(),
        };
        println!("  {:>3}. {:<16} {:>5.1}s  {said}", q.steps.len(), step.about, clock.elapsed().as_secs_f64());
        clock = std::time::Instant::now();
        if let Err(e) = q.write(&dir, &name) {
            eprintln!("could not write {}: {e}", dir.display());
        }
    });
    if let Some(why) = &quest.stopped {
        eprintln!("stopped early: {why}");
    }

    let w = &quest.world;
    println!(
        "asked {asked} · {} words · {} kinds · {} edges · {} still on the frontier",
        w.lexicon.len(),
        w.relations.len(),
        w.edges.len(),
        quest.pending() + quest.pending_questions()
    );
    for (kind, kname) in w.relations.iter().filter(|(k, _)| w.relations.evidence(*k) > 0) {
        println!("  {:>12}  {} pairs", kname, w.relations.evidence(kind));
    }
    match quest.write(Path::new(&out), &name) {
        Ok(()) => println!("wrote {out}/README.md, {out}/relations.tsv and {} step pages", quest.steps.len()),
        Err(e) => eprintln!("could not write {out}: {e}"),
    }
}
