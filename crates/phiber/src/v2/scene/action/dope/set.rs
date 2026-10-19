//! The Dope Sheet's key operators on `ctx.op.keys` (see the module above):
//! `all`, `none`, `invert`, `click:<row>:<frame>` (`click+` with Shift,
//! `click:none`), `box:<f0>,<f1>,<r0>,<r1>` and `box:end`, `delete`, and
//! the move's `grab:start[:<frame>]`, `grab:at:<frame>`, `grab:confirm`,
//! `grab:cancel`. Row 0 is the summary, then the listed objects.
//!
//! The dispatcher lives here; the two halves of the match that got long split
//! out alongside — `set_edit` for the keyframe edits (clean, duplicate,
//! snap, mirror, reshape, grab) and `set_jump` for the playhead one — so
//! each file sits under the 240-line read cap (`tests/level_structure.rs`).

use super::super::BSceneActions;
use super::{BKeySelection, BKeyTarget};
use crate::v2::state::{BKey as _, BPath, BState, CtxKey, OpKey, log, rule::BRule};

/// `ctx.op.keys` (see the module).
pub struct BKeyOpRule;

impl BRule for BKeyOpRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::Keys.path()
    }

    fn after(&self, _: &BPath, op: &str, s: &mut BState) {
        let Some(model) = crate::v2::scene::source::loaded_shared() else { return };
        let shown = BKeyTarget::shown(s, &model);
        let mut picks = BKeySelection::now(s);
        let every = |f: &dyn Fn(&mut BKeySelection, &str, f32), picks: &mut BKeySelection| {
            for name in &shown {
                BKeyTarget::columns(&model.actions, name).into_iter().for_each(|c| f(picks, name, c));
            }
        };
        let mut part = op.split(':');
        let head = (part.next().unwrap_or(""), part.next(), part.next());
        match head {
            ("all", ..) => every(&|p, n, c| p.add(n, c), &mut picks),
            ("none", ..) => picks = BKeySelection::default(),
            ("invert", ..) => every(&|p, n, c| p.flip(n, c), &mut picks),
            (click @ ("click" | "click+"), Some(row), Some(frame)) => {
                let (Ok(row), Ok(frame)) = (row.parse::<usize>(), frame.parse::<f32>()) else { return };
                let rows: Vec<&String> = if row == 0 { shown.iter().collect() } else { shown.get(row - 1).into_iter().collect() };
                let rows: Vec<&String> = rows.into_iter().filter(|n| BKeyTarget::columns(&model.actions, n).iter().any(|c| (c - frame).abs() < 5e-4)).collect();
                let on = rows.iter().any(|n| picks.has(n, frame));
                if click == "click" {
                    picks = BKeySelection::default();
                }
                for name in rows {
                    match click == "click+" && on {
                        true => picks.0.retain(|p| !(p.object == *name && (p.frame - frame).abs() < 5e-4)),
                        false => picks.add(name, frame),
                    }
                }
            }
            ("click", Some("none"), _) => picks = BKeySelection::default(),
            ("box", Some("end"), _) => return s.remove(CtxKey::KeyBox),
            ("box", Some(range), _) => {
                let v: Vec<f32> = range.split(',').filter_map(|v| v.parse().ok()).collect();
                let [f0, f1, r0, r1] = v[..] else { return };
                let base = match s.get(CtxKey::KeyBox).and_then(|b| b.split_once('#')) {
                    Some((_, base)) => BKeySelection::parse(base),
                    None => picks.clone(),
                };
                s.set(CtxKey::KeyBox, format!("{range}#{}", base.encode()));
                picks = base;
                let (rows, frames) = ((r0.min(r1) as usize)..=(r0.max(r1) as usize), f0.min(f1)..=f0.max(f1));
                for (i, name) in shown.iter().enumerate() {
                    if rows.contains(&0) || rows.contains(&(i + 1)) {
                        BKeyTarget::columns(&model.actions, name).into_iter().filter(|c| frames.contains(c)).for_each(|c| picks.add(name, c));
                    }
                }
            }
            ("delete", ..) => {
                let mut edits = BSceneActions::now(s);
                edits.keyed.extend(BKeyTarget::steps(&model.actions, &picks, None));
                edits.write(s);
                log::push(format!("Deleted {} keyframe column(s)", picks.0.len()));
                picks = BKeySelection::default();
            }
            // Jump moves the PLAYHEAD, not the keys — it writes `CtxKey::Frame`
            // and returns, so the picks never travel through `picks.write`.
            ("jump", ..) => return super::set_jump::jump(s, &picks),
            // Every remaining arm (clean / duplicate / snap / mirror / ipo /
            // easing / handle / grab:*) edits the keys. The helper owns its
            // own `picks` and writes them when it is done.
            _ => return super::set_edit::edit(head, s, &model, &shown, picks),
        }
        picks.write(s);
    }
}
