//! The action log: one line for everything that happens to the app.
//!
//! Blender's Info editor keeps a report of every operator that ran; this is
//! the same idea at the foot of the window. Every action that reaches
//! `BPhiberApp::handle_action`, and every event the app raises itself (a file
//! finishing its read), is turned into a sentence — "Zoomed to 1.7×",
//! "Switched to Shading", "Opened Cube.blend: 305 objects" — and kept here,
//! newest first, for the status bar to show.
//!
//! The log is derived, never authoritative: it describes a change by reading
//! the state before and after, so it cannot disagree with what the app did.
//! It lives beside the state, in a render-pass scope like the loaded scene,
//! because the view is built from state alone and a list this long has no
//! business being string-encoded into it.

use super::{BState, val};
use crate::v2::view::prim::BAction;
use std::{cell::RefCell, rc::Rc};

/// One line of the log, `at` when it was written: the session journal
/// (`app::journal`) files every line with its time and the gap since the last.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BLogEntry {
    /// Monotonic, so a view can tell new lines from old.
    pub seq: u64,
    pub text: String,
    pub at: web_time::Instant,
}

/// How many lines are kept. Older ones fall off the end.
pub const CAPACITY: usize = 500;

thread_local! {
    static LOG: RefCell<(u64, Rc<Vec<BLogEntry>>)> = RefCell::new((0, Rc::new(Vec::new())));
}

/// Append one line.
pub fn push(text: impl Into<String>) {
    LOG.with(|cell| {
        let mut log = cell.borrow_mut();
        log.0 += 1;
        let seq = log.0;
        let entries = Rc::make_mut(&mut log.1);
        entries.insert(0, BLogEntry { seq, text: text.into(), at: web_time::Instant::now() });
        entries.truncate(CAPACITY);
    });
}

/// Every line, newest first. Shared: a frame that reads it several times pays once.
pub fn entries() -> Rc<Vec<BLogEntry>> {
    LOG.with(|cell| Rc::clone(&cell.borrow().1))
}

/// The newest line, if there is one.
pub fn latest() -> Option<BLogEntry> {
    LOG.with(|cell| cell.borrow().1.first().cloned())
}

/// Forget everything.
pub fn clear() {
    LOG.with(|cell| *cell.borrow_mut() = (0, Rc::new(Vec::new())));
}

/// Describe `action` by what it changed between `before` and `after`, and
/// log it. Keys that change on every pointer movement — a menu opening under
/// the cursor, a hover switch — say nothing; they are not actions the user took.
pub fn record(action: &BAction, before: &BState, after: &BState) -> Option<String> {
    let line = describe(action, before, after)?;
    push(line.clone());
    Some(line)
}

/// The base name of a path.
fn file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

fn title(slug: &str) -> String {
    let mut out = String::new();
    for (i, word) in slug.split(['_', '-']).enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}

