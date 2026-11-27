//! A slider, as Reality Composer Pro's material inspector has one for its
//! roughness and its metal: a bar lit up to the value kept, a knob where
//! it stands, the value at the bar's end. Its choices are its steps, the
//! least at the left; a press on the bar keeps the step under it. The
//! knob stands at the step nearest what is kept, so a value between two
//! steps (a material's own) still has its place.

use super::kind::{BAR, DIM, GAP, LINE, PChoice};
use crate::pui::core::light::label;
use crate::pui::core::{PLight, PPaint, PPoint, PRect};

/// Draw a slider from a place across a width: where it ends, and where
/// each step can be pressed.
pub fn slider(choice: &[PChoice], now: &str, paint: &mut dyn PPaint, at: PPoint, wide: f32) -> (f32, Vec<PRect>) {
    let (line, gap, bar, dim) = (paint.size(LINE), paint.size(GAP), paint.size(BAR), paint.color(DIM));
    let value = |c: &PChoice| c.value.parse::<f32>().unwrap_or_default();
    let kept = now.parse::<f32>().ok();
    // Which step it stands at: the one nearest what is kept.
    let step = kept.and_then(|kept| choice.iter().enumerate().min_by(|a, b| (value(a.1) - kept).abs().total_cmp(&(value(b.1) - kept).abs())).map(|(i, _)| i));
    let said = kept.map(|k| format!("{k:.2}")).unwrap_or_default();
    let note = paint.measure(&said).max(paint.measure("0.00"));
    let track = PRect::new(at.x, at.y + (line - bar) / 2.0, (wide - note - gap / 2.0).max(1.0), bar);
    let count = choice.len().max(1) as f32;
    let each = track.size.width / count;
    label(paint, track, bar / 2.0, PLight::Soft, false, "");
    if let Some(step) = step {
        let reach = (step as f32 + 0.5) * each;
        label(paint, PRect::new(track.origin.x, track.origin.y, reach, bar), bar / 2.0, PLight::Strong, false, "");
        let knob = line * 0.7;
        let middle = PPoint::new(track.origin.x + reach, track.center().y);
        label(paint, PRect::new(middle.x - knob / 2.0, middle.y - knob / 2.0, knob, knob), knob / 2.0, PLight::Strong, true, "");
    }
    paint.text(PPoint::new(at.x + wide - note, at.y), &said, dim);
    let rect = (0..choice.len()).map(|i| PRect::new(track.origin.x + i as f32 * each, at.y, each, line)).collect();
    (at.y + line, rect)
}
