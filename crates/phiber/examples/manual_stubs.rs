//! Writes the manual's pages from the declaration — a stub for every node
//! that has no prose yet, and an index for every chapter.
//!
//! The tree on disk is a *projection* of `v2::manual::MANUAL`, never the
//! other way round: add a node and its file appears here; the file is where
//! the prose goes. A page that already has prose is left alone, so this is
//! safe to re-run — it only ever fills gaps.
//!
//! Run with:  cargo run --release -p phiber --example manual_stubs
//! Writes to: crates/phiber/docs/manual/

use phiber::v2::manual::{BManualNode, BManualState, MANUAL, progress, stubs};

const ROOT: &str = "crates/phiber/docs/manual";

/// The line that tells a reader — and the next writer — where this page
/// stands and what it has to match.
fn front(node: &BManualNode, path: &str) -> String {
    let answers = match node.blender.is_empty() {
        true => "phiber's own — Blender has no counterpart for this.".to_string(),
        false => format!("Answers to Blender's `manual/{}`.", node.blender),
    };
    let figure = match node.figure {
        Some(f) => format!("\n**Figure:** `{}` with the {} in focus — see `figures/`.\n", f.space.slug(), f.host.spec().label),
        None => String::new(),
    };
    format!(
        "# {}\n\n<!-- Generated stub from `v2::manual::set` ({path}). Write the prose here;\n     re-running `manual_stubs` will not overwrite a page that has any. -->\n\n> **State:** {:?}. {answers}\n{figure}\n",
        node.title, node.state
    )
}

fn write(node: &'static BManualNode, dir: &std::path::Path, path: &str) -> std::io::Result<usize> {
    let mut made = 0;
    let here = format!("{path}/{}", node.slug);
    let (dir, file) = match node.children.is_empty() {
        true => (dir.to_path_buf(), dir.join(format!("{}.md", node.slug))),
        false => (dir.join(node.slug), dir.join(node.slug).join("index.md")),
    };
    std::fs::create_dir_all(&dir)?;
    // Only ever fill a gap: a page with prose in it is the writer's.
    let empty = std::fs::read_to_string(&file).map(|t| t.trim().is_empty()).unwrap_or(true);
    if empty {
        std::fs::write(&file, front(node, &here))?;
        made += 1;
    }
    for child in node.children {
        made += write(child, &dir, &here)?;
    }
    Ok(made)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(ROOT);
    let mut made = 0;
    for chapter in MANUAL {
        made += write(chapter, root, "")?;
    }

    // The gap list, as a file, so it can be read without running anything.
    let (done, total) = progress();
    let mut gaps = format!(
        "# Gap list\n\nGenerated from `v2::manual::set`. {done} of {total} pages done; {} still stubs.\n\n",
        stubs().len()
    );
    for chapter in MANUAL {
        let all = chapter.walk();
        let open: Vec<_> = all.iter().filter(|n| n.state == BManualState::Stub).collect();
        gaps += &format!("## {} — {} of {} written\n\n", chapter.title, all.len() - open.len(), all.len());
        for node in open {
            let against = match node.blender.is_empty() {
                true => "phiber's own".to_string(),
                false => format!("`{}`", node.blender),
            };
            gaps += &format!("- `{}` — {} · against {against}\n", node.slug, node.title);
        }
        gaps.push('\n');
    }
    std::fs::write(root.join("GAPS.md"), gaps)?;

    println!("{made} stub pages written under {ROOT}; {done}/{total} done, gap list in {ROOT}/GAPS.md");
    Ok(())
}
