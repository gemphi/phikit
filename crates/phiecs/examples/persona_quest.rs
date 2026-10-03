//! Run the quest a persona file plans: one quest per seed, one kind at a
//! time, with the persona's voice, into `<out>/<persona>/<seed>/`.
//!
//! ```text
//! cargo run -p phiecs --release --example persona_quest -- personas/rustacean.md ../phiano/data/personas ollama run phi4
//! cargo run -p phiecs --release --example persona_quest -- personas/rustacean.md /tmp/p synthetic 64
//! ```
//! A seed whose folder already has `relations.tsv` (and `sentences.tsv`, if
//! the persona has a voice) is skipped, so a stopped run resumes. Each quest
//! writes its steps, `relations.tsv` and `sentences.tsv` as every answer
//! arrives.
//!
//! `--reuse <dir>`: a seed that `<dir>/<seed>/relations.tsv` already asked
//! (the night's tables) is not asked again — its table is copied, and only
//! the voice question is put, once for each word the table asked about.

use phiecs::persona::Persona;
use phiecs::prelude::*;
use phiecs::quest::Synthetic;
use std::path::Path;

fn main() {
    let mut a: Vec<String> = std::env::args().skip(1).collect();
    let reuse = a.iter().position(|x| x == "--reuse").map(|i| {
        let d = a.get(i + 1).cloned().unwrap_or_default();
        a.drain(i..(i + 2).min(a.len()));
        std::path::PathBuf::from(d)
    });
    let (Some(file), Some(out)) = (a.first(), a.get(1)) else {
        eprintln!("usage: persona_quest <persona.md> <out dir> <oracle command…>");
        std::process::exit(2);
    };
    let text = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("cannot read {file}: {e}"));
    let persona = Persona::from_md(&text).unwrap_or_else(|e| panic!("{file}: {e}"));
    println!("{} · seeds {} · kinds {} · {} questions a seed · voice: {}", persona.name, persona.seeds.join(", "), persona.kinds.join(", "), persona.per_seed, persona.voice);
    let root = Path::new(out).join(persona.slug());
    for seed in &persona.seeds {
        let dir = root.join(seed);
        let voiced = persona.voice.is_empty() || dir.join("sentences.tsv").exists();
        if dir.join("relations.tsv").exists() && voiced {
            println!("{seed}: done already, skipped");
            continue;
        }
        let mut synthetic;
        let mut spawned;
        let oracle: &mut dyn Oracle = match a.get(2).map(String::as_str) {
            Some("synthetic") => {
                synthetic = Synthetic::new(a.get(3).and_then(|s| s.parse().ok()).unwrap_or(64));
                &mut synthetic
            }
            Some(cmd) => {
                let args: Vec<&str> = a[3..].iter().map(String::as_str).collect();
                spawned = Spawned::new(cmd, &args);
                &mut spawned
            }
            None => panic!("name an oracle: `ollama run phi4` or `synthetic 64`"),
        };
        let name = oracle.name();
        if let Some(table) = reuse.as_ref().map(|r| r.join(seed).join("relations.tsv")).filter(|t| t.exists()) {
            let text = std::fs::read_to_string(&table).unwrap_or_default();
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(dir.join("relations.tsv"), &text);
            let mut words: Vec<String> = Vec::new();
            for r in phiecs::quest::parse_tsv(&text) {
                if !words.contains(&r.about) {
                    words.push(r.about);
                }
            }
            println!("{seed}: reusing {} ({} words asked) · voice only", table.display(), words.len());
            let mut quest = Quest::new(seed).with_voice(&persona.voice);
            let sentences = dir.join("sentences.tsv");
            let mut clock = std::time::Instant::now();
            quest.ask_voices(oracle, &words, |q, step| {
                println!("  {:>3}. {:<16} {:>5.1}s  {} sentences", q.steps.len(), step.about, clock.elapsed().as_secs_f64(), q.sentences.len());
                clock = std::time::Instant::now();
                let lines: String = q.sentences.iter().map(|(w, l)| format!("{w}\t{l}\n")).collect();
                let _ = std::fs::write(&sentences, lines);
            });
            if let Some(why) = &quest.stopped {
                eprintln!("{seed}: stopped early: {why}");
            }
            continue;
        }
        let mut quest = Quest::for_persona(&persona, seed);
        let mut clock = std::time::Instant::now();
        quest.run_watching(oracle, |q, step| {
            println!("  {:>3}. {:<16} {:>5.1}s  {} kept · {} sentences", q.steps.len(), step.about, clock.elapsed().as_secs_f64(), step.kept.len(), q.sentences.len());
            clock = std::time::Instant::now();
            if let Err(e) = q.write(&dir, &name) {
                eprintln!("could not write {}: {e}", dir.display());
            }
        });
        if let Some(why) = &quest.stopped {
            eprintln!("{seed}: stopped early: {why}");
        }
        let _ = quest.write(&dir, &name);
        println!("{seed}: {} questions · {} sentences → {}", quest.steps.len(), quest.sentences.len(), dir.display());
    }
}
