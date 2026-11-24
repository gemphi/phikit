//! The orientation ball in a 3D view's corner, as Reality Composer Pro
//! and Blender have it: the three ways of the scene (across, up, toward)
//! as they stand from the camera now, each a line from the middle to a
//! ball in the theme's colour for it, and the way back from each a smaller
//! fainter ball; on a round ground of its own. What is nearer the eye is
//! drawn over what is farther. A press on a ball looks along that way at
//! the scene; a drag on it turns the scene as a drag anywhere does.

use super::lens::{PCamera, dot};
use super::paint::{PColor, PPaint, PRect};

/// The theme's tokens: the ground, and the colour of each way.
pub const GROUND: &str = "scene.ball";
pub const WAY: [&str; 3] = ["scene.x", "scene.y", "scene.z"];

/// The three ways, and the way back from each.
const AXIS: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// Draw the ball in a square as the camera stands: where each of its six
/// balls is, and the way the eye looks along when it is pressed (from
/// that ball toward the middle).
pub fn ball(paint: &mut dyn PPaint, square: PRect, camera: &PCamera) -> Vec<(PRect, [f32; 3])> {
    let lens = camera.lens();
    let middle = square.center();
    let reach = square.size.width.min(square.size.height) / 2.0;
    paint.round(square, reach, paint.color(GROUND));
    // Each end: its way, its colour, whether it is the way back, and how
    // far from the eye it lies.
    let mut end: Vec<([f32; 3], PColor, bool, f32)> = Vec::new();
    for (index, axis) in AXIS.iter().enumerate() {
        let colour = paint.color(WAY[index]);
        end.push((*axis, colour, false, dot(*axis, lens.forward)));
        let back = axis.map(|c| -c);
        end.push((back, colour, true, dot(back, lens.forward)));
    }
    end.sort_by(|a, b| b.3.total_cmp(&a.3));
    let mut out = Vec::new();
    let long = reach * 0.66;
    for (way, colour, back, _) in end {
        let (x, y) = (dot(way, lens.right) * long, -dot(way, lens.up) * long);
        let size = match back {
            true => reach * 0.2,
            false => reach * 0.3,
        };
        let colour = match back {
            true => PColor { a: colour.a * 0.45, ..colour },
            false => colour,
        };
        // A way stands out of the middle as a line of small dots.
        if !back {
            let steps = 6;
            (1..steps).for_each(|step| {
                let t = step as f32 / steps as f32;
                let dot = reach * 0.07;
                paint.round(PRect::new(middle.x + x * t - dot / 2.0, middle.y + y * t - dot / 2.0, dot, dot), dot / 2.0, colour);
            });
        }
        let rect = PRect::new(middle.x + x - size / 2.0, middle.y + y - size / 2.0, size, size);
        paint.round(rect, size / 2.0, colour);
        paint.press(rect);
        out.push((rect, way.map(|c| -c)));
    }
    out
}
