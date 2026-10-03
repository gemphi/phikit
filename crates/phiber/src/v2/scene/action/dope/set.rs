//! The Dope Sheet's key operators on `ctx.op.keys` (see the module above):
//! `all`, `none`, `invert`, `click:<row>:<frame>` (`click+` with Shift,
//! `click:none`), `box:<f0>,<f1>,<r0>,<r1>` and `box:end`, `delete`, and
//! the move's `grab:start[:<frame>]`, `grab:at:<frame>`, `grab:confirm`,
//! `grab:cancel`. Row 0 is the summary, then the listed objects.

use super::super::BSceneActions;
use super::{BKeyGrab, BKeyPick, BKeySelection, BKeyTarget};
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
        match (part.next().unwrap_or(""), part.next(), part.next()) {
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
            ("grab", Some("start"), from) => {
                if picks.0.is_empty() {
                    return log::push("No keyframes selected to move".to_string());
                }
                let base = BSceneActions::now(s).keyed.len();
                BKeyGrab { from: from.and_then(|f| f.parse().ok()), base, picks: picks.clone() }.write(s);
                return;
            }
            ("grab", Some("at"), Some(frame)) => {
                let (Some(mut grab), Ok(frame)) = (BKeyGrab::now(s), frame.parse::<i32>()) else { return };
                let Some(from) = grab.from else {
                    grab.from = Some(frame);
                    return grab.write(s);
                };
                let by = (frame - from) as f32;
                let mut edits = BSceneActions::now(s);
                edits.keyed.truncate(grab.base);
                if by != 0.0 {
                    // The keys as they stood when the grab began.
                    let before = edits.keyed_actions(&super::super::set::BActionScene::raw(model.clone()));
                    edits.keyed.extend(BKeyTarget::steps(&before, &grab.picks, Some(by)));
                }
                edits.write(s);
                picks = BKeySelection(grab.picks.0.iter().map(|p| BKeyPick { object: p.object.clone(), frame: p.frame + by }).collect());
            }
            ("grab", Some("confirm"), _) => {
                let Some(grab) = BKeyGrab::now(s) else { return };
                s.remove(CtxKey::KeyGrab);
                let by = picks.0.first().zip(grab.picks.0.first()).map_or(0.0, |(a, b)| a.frame - b.frame);
                return log::push(format!("Moved {} keyframe column(s) by {by} frame(s)", picks.0.len()));
            }
            ("grab", Some("cancel"), _) => {
                let Some(grab) = BKeyGrab::now(s) else { return };
                s.remove(CtxKey::KeyGrab);
                let mut edits = BSceneActions::now(s);
                edits.keyed.truncate(grab.base);
                edits.write(s);
                picks = grab.picks;
            }
            _ => return,
        }
        picks.write(s);
    }
}

