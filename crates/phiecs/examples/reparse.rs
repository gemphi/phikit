//! Rebuild a quest's `relations.tsv` from its step pages with today's parser,
//! without asking the oracle again.
//!
//! ```text
//! cargo run --release --example reparse -- ../phiano/data/quest/phi4_cat
//! ```

use phiecs::quest::reparse_dir;
use std::path::Path;

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| "quest".to_string());
    match reparse_dir(Path::new(&dir)) {
        Ok(pages) => {
            let rows: usize = pages.iter().map(|(_, n)| n).sum();
            for (i, (word, n)) in pages.iter().enumerate() {
                println!("  {:>3}. {word:<16} {n} relations", i + 1);
            }
            println!("rewrote {dir}/relations.tsv: {} pages, {rows} relations", pages.len());
        }
        Err(e) => eprintln!("could not reparse {dir}: {e}"),
    }
}
