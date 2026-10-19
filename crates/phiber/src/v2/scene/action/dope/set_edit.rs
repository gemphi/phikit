//! The keyframe-edit arms of the Dope Sheet operator match: `clean` /
//! `duplicate` / `snap` / `mirror` / `ipo` / `easing` / `handle`, and the
//! `grab:*` sub-ops that move picked columns through time.
//!
//! Pulled out of `set.rs` so the dispatcher and the small picking arms fit
//! under the 240-line read cap (`tests/level_structure.rs`); the parent
//! threads in the shared setup — the model, the shown rows, the current
//! picks — so the arms read as they did in-line.

use super::super::BSceneActions;
use super::super::keys::{BKeyInsert, BKeyed};
use super::super::retime::BKeyStep;
use super::{BKeyGrab, BKeyPick, BKeySelection, BKeyTarget};
use crate::phiphx::scene::SceneModel;
use crate::v2::state::{BKey as _, BState, CtxKey, log};
use std::sync::Arc;

pub(super) fn edit(
    head: (&str, Option<&str>, Option<&str>),
    s: &mut BState,
    model: &Arc<SceneModel>,
    shown: &[String],
    mut picks: BKeySelection,
) {
    match head {
        // Key ▸ Clean Keyframes / Channel ▸ Clean Channels
        // (`ACTION_OT_clean`). Blender's one `channels` property becomes
        // the two `clean_fcurve` takes — `cleardefault = channels`,
        // `only_selected_keys = !channels` — so `clean:keys` simplifies
        // the selected columns and `clean:channels` walks every key and
        // drops a curve left holding one at its default (`action::clean`).
        // One step per `(action, slot)` the clean reaches: the walk is
        // per curve, so the step records the intent and runs it at replay.
        ("clean", kind, _) => {
            let channels = kind == Some("channels");
            if !channels && picks.0.is_empty() {
                return log::push("No keyframes selected to clean".to_string());
            }
            let mut edits = BSceneActions::now(s);
            let before = edits.keyed_actions(&super::super::set::BActionScene::raw(model.clone()));
            // Clean Channels reaches every shown object's slots; Clean
            // Keyframes only the ones the picks stand on.
            let reached: Vec<(String, f32)> = match channels {
                true => shown.iter().map(|n| (n.clone(), f32::NAN)).collect(),
                false => picks.0.iter().map(|p| (p.object.clone(), p.frame)).collect(),
            };
            let mut steps: Vec<super::super::clean::BKeyClean> = Vec::new();
            for (object, frame) in reached {
                for t in BKeyTarget::of(&before, &object) {
                    match steps.iter_mut().find(|c| c.action == t.action && c.slot == t.slot) {
                        Some(c) if !channels && !c.frames.contains(&frame) => c.frames.push(frame),
                        Some(_) => {}
                        None => steps.push(super::super::clean::BKeyClean {
                            action: t.action,
                            slot: t.slot,
                            frames: match channels {
                                true => Vec::new(),
                                false => vec![frame],
                            },
                            threshold: super::super::clean::DEFAULT_THRESHOLD,
                            channels,
                        }),
                    }
                }
            }
            if steps.is_empty() {
                return log::push("No channels to clean".to_string());
            }
            let cleaned = steps.len();
            edits.keyed.extend(steps.into_iter().map(BKeyStep::Clean));
            edits.write(s);
            // The picks may name frames the clean took off; the ones
            // still keyed stay picked, as Blender leaves the selection.
            let after = BSceneActions::now(s).keyed_actions(&super::super::set::BActionScene::raw(model.clone()));
            picks = BKeySelection(
                picks.0.into_iter().filter(|p| BKeyTarget::columns(&after, &p.object).iter().any(|c| (c - p.frame).abs() < 5e-4)).collect(),
            );
            log::push(match channels {
                true => format!("Cleaned {cleaned} channel group(s)"),
                false => format!("Cleaned {cleaned} channel group(s) of selected keys"),
            });
        }
        // Key ▸ Duplicate: an insert per `(action, slot)` the pick keys,
        // at the column's next free frame, replaying the curves' values at
        // the pick (`ACTION_OT_duplicate`, `space_action/action_edit.cc`'s
        // `actkeys_duplicate_exec` → `duplicate_fcurve_keys`,
        // `animation/keyframes_general.cc`). Divergence: Blender's array
        // stores two keys at the same frame and chains a grab for the user
        // to drop them where they want; phiber keeps one key per frame per
        // curve, so the copy lands on the next free frame and the picks
        // follow — the user grabs from there with G, as with any other
        // selection.
        ("duplicate", ..) => {
            if picks.0.is_empty() {
                return log::push("No keyframes selected to duplicate".to_string());
            }
            let mut edits = BSceneActions::now(s);
            let before = edits.keyed_actions(&super::super::set::BActionScene::raw(model.clone()));
            let mut new_picks: Vec<BKeyPick> = Vec::new();
            // Planned inserts in this batch, so a second duplicate on the
            // same column skips the frames the first one just filled.
            let mut planned: Vec<(String, String, f32)> = Vec::new();
            for pick in &picks.0 {
                let targets: Vec<BKeyTarget> = BKeyTarget::of(&before, &pick.object)
                    .into_iter()
                    .filter(|t| t.columns.iter().any(|c| (c - pick.frame).abs() < 5e-4))
                    .collect();
                if targets.is_empty() {
                    continue;
                }
                let occupied = |a: &str, sl: &str, f: f32, planned: &[(String, String, f32)]| {
                    let in_file = targets.iter().any(|t| t.action == a && t.slot == sl && t.columns.iter().any(|c| (c - f).abs() < 5e-4));
                    let in_batch = planned.iter().any(|(ea, es, ef)| ea == a && es == sl && (ef - f).abs() < 5e-4);
                    in_file || in_batch
                };
                // Column-wide next free frame: the first integer step on
                // which NO target of this column is keyed.
                let mut to = pick.frame + 1.0;
                while targets.iter().any(|t| occupied(&t.action, &t.slot, to, &planned)) {
                    to += 1.0;
                }
                let mut inserted = false;
                for target in &targets {
                    if target.slot.is_empty() {
                        continue;
                    }
                    let Some(action) = before.iter().find(|a| a.name == target.action) else { continue };
                    let Some(slot) = action.slots.iter().find(|s| s.identifier == target.slot) else { continue };
                    let channels: Vec<BKeyed> = action.curves[slot.curves.clone()]
                        .iter()
                        .filter(|c| c.keys.iter().any(|k| (k.co[0] - pick.frame).abs() < 5e-4))
                        .map(|c| BKeyed { path: c.path.clone(), index: c.index, value: c.evaluate(pick.frame) })
                        .collect();
                    if channels.is_empty() {
                        continue;
                    }
                    edits.keyed.push(BKeyStep::Insert(BKeyInsert {
                        action: target.action.clone(),
                        slot: target.slot.clone(),
                        frame: to,
                        channels,
                    }));
                    planned.push((target.action.clone(), target.slot.clone(), to));
                    inserted = true;
                }
                if inserted {
                    new_picks.push(BKeyPick { object: pick.object.clone(), frame: to });
                }
            }
            if new_picks.is_empty() {
                return log::push("No keyframes to duplicate".to_string());
            }
            edits.write(s);
            log::push(format!("Duplicated {} keyframe column(s)", new_picks.len()));
            picks = BKeySelection(new_picks);
        }
        // Key ▸ Snap: move the selected columns to a computed frame
        // (`ACTION_OT_snap`). The picks follow to where they land, so a
        // second snap acts on the moved keys.
        ("snap", Some(kind), _) => {
            if picks.0.is_empty() {
                return log::push("No keyframes selected to snap".to_string());
            }
            let fps = s.get(CtxKey::Fps).and_then(|v| v.parse::<f32>().ok()).filter(|f| *f > 1e-3).unwrap_or(24.0);
            let current = s.get(CtxKey::Frame).and_then(|v| v.parse::<f32>().ok()).unwrap_or(1.0);
            let markers: Vec<f32> = super::super::super::marker::BMarkerList::now(s).list.iter().map(|(f, _)| *f as f32).collect();
            let nearest = |f: f32| markers.iter().copied().min_by(|a, b| (a - f).abs().total_cmp(&(b - f).abs())).unwrap_or(f);
            let target: Box<dyn Fn(f32) -> f32> = match kind {
                "current" => Box::new(move |_| current),
                "nearest_frame" => Box::new(|f: f32| f.round()),
                "nearest_second" => Box::new(move |f: f32| (f / fps).round() * fps),
                "nearest_marker" => Box::new(move |f: f32| nearest(f)),
                _ => return,
            };
            let mut edits = BSceneActions::now(s);
            edits.keyed.extend(BKeyTarget::snap(&model.actions, &picks, target.as_ref()));
            edits.write(s);
            // The picks follow to where they landed.
            picks = BKeySelection(picks.0.iter().map(|p| BKeyPick { object: p.object.clone(), frame: target(p.frame) }).collect());
        }
        // Key ▸ Mirror (time axis): reflect the selected columns across a
        // pivot frame — the current frame, time zero, or the first
        // selected marker (`ACTION_OT_mirror`). A reflection is a move to
        // `2·pivot − frame`, so it rides the same per-column retime as Snap.
        ("mirror", Some(kind), _) => {
            if picks.0.is_empty() {
                return log::push("No keyframes selected to mirror".to_string());
            }
            let current = s.get(CtxKey::Frame).and_then(|v| v.parse::<f32>().ok()).unwrap_or(1.0);
            let pivot = match kind {
                "current" => current,
                "zero" => 0.0,
                "marker" => {
                    let Some(first) = super::super::super::marker::BMarkerList::selected(s).into_iter().min() else {
                        return log::push("No selected marker to mirror over".to_string());
                    };
                    first as f32
                }
                _ => return,
            };
            let mut edits = BSceneActions::now(s);
            edits.keyed.extend(BKeyTarget::snap(&model.actions, &picks, |f| 2.0 * pivot - f));
            edits.write(s);
            picks = BKeySelection(picks.0.iter().map(|p| BKeyPick { object: p.object.clone(), frame: 2.0 * pivot - p.frame }).collect());
        }
        // Key ▸ Interpolation / Easing / Handle Type: set a shape property
        // on the selected keys (`reshape.rs`). The picks stay as they were,
        // so a run of changes acts on the same keys.
        (field @ ("ipo" | "easing" | "handle"), Some(kind), _) => {
            let Some(shape) = super::super::reshape::BKeyShape::from_value(field, kind) else { return };
            if picks.0.is_empty() {
                return log::push("No keyframes selected to reshape".to_string());
            }
            let mut edits = BSceneActions::now(s);
            edits.keyed.extend(BKeyTarget::reshapes(&model.actions, &picks, shape));
            edits.write(s);
            return;
        }
        head @ ("grab", ..) => {
            picks = match super::set_grab::grab(head, s, model, picks) {
                Some(picks) => picks,
                None => return,
            };
        }
        _ => return,
    }
    picks.write(s);
}
