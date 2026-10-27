//! Where one `.blend` open spends its time: every stage `Phx::open_with`
//! reports, totalled by stage, and every gap between two reports longer
//! than `--slow` milliseconds named by the labels either side of it.
//!
//!   cargo run --release -p phiphx --example open_stages -- FILE [--slow MS]
//!
//! `bench_open` says how long an open takes; this says where. The labels are
//! the progress lines the app's loading panel shows, so a slow object reads
//! as `Reading objects (812/3515) Rock.044` — the report after it was read.
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// A stage's name without its counter or object: `Reading objects (12/40)
/// Cube` and `Indexing blocks (900)` total under `Reading objects` and
/// `Indexing blocks`.
fn stage(label: &str) -> &str {
    label.split(" (").next().unwrap_or(label)
}

struct Clock {
    began: Instant,
    last: Instant,
    label: String,
    total: BTreeMap<String, Duration>,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(file) = args.first() else {
        eprintln!("usage: open_stages FILE [--slow MS]");
        std::process::exit(2);
    };
    let slow = match args.iter().position(|a| a == "--slow") {
        Some(at) => args.get(at + 1).and_then(|ms| ms.parse().ok()).unwrap_or(300),
        None => 300,
    };
    let slow = Duration::from_millis(slow);
    let now = Instant::now();
    let clock = Mutex::new(Clock { began: now, last: now, label: "start".to_string(), total: BTreeMap::new() });
    let report = |_fraction: f32, label: &str| {
        let mut c = clock.lock().unwrap_or_else(|e| e.into_inner());
        let at = Instant::now();
        let gap = at - c.last;
        // The gap is the work between the two reports: the one before
        // announced it ("Evaluating modifiers") or the one now closes it
        // ("Reading objects (n/…) Name"), so both are printed.
        if gap >= slow {
            println!("{:>9.1}s  +{:>8.1}s  {}  ->  {}", (at - c.began).as_secs_f64(), gap.as_secs_f64(), c.label, label);
        }
        let key = stage(&c.label).to_string();
        *c.total.entry(key).or_default() += gap;
        if stage(&c.label) != stage(label) {
            println!("{:>9.1}s  {}", (at - c.began).as_secs_f64(), stage(label));
        }
        c.last = at;
        c.label = label.to_string();
        true
    };
    let model = phiphx::Phx::open_with(std::path::Path::new(file), &report);
    let mut c = clock.lock().unwrap_or_else(|e| e.into_inner());
    let at = Instant::now();
    let tail = at - c.last;
    let key = format!("{} (to the end of open)", stage(&c.label));
    *c.total.entry(key).or_default() += tail;
    println!("{:>9.1}s  open returned ({})", (at - c.began).as_secs_f64(), if model.is_ok() { "ok" } else { "error" });
    let mut totals: Vec<(&String, &Duration)> = c.total.iter().collect();
    totals.sort_by(|a, b| b.1.cmp(a.1));
    println!("\nBy stage:");
    for (name, spent) in totals {
        println!("{:>9.1}s  {name}", spent.as_secs_f64());
    }
    if let Ok(model) = model {
        let verts: usize = model.objects.iter().map(|o| o.mesh.vertices.len()).sum();
        let tris: usize = model.objects.iter().map(|o| o.mesh.indices.len() / 3).sum();
        println!("\n{} objects, {} vertices, {} triangles", model.objects.len(), verts, tris);
    }
}
