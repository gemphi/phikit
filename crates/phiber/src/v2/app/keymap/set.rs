//! The bindings: the menus of the editor under the pointer first (Blender's
//! region keymap, `wm_event_do_handlers`), then the keys Blender binds
//! outside its menus, then every other menu. When two items share a key —
//! `Home` frames all in several editors, `A` selects all in more than one —
//! the editor under the pointer wins, and without one the first host in
//! `PRIORITY`: the top bar first, the 3D viewport next.

use super::BKeystroke;
use crate::v2::host::BHostType;
use crate::v2::part::menubar::item::{BMenuItem, BMenuKind, BMenuSpec};
use crate::v2::part::menubar::tree::{item_path, leaf_action};
use crate::v2::part::playback::set::BTransportButton;
use crate::v2::state::{BKey as _, BState, CtxKey, ModalKey, OpKey, View3dKey, val::{V_EMPTY, V_FALSE, V_REVERSE, V_TRUE}};
use crate::v2::view::prim::BAction;

/// One binding: a keystroke, what it runs (given the state), its name.
pub struct BBinding {
    pub stroke: BKeystroke,
    pub action: Box<dyn Fn(&BState) -> BAction>,
    pub name: String,
}

/// Which editor's menus win a shared key.
pub const PRIORITY: &[BHostType] = &[
    BHostType::Topbar,
    BHostType::View3D,
    BHostType::Time,
    BHostType::Outliner,
    BHostType::Buts,
    BHostType::Image,
    BHostType::Uv,
    BHostType::File,
    BHostType::Asset,
    BHostType::Node,
    BHostType::Prefs,
];

fn walk(host: BHostType, trail: &mut Vec<&'static str>, items: &'static [BMenuItem], out: &mut Vec<BBinding>) {
    for item in items {
        match item.kind {
            BMenuKind::Sep | BMenuKind::Recent | BMenuKind::History | BMenuKind::Live(_) => {}
            BMenuKind::Sub(subs) => {
                trail.push(item.label);
                walk(host, trail, subs, out);
                trail.pop();
            }
            _ => {
                let Some(stroke) = BKeystroke::parse(item.shortcut) else { continue };
                if out.iter().any(|b| b.stroke == stroke) {
                    continue;
                }
                let action = leaf_action(host.slug(), trail, item);
                out.push(BBinding { stroke, action: Box::new(move |_| action.clone()), name: item_path(trail, item.label) });
            }
        }
    }
}

fn fixed(out: &mut Vec<BBinding>, label: &str, name: &str, action: impl Fn(&BState) -> BAction + 'static) {
    let stroke = BKeystroke::parse(label).expect("a binding label");
    if !out.iter().any(|b| b.stroke == stroke) {
        out.push(BBinding { stroke, action: Box::new(action), name: name.to_string() });
    }
}

