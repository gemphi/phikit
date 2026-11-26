//! Materials: the kit's table of them, each drawn as a ball, its pattern
//! standing out and its metal mirroring the sky. The balls are kept as
//! one picture beside the system's temporary files, to look at.

use crate::pui::core::{PCamera, PMaterial, PPattern, PRaster, PSolid, PSun};
use crate::service::PFolder;

fn table() -> Vec<PMaterial> {
    let text = std::fs::read_to_string(format!("{}/material.tsv", PFolder::shared())).expect("the kit's materials");
    PMaterial::table(&text)
}

/// A ball of a material, drawn on a ground, so many points a side: its
/// blue, green, red, solid points.
pub(crate) fn ball(material: &PMaterial, side: u32) -> Vec<u8> {
    let (triangle, normal) = PMaterial::ball(96);
    let camera = PCamera { eye: [0.0, 0.35, 2.3], at: [0.0, 0.0, 0.0], up: [0.0, 1.0, 0.0], fov: 30.0, near: 0.01 };
    let sun = PSun { way: [0.5, 0.9, 0.7], key: 0.85, sky: [0.92, 0.93, 0.95], ground: [0.32, 0.3, 0.28], shine: 0.0, gloss: 1.0 };
    let mut raster = PRaster::new(side * 2, side * 2, [242, 240, 238, 255]);
    raster.draw(&camera, &[PSolid { triangle: &triangle, normal: &normal, colour: material.colour, material: Some(material) }], &sun);
    raster.smooth(2).2
}

#[test]
fn sixteen_materials_each_a_ball_its_pattern_showing() {
    let all = table();
    assert_eq!(all.len(), 16, "{:?}", all.iter().map(|m| &m.name).collect::<Vec<_>>());
    let side = 192u32;
    let mut sheet = vec![255u8; (side * 4 * side * 4 * 4) as usize];
    let drawn: Vec<Vec<u8>> = all.iter().map(|material| ball(material, side)).collect();
    for (index, pixel) in drawn.iter().enumerate() {
        // Its points, row by row, into the sheet of four by four.
        let (cx, cy) = ((index % 4) as u32 * side, (index / 4) as u32 * side);
        for row in 0..side {
            let from = (row * side * 4) as usize;
            let to = (((cy + row) * side * 4 + cx) * 4) as usize;
            sheet[to..to + side as usize * 4].copy_from_slice(&pixel[from..from + side as usize * 4]);
        }
    }
    let rgba: Vec<u8> = sheet.chunks(4).flat_map(|p| [p[2], p[1], p[0], 255]).collect();
    let file = std::env::temp_dir().join("appkit-material-ball.png");
    image::save_buffer(&file, &rgba, side * 4, side * 4, image::ColorType::Rgba8).expect("the sheet");
    for (material, pixel) in all.iter().zip(&drawn) {
        // A pattern shows: across the ball's middle, along or down, it is
        // not one smooth sweep.
        let light = |x: u32, y: u32| (0..3).map(|c| pixel[((y * side + x) * 4) as usize + c] as i32).sum::<i32>();
        let along: Vec<i32> = (side / 3..side * 2 / 3).map(|x| light(x, side / 2)).collect();
        let down: Vec<i32> = (side / 3..side * 2 / 3).map(|y| light(side / 2, y)).collect();
        let jump = |line: &[i32]| line.windows(2).map(|w| (w[1] - w[0]).abs()).max().unwrap_or(0);
        let (jump, shade) = (jump(&along).max(jump(&down)), along);
        match material.pattern {
            PPattern::Plain | PPattern::Wood | PPattern::Brushed => {}
            _ => assert!(jump > 2, "{} shows no pattern: {shade:?}", material.name),
        }
    }
    // Chrome mirrors: brighter where it faces the sky than the ground.
    let chrome = ball(all.iter().find(|m| m.name == "chrome").expect("chrome"), side);
    let at = |y: u32| chrome[((y * side + side / 2) * 4 + 1) as usize];
    assert!(at(side / 4) > at(side * 3 / 4), "{} {}", at(side / 4), at(side * 3 / 4));
}
