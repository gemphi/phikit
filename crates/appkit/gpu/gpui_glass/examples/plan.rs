//! Prints the plan for one Regular panel in the middle of a window, as JSON:
//! the copy, the layer size and the four blocks as bytes. A renderer on any
//! side of a language boundary can replay it.
//!
//! `cargo run -p glass --example plan -- 800 600`

use gpui_glass::{GlassMaterial, GlassPlan, GlassRequest, GlassStyle};

fn main() {
    let mut side = std::env::args().skip(1).filter_map(|word| word.parse::<u32>().ok());
    let (width, height) = (side.next().unwrap_or(800), side.next().unwrap_or(600));
    let (w, h) = (width as f32, height as f32);
    let material = GlassMaterial::of(GlassStyle::Regular);
    let bounds = [w * 0.25, h * 0.25, w * 0.5, h * 0.5];
    let visible = [bounds[0], bounds[1], bounds[0] + bounds[2], bounds[1] + bounds[3]];
    let request = GlassRequest::of(&material, bounds, visible, [24.0; 4], 1.0, 1.0);
    let Some(plan) = GlassPlan::of(&request, width, height) else {
        println!("null");
        return;
    };
    let block = plan.stage.map(|stage| format!("{:?}", stage.bytes())).join(",");
    println!(
        "{{\"width\":{width},\"height\":{height},\"capture\":{:?},\"small\":{:?},\"bounds\":{:?},\"stage\":[{block}]}}",
        plan.capture, plan.small, bounds
    );
}
