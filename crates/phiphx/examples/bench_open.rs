//! Time a `.blend` open exactly the way Phiber opens it (no sidecar, real
//! progress callbacks), against Blender on the same file and machine.
//!
//!   cargo run --release -p phiphx --example bench_open -- FILE... [-n RUNS]
//!
//! The Blender side is `tests/truth/blender_open_time.py`, which reports
//! the same two halves: the file read, and the evaluation that realises
//! every object's geometry. `Phx::open` does both before it returns, so its
//! total is what belongs beside Blender's.
//!
//! **Runs matter.** Blender's own figures come from one process opening
//! several files one after another, so everything but the first is measured
//! with warm allocators and a built thread pool. A single `open` in a fresh
//! process pays for those once, which on a small file is most of the
//! number: `LIGHTS.blend` measured 9.4 ms cold and 1.2 ms warm. So this
//! repeats each file and reports **min** and **median** — the min being the
//! fairest thing to put beside Blender's warm figure, and the spread being
//! what says whether a file's cost is work or warm-up.
//!
//! The vertex total is printed because a reader that is faster for having
//! read less is not faster, and the two sides are expected to agree on it.
use std::time::{Duration, Instant};

/// What one file's runs came to.
struct Timing {
    path: String,
    bytes: u64,
    objects: usize,
    verts: usize,
    run: Vec<Duration>,
}

impl Timing {
    fn of(path: &str, runs: usize) -> Option<Self> {
        let file = std::path::Path::new(path);
        let bytes = std::fs::metadata(file).ok()?.len();
        let mut run = Vec::with_capacity(runs);
        let mut last = None;
        for _ in 0..runs {
            let started = Instant::now();
            // `open_with` and a progress sink that does nothing: the real
            // callback path, so the measurement includes what Phiber pays
            // for reporting, without printing a page per run.
            let model = phiphx::Phx::open_with(file, &|_fraction, _stage| true).ok()?;
            run.push(started.elapsed());
            last = Some(model);
        }
        let model = last?;
        Some(Self {
            path: file.file_name()?.to_string_lossy().into_owned(),
            bytes,
            // Cameras and lights are objects to Blender, and are counted
            // here the same way so the two columns mean one thing.
            objects: model.objects.len() + model.cameras.len() + model.lights.len(),
            verts: model.objects.iter().map(|o| o.mesh.vertices.len()).sum(),
            run,
        })
    }

    fn min(&self) -> Duration {
        self.run.iter().copied().min().unwrap_or_default()
    }

    /// The middle run, which says more than a mean when a cold first run is
    /// in the set.
    fn median(&self) -> Duration {
        let mut sorted = self.run.clone();
        sorted.sort_unstable();
        sorted.get(sorted.len() / 2).copied().unwrap_or_default()
    }

    /// One tab-separated line, the same shape the Blender script writes:
    /// file, bytes, objects, vertices, min ms, median ms.
    fn line(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{:.1}\t{:.1}\n",
            self.path,
            self.bytes,
            self.objects,
            self.verts,
            self.min().as_secs_f64() * 1000.0,
            self.median().as_secs_f64() * 1000.0,
        )
    }
}

fn main() {
    let mut runs = 1;
    let mut file = Vec::new();
    let mut argv = std::env::args().skip(1);
    while let Some(word) = argv.next() {
        match word.as_str() {
            "-n" => runs = argv.next().and_then(|n| n.parse().ok()).unwrap_or(1),
            _ => file.push(word),
        }
    }
    if file.is_empty() {
        eprintln!("usage: bench_open FILE... [-n RUNS]");
        std::process::exit(1);
    }
    println!("# file\tbytes\tobjects\tvertices\tmin ms\tmedian ms  ({runs} run(s) each)");
    for path in &file {
        match Timing::of(path, runs.max(1)) {
            Some(timing) => print!("{}", timing.line()),
            None => eprintln!("# {path}: could not be opened"),
        }
    }
}
