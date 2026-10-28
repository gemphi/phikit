//! One object's evaluated vertices against a dump of Blender's: the largest
//! and the mean distance between same-index vertices.
//!
//! Written for the Corrective Smooth port (`modifier/corrective_smooth.rs`),
//! whose real use is `Hair.001` in the CW-POSES files — too big a file to be
//! a test. The truth is one line per dump, `tag \t count \t x y z x y z …`,
//! as `tests/fixtures/make_corrective_smooth.py` writes it:
//!
//! ```text
//! cargo run --release -p phiphx --no-default-features --example eval_vs_blender -- \
//!     f.blend "Hair.001" truth.tsv full
//! PHIPHX_MODIFIERS=1 cargo run … -- f.blend "Hair.001" truth.tsv cs_only
//! ```

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [path, object, truth, tag] = args.as_slice() else {
        eprintln!("usage: eval_vs_blender <file.blend> <object> <truth.tsv> <tag>");
        std::process::exit(2);
    };
    let text = std::fs::read_to_string(truth).expect("truth file");
    let theirs: Vec<[f32; 3]> = text
        .lines()
        .find(|l| l.split('\t').next() == Some(tag.as_str()))
        .and_then(|l| l.split('\t').nth(2))
        .map(|v| {
            let nums: Vec<f32> = v.split(' ').filter_map(|x| x.parse().ok()).collect();
            nums.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect()
        })
        .unwrap_or_else(|| panic!("no `{tag}` line in {truth}"));
    let model = phiphx::Phx::open(std::path::Path::new(path)).expect("the file opens");
    let ours = &model.objects.iter().find(|o| &o.name == object).unwrap_or_else(|| panic!("no object {object}")).mesh.vertices;
    println!("{object} [{tag}]: phiphx {} vertices, Blender {}", ours.len(), theirs.len());
    if ours.len() != theirs.len() {
        std::process::exit(1);
    }
    let d: Vec<f32> = ours
        .iter()
        .zip(&theirs)
        .map(|(p, q)| (0..3).map(|i| (p[i] - q[i]).powi(2)).sum::<f32>().sqrt())
        .collect();
    let (lo, hi) = theirs.iter().fold(([f32::MAX; 3], [f32::MIN; 3]), |(lo, hi), p| {
        (std::array::from_fn(|i| lo[i].min(p[i])), std::array::from_fn(|i| hi[i].max(p[i])))
    });
    let size = (0..3).map(|i| (hi[i] - lo[i]).powi(2)).sum::<f32>().sqrt();
    let max = d.iter().copied().fold(0.0, f32::max);
    let mean = d.iter().sum::<f32>() / d.len().max(1) as f32;
    println!("size (bbox diagonal) {size:.4}  max {max:.3e} ({:.3e} of size)  mean {mean:.3e}", max / size);
}