/// Every binding, in the order a key is matched against them, the menus of
/// `over` — the editor under the pointer — first.
pub fn bindings(over: Option<BHostType>) -> Vec<BBinding> {
    let mut out = Vec::new();
    // Over the Dope Sheet's keys, G moves them and X and Delete delete them
    // (the Dope Sheet keymap), not the Marker menu's G or the Channel
    // menu's X, which belong to the marker band and the channel list.
    if over == Some(BHostType::Action) {
        let keys = |op: &'static str| move |_: &BState| BAction::Set(OpKey::Keys.path(), op.to_string());
        fixed(&mut out, "G", "transform.transform (TIME_TRANSLATE)", keys("grab:start"));
        fixed(&mut out, "X", "action.delete", keys("delete"));
        fixed(&mut out, "Delete", "action.delete", keys("delete"));
    }
    if let Some(host) = over.filter(|host| *host != BHostType::Topbar) {
        for menu in BMenuSpec::list_for(host) {
            walk(host, &mut vec![menu.label], menu.items, &mut out);
        }
    }
    // A node editor's Home, `NODE_OT_view_all`. Outside the menus because the
    // node editors have no View menu here yet — the framing existed and
    // nothing could ask for it, which is the gap this closes.
    if let Some(host) = over.filter(|host| {
        matches!(host, BHostType::Node | BHostType::Shader | BHostType::Compositor | BHostType::GeoNode)
    }) {
        fixed(&mut out, "Home", "node.view_all", move |_| {
            BAction::Set(OpKey::NodeView.path(), format!("{}:all", host.slug()))
        });
    }
    // The 3D Viewport's own key outside its menus: Alt Z, `view3d.toggle_xray`.
    if over == Some(BHostType::View3D) {
        fixed(&mut out, "Alt Z", "view3d.toggle_xray", |_| BAction::Set(OpKey::View3d.path(), "toggle_xray".to_string()));
    }
    // The NLA editor's Shift A, `nla.actionclip_add`: the Add menu opens on
    // Action ▸ and the actions (Blender opens a search over them).
    if over == Some(BHostType::Nla) {
        fixed(&mut out, "Shift A", "nla.actionclip_add", |_| BAction::Toggle(crate::v2::state::MenuKey::Top(crate::v2::host::nla::SLUG, "Add").path()));
    }
    // The keys Blender binds in `Screen` / `Frames` / `Window`, not in a menu.
    let step = |intent: &'static str| move |_: &BState| BAction::Set(CtxKey::FrameStep.path(), intent.to_string());
    fixed(&mut out, "Space", "screen.animation_play", |state| {
        let playing = BTransportButton::playing(state);
        BAction::Set(CtxKey::Playing.path(), if playing { V_FALSE } else { V_TRUE }.to_string())
    });
    fixed(&mut out, "Shift Ctrl Space", "screen.animation_play (reverse)", |_| BAction::Set(CtxKey::Playing.path(), V_REVERSE.to_string()));
    fixed(&mut out, "Escape", "screen.animation_cancel / wm.window_close", |state| {
        // A plain Esc anywhere stops a render under way, whatever else has
        // the keyboard — Blender's `G.is_break`, set in `wm_event_do_handlers`
        // for exactly this ("we can always cancel operations such as
        // rendering ... no matter which operation is currently handling
        // events"), which the render's `test_break` reads.
        if crate::v2::scene::render_job::rendering() {
            BAction::Set(OpKey::Render.path(), "cancel".to_string())
        } else if crate::v2::scene::action::dope::BKeyGrab::active(state) {
            BAction::Set(OpKey::Keys.path(), "grab:cancel".to_string())
        } else if crate::v2::scene::grab::BMarkerGrab::active(state) {
            // A marker grab is put back (`ed_marker_move_cancel`).
            BAction::Set(OpKey::Marker.path(), "grab:cancel".to_string())
        } else if BTransportButton::playing(state) {
            BAction::Set(CtxKey::Playing.path(), V_FALSE.to_string())
        } else if crate::v2::scene::boxsel::armed(state) {
            // The armed box gesture is cancelled (`WM_gesture_end`).
            BAction::Set(View3dKey::BoxSelect.path(), V_FALSE.to_string())
        } else if crate::v2::scene::circlesel::BCircle::armed(state) {
            BAction::Set(View3dKey::CircleSelect.path(), V_FALSE.to_string())
        } else if state.get(View3dKey::RenderBorder) == Some(V_TRUE) {
            BAction::Set(View3dKey::RenderBorder.path(), V_FALSE.to_string())
        } else if state.get(crate::v2::state::ImageKey::BorderArm) == Some(V_TRUE) {
            BAction::Set(crate::v2::state::ImageKey::BorderArm.path(), V_FALSE.to_string())
        } else {
            BAction::Set(ModalKey::Open.path(), V_EMPTY.to_string())
        }
    });
    fixed(&mut out, "Left", "screen.frame_offset −1", step("-1"));
    fixed(&mut out, "Right", "screen.frame_offset +1", step("+1"));
    fixed(&mut out, "Up", "screen.keyframe_jump", step("key+"));
    fixed(&mut out, "Down", "screen.keyframe_jump (previous)", step("key-"));
    fixed(&mut out, "Shift Left", "screen.frame_jump (start)", step("start"));
    fixed(&mut out, "Shift Right", "screen.frame_jump (end)", step("end"));
    fixed(&mut out, "Delete", "object.delete", |_| BAction::Set(OpKey::SceneEdit.path(), "delete".to_string()));
    // Then the menus, the top bar's first.
    for host in PRIORITY {
        for menu in BMenuSpec::list_for(*host) {
            walk(*host, &mut vec![menu.label], menu.items, &mut out);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::action::apply_action;

    fn stroke(label: &str) -> BKeystroke {
        BKeystroke::parse(label).unwrap()
    }

    #[test]
    fn the_menus_shortcuts_are_the_bindings_and_the_top_bar_wins_a_shared_key() {
        let all = bindings(None);
        let find = |label: &str| all.iter().find(|b| b.stroke == stroke(label)).map(|b| b.name.clone());
        assert_eq!(find("F12").as_deref(), Some("Render ▸ Render Image"));
        assert_eq!(find("F11").as_deref(), Some("Render ▸ View Render"));
        assert_eq!(find("Ctrl F11").as_deref(), Some("Render ▸ View Animation"));
        assert_eq!(find("Ctrl Z").as_deref(), Some("Edit ▸ Undo"));
        assert_eq!(find("Ctrl Q").as_deref(), Some("File ▸ Quit"));
        assert_eq!(find("Ctrl S").as_deref(), Some("File ▸ Save"));
        assert_eq!(find("Ctrl Alt S").as_deref(), Some("File ▸ Save Incremental"));
        assert_eq!(find("Numpad 7").as_deref(), Some("View ▸ Viewpoint ▸ Top"));
        assert_eq!(find("Home").as_deref(), Some("View ▸ Frame All"), "the 3D viewport's, not the Outliner's or the Timeline's");
        assert_eq!(find("A").as_deref(), Some("Select ▸ All"));
        assert_eq!(find("Shift D").as_deref(), Some("Object ▸ Duplicate Objects"));
        assert!(find("Space").is_some() && find("Escape").is_some() && find("Left").is_some());
        // No key is bound twice.
        let mut seen = std::collections::HashSet::new();
        for b in &all {
            assert!(seen.insert(b.stroke.clone()), "{:?} bound twice", b.stroke);
        }
    }

    #[test]
    fn the_editor_under_the_pointer_takes_a_shared_key_first() {
        let find = |over: Option<BHostType>, label: &str| bindings(over).into_iter().find(|b| b.stroke == stroke(label)).map(|b| b.name);
        assert_eq!(find(Some(BHostType::Time), "G").as_deref(), Some("Marker ▸ Move Marker"));
        assert_eq!(find(Some(BHostType::Time), "F2").as_deref(), Some("Marker ▸ Rename Marker"), "over the Timeline F2 renames a marker");
        assert_eq!(find(Some(BHostType::View3D), "F2").as_deref(), find(None, "F2").as_deref(), "elsewhere the top bar's Rename Active Item");
        assert_eq!(find(Some(BHostType::Time), "Space").as_deref(), Some("screen.animation_play"), "a key the editor does not bind falls through");
    }

    #[test]
    fn space_plays_and_pauses_and_escape_stops_then_closes() {
        let mut state = BState::new();
        let (play, _) = super::super::action_for(&stroke("Space"), &state).unwrap();
        apply_action(&mut state, &play);
        assert_eq!(state.get(CtxKey::Playing), Some(V_TRUE));
        let (pause, _) = super::super::action_for(&stroke("Space"), &state).unwrap();
        apply_action(&mut state, &pause);
        assert_eq!(state.get(CtxKey::Playing), Some(V_FALSE));
        state.set(CtxKey::Playing, V_TRUE);
        state.set(ModalKey::Open, "settings");
        let (stop, _) = super::super::action_for(&stroke("Escape"), &state).unwrap();
        apply_action(&mut state, &stop);
        assert_eq!((state.get(CtxKey::Playing), state.get(ModalKey::Open)), (Some(V_FALSE), Some("settings")), "playing: Escape only stops");
        let (close, _) = super::super::action_for(&stroke("Escape"), &state).unwrap();
        apply_action(&mut state, &close);
        assert_eq!(state.get(ModalKey::Open), Some(""), "then it closes the dialog");
    }
}
