//! Print object / triangle counts and open time: `cargo run --release --example tri_count -- file.blend`
fn main() {
    for p in std::env::args().skip(1) {
        let t = std::time::Instant::now();
        let m = phiphx::phx::Phx::open(std::path::Path::new(&p)).expect("open");
        let tris: usize = m.objects.iter().map(|o| o.mesh.indices.len() / 3).sum();
        let verts: usize = m.objects.iter().map(|o| o.mesh.vertices.len()).sum();
        let corners: usize = m.objects.iter().map(|o| o.mesh.corners.len()).sum();
        let uvs: usize = m.objects.iter().map(|o| o.uvs.len()).sum();
        println!("{p}: {} objects, {tris} tris, {verts} verts, {corners} corners, {uvs} uvs, {:?}", m.objects.len(), t.elapsed());
        println!("peak {} MB", rss());
    }
}

/// Peak resident memory (Linux; 0 elsewhere).
fn rss() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| s.lines().find(|l| l.starts_with("VmHWM")).and_then(|l| l.split_whitespace().nth(1)?.parse().ok()))
        .map_or(0, |kb: u64| kb / 1024)
}