/// The sentence for one action, or `None` when it is not worth a line.
pub fn describe(action: &BAction, before: &BState, after: &BState) -> Option<String> {
    let path = match action {
        BAction::Set(p, _) | BAction::Adjust(p, _) | BAction::Toggle(p) | BAction::Open(p) | BAction::Reset(p) => p,
    };
    let encoded = path.encode();
    let key = encoded.strip_prefix("ctx.").unwrap_or(&encoded);
    // One viewport's own view key (`view3d.<scope>.<slot>.<field>`) says
    // what the shared one would.
    let shared = match key.split('.').collect::<Vec<_>>().as_slice() {
        ["view3d", _, _, "pan_x" | "pan_y"] => Some("view3d.pan".to_string()),
        ["view3d", _, _, field @ ("yaw" | "pitch" | "zoom" | "perspective" | "pivot" | "lens")] => Some(format!("view3d.{field}")),
        _ => None,
    };
    let key = shared.as_deref().unwrap_or(key);
    // G / R / S name themselves by mode (their rule clears the key).
    if let ("op.transform", BAction::Set(_, value)) = (key, action) { return crate::v2::scene::xform::BTransformModal::describe(value, before); }
    // Pointer-driven state: menus open as the pointer sweeps, hover arms
    // siblings. Neither is something the user did on purpose.
    if key.starts_with("menu.") || key.ends_with(".hover") || key.starts_with("window.") {
        return None;
    }
    let was = before.get_encoded(&encoded).unwrap_or("");
    let now = after.get_encoded(&encoded).unwrap_or("");
    if was == now && !matches!(action, BAction::Open(_)) {
        return None;
    }
    let num = |v: &str| v.parse::<f32>().ok();
    Some(match key {
        "view3d.zoom" => format!("Zoomed to {:.2}×", num(now).unwrap_or(1.0)),
        "view3d.yaw" | "view3d.pitch" => {
            let (yaw, pitch) = (crate::v2::scene::view_yaw(after), crate::v2::scene::view_pitch(after));
            format!("Orbited to {yaw:.0}° / {pitch:.0}°")
        }
        "view3d.pan" => "Panned the view".to_string(),
        "view3d.perspective" => {
            if now == val::V_TRUE { "Perspective view".to_string() } else { "Orthographic view".to_string() }
        }
        "view3d.view" => format!("View: {}", title(now)),
        "view3d.shading" => format!("Shading: {}", title(now)),
        "space" => format!("Switched to {}", title(now)),
        "modal" => {
            if now.is_empty() { "Closed the dialog".to_string() } else { format!("Opened {}", title(now)) }
        }
        "modal.maximized" => if now == val::V_TRUE { "Maximised the dialog" } else { "Restored the dialog" }.to_string(),
        "file.selected" => format!("Opening {}", file_name(now)),
        "scene" => format!("Scene: {now}"),
        "object" => format!("Selected {now}"),
        "mode" => format!("Mode: {}", title(now)),
        "tool" => format!("Tool: {}", title(now)),
        "frame" => format!("Frame {now}"),
        "playing" => if now == val::V_TRUE { "Playing" } else { "Paused" }.to_string(),
        "theme" => format!("Theme: {}", title(now)),
        "activity.tab" => format!("Activity: {}", title(now)),
        "activity.group" => {
            if now.is_empty() { "Closed the activity strip".to_string() } else { format!("Opened {} options", title(now)) }
        }
        "link" => format!("Opened {now} in the browser"),
        k if k.starts_with("buts.dropdown.") => return None,
        k if k.starts_with("buts.") => {
            use crate::v2::part::props::prop::BPropKind;
            let name = k.trim_start_matches("buts.");
            match crate::v2::part::props::set::prop_named(name) {
                Some(prop) => {
                    let shown = match prop.kind {
                        BPropKind::Enum(items) => items
                            .iter()
                            .find(|i| i.key == now)
                            .map_or(now, |i| i.label)
                            .to_string(),
                        _ => now.to_string(),
                    };
                    format!("{}: {shown}", prop.label)
                }
                None => format!("{}: {now}", title(name)),
            }
        }
        k if k.starts_with("zone.") => {
            let zone = title(k.trim_start_matches("zone."));
            if now == val::V_TRUE { format!("Showed the {zone} zone") } else { format!("Hid the {zone} zone") }
        }
        k if k.starts_with("load.") => return None,
        k if k.starts_with("area.") || k.contains(".size") || k.contains(".fraction") => "Resized".to_string(),
        k if k.contains(".expanded") => {
            let what = title(k.split('.').next().unwrap_or(k));
            if now == val::V_TRUE { format!("Expanded the {what} rail") } else { format!("Collapsed the {what} rail") }
        }
        k if k.ends_with(".tab") => format!("{}: {}", title(k.split('.').next().unwrap_or(k)), title(now)),
        _ => match action {
            BAction::Toggle(_) => format!("{}: {}", title(key), if now == val::V_TRUE { "on" } else { "off" }),
            BAction::Reset(_) => format!("Reset {}", title(key)),
            BAction::Open(_) => format!("Opened {}", title(key)),
            BAction::Set(..) | BAction::Adjust(..) => format!("{}: {}", title(key), now),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::{BKey as _, BPath, CtxKey, MenuKey};

    fn set(before: &BState, path: &[&'static str], value: &str) -> (BAction, BState) {
        let action = BAction::Set(BPath::owned(path.to_vec()), value.to_string());
        let mut after = before.clone();
        super::super::action::apply_action(&mut after, &action);
        (action, after)
    }

    #[test]
    fn a_zoom_reads_as_a_sentence_with_the_new_value() {
        let before = BState::new();
        let (a, after) = set(&before, &["view3d", "zoom"], "1.7");
        assert_eq!(describe(&a, &before, &after).as_deref(), Some("Zoomed to 1.70×"));
    }

    #[test]
    fn a_menu_opening_is_not_an_action_the_user_took() {
        let before = BState::new();
        let path = MenuKey::Top("topbar", "File").path();
        let action = BAction::Set(path, "true".to_string());
        let mut after = before.clone();
        super::super::action::apply_action(&mut after, &action);
        let a = action;
        assert_eq!(describe(&a, &before, &after), None);
    }

    #[test]
    fn an_action_that_changes_nothing_says_nothing() {
        let mut before = BState::new();
        before.set(CtxKey::Space, "shading");
        let (a, after) = set(&before, &["space"], "shading");
        assert_eq!(describe(&a, &before, &after), None);
    }

    #[test]
    fn the_log_keeps_the_newest_first_and_bounds_itself() {
        clear();
        for i in 0..(CAPACITY + 10) {
            push(format!("line {i}"));
        }
        let all = entries();
        assert_eq!(all.len(), CAPACITY);
        assert_eq!(all[0].text, format!("line {}", CAPACITY + 9));
        assert_eq!(latest().map(|e| e.seq), Some((CAPACITY + 10) as u64));
        clear();
    }
}
