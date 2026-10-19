//! What the spatial node view would show for every material of a file: its
//! solids by class, its tubes, and the floor they stand on — the layout pass
//! (`canvas/node/spatial/layout.rs`) run over a real file with no window.
//!
//!   cargo run --release -p phiber --example spatial_graph -- FILE [name filter] [--top N]
//!
//! Written for the CW-POSES set: pose 26 carries 337 materials and 3 073
//! nodes, Epic Female 01 236 and 2 288, so a filter or `--top` keeps the
//! print readable. An idname wrapped as `Undefined[...]` is a node whose type
//! the saving Blender did not know; it is counted apart because Blender draws
//! it as "Undefined" in red rather than as the type inside the brackets.

use phiber::phiphx::Phx;
use phiber::v2::view::draw::canvas::node::plan::BCardSize;
use phiber::v2::view::draw::canvas::node::select::BNodeSelection;
use phiber::v2::view::draw::canvas::node::spatial::layout::BSpatialLayout;
use std::collections::BTreeMap;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = args.first().expect("file");
    let top: usize = args.iter().position(|a| a == "--top").and_then(|i| args.get(i + 1)).and_then(|n| n.parse().ok()).unwrap_or(usize::MAX);
    let filter = args.get(1).filter(|a| !a.starts_with("--")).cloned().unwrap_or_default();
    let started = std::time::Instant::now();
    let model = Phx::open(std::path::Path::new(path)).expect("open");
    println!("{} — {} objects, read in {:.1}s", path, model.objects.len(), started.elapsed().as_secs_f32());
    // The theme's defaults for a card, as `set::render` reads them off the
    // resolved style: header 30, row 36, width 140 are Blender's own units.
    let size = BCardSize { header: 30.0, unit: 36.0, width: 140.0 };
    let selection = BNodeSelection::default();
    let mut rows = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for object in &model.objects {
        for material in &object.materials {
            if !material.name.contains(&filter) || !seen.insert(material.name.clone()) || material.graph.is_empty() {
                continue;
            }
            let graph = &material.graph;
            let offsets: Vec<(String, (f32, f32))> = Vec::new();
            let layout = BSpatialLayout::of(graph, &graph.links, &offsets, size, &selection);
            let mut classes: BTreeMap<String, usize> = BTreeMap::new();
            for solid in &layout.solids {
                *classes.entry(format!("{:?}", solid.class)).or_default() += 1;
            }
            let undefined = graph.nodes.iter().filter(|n| n.idname.starts_with("Undefined[")).count();
            let tallest = layout.solids.iter().map(|s| s.spec.tier).max().unwrap_or(0);
            let floor = layout.floor().map(|(x0, y0, x1, y1)| format!("{:.0}x{:.0}", x1 - x0, y1 - y0)).unwrap_or_default();
            rows.push((layout.solids.len(), format!(
                "{:36} {:28} nodes={:3} links={:3} solids={:3} plates={} tubes={:3} undefined={} tallest=tier{} floor={} classes={:?}",
                object.name, material.name, graph.nodes.len(), graph.links.len(), layout.solids.len(), layout.plates.len(), layout.tubes.len(), undefined, tallest, floor, classes,
            )));
        }
    }
    rows.sort_by(|a, b| b.0.cmp(&a.0));
    for (_, line) in rows.iter().take(top) {
        println!("{line}");
    }
    let (materials, nodes): (usize, usize) = (rows.len(), rows.iter().map(|r| r.0).sum());
    println!("{materials} materials with a graph, {nodes} solids in all");
}
