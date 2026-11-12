// SPDX-License-Identifier: Apache-2.0
//! Gate 2's census: which Blender node kinds and which closures the
//! materials and worlds of the test scenes use, and how many scenes each
//! one unblocks. Writes `CENSUS.md`.
//!
//! ```text
//! cargo run -p cycles --release --example census
//! ```
//!
//! A scene counts as served by a set of ported kinds when every node kind
//! and closure its materials and its world use is in the set (the scene may
//! still wait for a lamp, which is gate 3: the table says so). The order is
//! greedy: the kind that completes the most scenes next, ties by use.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use cycles::blender::read_model;
use cycles::truth::{TestScene, find_root};
/// What gate 1 already shades: a lone diffuse or emission, the output.
const PORTED: [&str; 4] = ["ShaderNodeOutputMaterial", "ShaderNodeBsdfDiffuse", "ShaderNodeEmission", "WorldBackground"];
/// The closures a node makes (`kernel/svm/closure.h`'s cases), by node.
fn closure_of(idname: &str) -> &'static [&'static str] {
    match idname {
        "ShaderNodeBsdfDiffuse" => &["closure diffuse"],
        "ShaderNodeBsdfGlossy" | "ShaderNodeBsdfAnisotropic" => &["closure glossy (microfacet)"],
        "ShaderNodeBsdfGlass" | "ShaderNodeBsdfRefraction" => &["closure glass (microfacet transmission)"],
        "ShaderNodeBsdfTransparent" => &["closure transparent"],
        "ShaderNodeBsdfTranslucent" => &["closure translucent"],
        "ShaderNodeBsdfSheen" => &["closure sheen"],
        "ShaderNodeBsdfToon" => &["closure toon"],
        "ShaderNodeBsdfHair" | "ShaderNodeBsdfHairPrincipled" => &["closure hair"],
        "ShaderNodeEmission" => &["closure emission"],
        "ShaderNodeBackground" => &["closure background"],
        "ShaderNodeHoldout" => &["closure holdout"],
        "ShaderNodeSubsurfaceScattering" => &["closure subsurface"],
        "ShaderNodeVolumeScatter" | "ShaderNodeVolumeAbsorption" | "ShaderNodeVolumePrincipled" => &["closure volume"],
        "ShaderNodeMixShader" => &["closure mix"],
        "ShaderNodeAddShader" => &["closure add"],
        _ => &[],
    }
}
/// The Principled BSDF's lobes a material turns on: an input that is linked
/// or not at its default. `input` gives (default value as a float, linked).
fn principled_lobe(node: &impl PrincipledInput) -> Vec<&'static str> {
    let mut out = Vec::new();
    for (input, lobe) in [
        ("Metallic", "closure principled metal"),
        ("Transmission Weight", "closure principled transmission (glass)"),
        ("Coat Weight", "closure principled coat"),
        ("Sheen Weight", "closure principled sheen"),
        ("Emission Strength", "closure emission"),
        ("Subsurface Weight", "closure subsurface"),
        ("Alpha", "closure transparent"),
        ("Anisotropic", "closure principled anisotropy"),
        ("Thin Film Thickness", "closure principled thin film"),
    ] {
        if node.active(input) {
            out.push(lobe);
        }
    }
    out
}
trait PrincipledInput {
    fn active(&self, name: &str) -> bool;
}
struct Node<'a> {
    node: &'a phiphx::blend::shade::graph::BlendNode,
    graph: &'a phiphx::blend::shade::graph::BlendNodeGraph,
}
impl PrincipledInput for Node<'_> {
    fn active(&self, name: &str) -> bool {
        use phiphx::blend::shade::socket::BSocketDefault::*;
        let linked = self.graph.links.iter().any(|l| l.to == self.node.name && self.node.inputs.iter().any(|s| s.identifier == l.to_socket && s.name == name));
        let value = self.node.inputs.iter().find(|s| s.name == name).and_then(|s| match s.default.as_ref()? {
            Float(v) => Some(*v),
            _ => None,
        });
        let strength = name == "Alpha";
        linked || value.is_some_and(|v| if strength { v < 1.0 } else { v != 0.0 })
    }
}
fn root() -> PathBuf {
    find_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("the test scenes")
}
/// What one scene needs: node kinds and closures, and the other blockers.
#[derive(Default)]
struct Need {
    kind: BTreeSet<String>,
    closure: BTreeSet<String>,
    lamp: usize,
}
fn need_of(model: &cycles::blender::Model) -> Need {
    let mut need = Need { lamp: model.lights.len(), ..Need::default() };
    let mut seen = BTreeSet::new();
    for object in &model.objects {
        for material in object.materials.iter() {
            if !seen.insert(material.name.clone()) {
                continue;
            }
            for node in &material.graph.nodes {
                if matches!(node.idname.as_str(), "NodeFrame" | "NodeReroute") {
                    continue;
                }
                need.kind.insert(node.idname.clone());
                for closure in closure_of(&node.idname) {
                    need.closure.insert((*closure).into());
                }
                if node.idname == "ShaderNodeBsdfPrincipled" {
                    need.closure.extend(["closure principled diffuse".to_string(), "closure principled specular (GGX)".to_string()]);
                    for lobe in principled_lobe(&Node { node, graph: &material.graph }) {
                        need.kind.insert(lobe.replace("closure ", "Principled lobe: "));
                        need.closure.insert(lobe.into());
                    }
                }
            }
        }
    }
    let world = &model.world.background;
    need.kind.insert("WorldBackground".into());
    if world.texture.is_some() || world.missing.is_some() {
        need.kind.insert("ShaderNodeTexEnvironment".into());
    }
    if world.sky.is_some() {
        need.kind.insert("ShaderNodeTexSky".into());
    }
    need
}
fn main() {
    let root = root();
    let mut scene: Vec<(String, Need)> = Vec::new();
    let mut categories: Vec<String> = std::fs::read_dir(&root).unwrap().filter_map(Result::ok).filter(|e| e.path().is_dir()).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    categories.sort();
    for category in categories {
        for test in TestScene::list(&root, &category) {
            if test.exists() {
                if let Ok(model) = read_model(&test.blend) {
                    scene.push((format!("{}/{}", test.category, test.name), need_of(&model)));
                }
            }
        }
    }
    let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    for (_, need) in &scene {
        for kind in &need.kind {
            *count.entry(kind).or_default() += 1;
        }
    }
    // Greedy order over the scenes with no lamp (gate 2's own).
    let mut ported: BTreeSet<String> = PORTED.iter().map(|s| s.to_string()).collect();
    let eligible: Vec<&(String, Need)> = scene.iter().filter(|(_, n)| n.lamp == 0).collect();
    let done = |ported: &BTreeSet<String>| eligible.iter().filter(|(_, n)| n.kind.is_subset(ported)).count();
    let start = done(&ported);
    let mut order: Vec<(String, usize, usize)> = Vec::new();
    loop {
        let before = done(&ported);
        let mut best: Option<(usize, usize, String)> = None;
        for kind in count.keys().filter(|k| !ported.contains(**k)) {
            let mut trial = ported.clone();
            trial.insert(kind.to_string());
            let gain = done(&trial) - before;
            let key = (gain, count[kind], kind.to_string());
            if best.as_ref().is_none_or(|b| (key.0, key.1) > (b.0, b.1)) {
                best = Some(key);
            }
        }
        let Some((_, uses, kind)) = best else { break };
        ported.insert(kind.clone());
        order.push((kind, uses, done(&ported)));
    }
    let mut out = String::new();
    out.push_str("# Census of the test scenes\n\nGenerated by `cargo run -p cycles --release --example census`; do not edit.\n\n");
    let lamp = scene.iter().filter(|(_, n)| n.lamp > 0).count();
    out.push_str(&format!("{} scenes with a reference were read. {} have a lamp (gate 3), {} have none. Of those {}, {} need nothing beyond gate 1.\n\n", scene.len(), lamp, eligible.len(), eligible.len(), start));
    out.push_str("Kinds are Blender node idnames, `closure ...` rows are the closures a node makes (the Principled BSDF's lobes by which inputs are linked or off their default), `WorldBackground` is the world's colour. A world's own node graph is not exposed by phiphx (see PLAN.md); its image or sky counts as `ShaderNodeTexEnvironment` or `ShaderNodeTexSky`. Node groups are not expanded here (`ShaderNodeGroup` is counted, its inner nodes are not).\n\n");
    out.push_str("## Port order\n\n`Scenes` is how many of all scenes use the kind. `Complete` is how many of the lamp-free scenes need nothing but the kinds up to and including this row.\n\n| # | Kind | Scenes | Complete |\n|---|---|---|---|\n");
    for (index, (kind, uses, complete)) in order.iter().enumerate() {
        out.push_str(&format!("| {} | {} | {} | {} |\n", index + 1, kind, uses, complete));
    }
    out.push_str("\n## Closures, over all scenes\n\nA closure is made by a node of the table above (the Principled BSDF by its lobes); it is ported with that node.\n\n| Closure | Scenes |\n|---|---|\n");
    let mut closure: BTreeMap<&str, usize> = BTreeMap::new();
    for (_, need) in &scene {
        for kind in &need.closure {
            *closure.entry(kind).or_default() += 1;
        }
    }
    let mut ranked: Vec<_> = closure.iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (kind, uses) in ranked {
        out.push_str(&format!("| {kind} | {uses} |\n"));
    }
    out.push_str("\n## Use, over all scenes\n\n| Kind | Scenes |\n|---|---|\n");
    let mut ranked: Vec<_> = count.iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (kind, uses) in ranked {
        out.push_str(&format!("| {kind} | {uses} |\n"));
    }
    std::fs::write(Path::new(env!("CARGO_MANIFEST_DIR")).join("CENSUS.md"), out).unwrap();
    println!("{} scenes, {} lamp-free, {} already gate 1, {} after the order", scene.len(), eligible.len(), start, order.last().map_or(start, |o| o.2));
}
