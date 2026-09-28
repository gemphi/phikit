//! Picking a camera, a light or an empty in the viewport — they have no
//! faces for a ray to hit, so Blender picks them by what it draws for them
//! (`overlay_extra.cc` into the select buffer): a click within a few pixels
//! of one of an object's lines picks it, and box, circle and lasso select
//! take it when its origin is inside, as they take a mesh.

use super::BPickView;
use crate::phiphx::scene::SceneModel;
use crate::v2::part::outliner::key::BHidden;
use crate::v2::scene::edit::BSceneEdit;
use crate::v2::scene::extra::scene_extras_now;
use crate::v2::state::BState;
use glam::Vec3;

/// How near a click has to land to a drawn line to pick it, in pixels —
/// Blender's select buffer reads a small square round the pointer.
pub const REACH: f32 = 6.0;

/// One camera, light or empty the viewport can pick: its name now, its
/// drawn lines and its origin, in world units.
#[derive(Clone, Debug, PartialEq)]
pub struct BPickExtra {
    pub name: String,
    pub lines: Vec<(Vec3, Vec3)>,
    pub origin: Vec3,
}

impl BPickExtra {
    /// Every camera, light and empty drawn and selectable, where it stands.
    pub fn list(s: &BState, model: &SceneModel) -> Vec<Self> {
        let edit = BSceneEdit::now(s);
        let (hidden, unselectable) = (BHidden::now(s), BHidden::for_select(s));
        let mut out: Vec<Self> = Vec::new();
        for line in scene_extras_now(s, model) {
            let Some(file) = line.file_name(model) else { continue };
            let name = edit.renamed.iter().find(|(_, old)| old == file).map_or(file, |(new, _)| new.as_str());
            if edit.deleted.iter().any(|d| d == file) || hidden.hides(s, name) || unselectable.hides(s, name) {
                continue;
            }
            let segment = (Vec3::from(line.a), Vec3::from(line.b));
            match out.last_mut().filter(|last| last.name == name) {
                Some(last) => last.lines.push(segment),
                None => out.push(Self { name: name.to_string(), lines: vec![segment], origin: Vec3::from(line.origin) }),
            }
        }
        out
    }

    /// The one with a line nearest the canvas point `at` on a `size` canvas,
    /// within `reach` pixels.
    pub fn nearest<'a>(list: &'a [Self], view: &BPickView, at: (f32, f32), size: (f32, f32), reach: f32) -> Option<&'a str> {
        let on_canvas = |p: Vec3| view.project(p).map(|(su, sv)| ((su + 0.5) * size.0, (sv + 0.5) * size.1));
        list.iter()
            .filter_map(|extra| {
                let near = extra
                    .lines
                    .iter()
                    .filter_map(|(a, b)| Some(Self::distance(at, on_canvas(*a)?, on_canvas(*b)?)))
                    .fold(f32::INFINITY, f32::min);
                (near <= reach).then_some((near, extra.name.as_str()))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, name)| name)
    }

    /// Those whose origin lands on a `size` canvas where `inside` says.
    pub fn origins_where(list: &[Self], view: &BPickView, size: (f32, f32), inside: impl Fn(f32, f32) -> bool) -> Vec<String> {
        list.iter()
            .filter(|extra| view.project(extra.origin).is_some_and(|(su, sv)| inside((su + 0.5) * size.0, (sv + 0.5) * size.1)))
            .map(|extra| extra.name.clone())
            .collect()
    }

    /// From `p` to the segment `a`–`b`, in pixels.
    fn distance(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let length = dx * dx + dy * dy;
        let t = if length > 0.0 { (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / length).clamp(0.0, 1.0) } else { 0.0 };
        ((p.0 - a.0 - t * dx).powi(2) + (p.1 - a.1 - t * dy).powi(2)).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::scene::camera::BViewSetup;

    #[test]
    fn a_click_near_a_line_picks_its_object_and_an_origin_inside_is_taken() {
        let view = BPickView { yaw: 0.0, pitch: 0.0, pan: (0.0, 0.0), zoom: 1.0, setup: BViewSetup::default(), aspect: 1.0 };
        let lamp = BPickExtra { name: "Lamp".into(), lines: vec![(Vec3::new(-0.2, 0.0, 0.0), Vec3::new(0.2, 0.0, 0.0))], origin: Vec3::ZERO };
        let far = BPickExtra { name: "Far".into(), lines: vec![(Vec3::new(3.0, 0.0, 3.0), Vec3::new(3.2, 0.0, 3.0))], origin: Vec3::new(3.0, 0.0, 3.0) };
        let list = [lamp, far];
        let size = (400.0, 400.0);
        // Front view: the origin is the canvas centre, the line runs across it.
        assert_eq!(BPickExtra::nearest(&list, &view, (203.0, 204.0), size, REACH), Some("Lamp"));
        assert_eq!(BPickExtra::nearest(&list, &view, (200.0, 220.0), size, REACH), None, "too far from any line");
        assert_eq!(BPickExtra::origins_where(&list, &view, size, |x, y| (x - 200.0).abs() < 5.0 && (y - 200.0).abs() < 5.0), ["Lamp"]);
    }
}
