//! What the Image editor's header offers — `IMAGE_HT_header` right of the
//! menus: the image data-block (`template_ID`), the gizmo and overlay
//! toggles, the render slot / view layer / pass (`template_image_layers`)
//! and the display channels — as the lists each dropdown picks from.

use crate::v2::host::BHostType;
use crate::v2::scene::render_job::BRenderShot;
use crate::v2::state::{BKey, BState, ImageKey, MenuKey, val::V_TRUE};
use crate::v2::view::prim::{BAction, BControl, BPick};
use crate::v2::view::BView;

/// The render result's data-block name, and `IMAGE_HT_header`'s New / Open.
pub const RENDER_RESULT: &str = "Render Result";
pub const NEW: &str = "New";
pub const OPEN: &str = "Open";
/// `RE_SLOT_COUNT`-style render slots, named as Blender names them.
pub const SLOTS: [&str; 8] = ["Slot 1", "Slot 2", "Slot 3", "Slot 4", "Slot 5", "Slot 6", "Slot 7", "Slot 8"];
/// The view layer a fresh scene has.
pub const VIEW_LAYER: &str = "ViewLayer";
/// The pass every render has, and the extra passes as `RenderPass.name`
/// spells them.
pub const COMBINED: &str = "Combined";
pub const DEPTH: &str = "Depth";
pub const NORMAL: &str = "Normal";
pub const DIFFUSE_COLOR: &str = "DiffCol";
/// `sima->display_channels`, in Blender's order.
pub const CHANNELS: [&str; 6] = ["Color and Alpha", "Color", "Alpha", "Red", "Green", "Blue"];
pub const CHANNEL_DEFAULT: &str = "Color";

/// The passes the render at hand can show: Combined, then whichever extras
/// the shot carries.
pub fn passes(shot: Option<&BRenderShot>) -> Vec<&'static str> {
    let mut out = vec![COMBINED];
    if let Some(shot) = shot {
        if !shot.depth.is_empty() {
            out.push(DEPTH);
        }
        if !shot.normal.is_empty() {
            out.push(NORMAL);
        }
        if !shot.albedo.is_empty() {
            out.push(DIFFUSE_COLOR);
        }
    }
    out
}

/// The value a key holds, or its default.
pub fn chosen<'a>(s: &'a BState, key: impl BKey, default: &'a str) -> &'a str {
    s.get(&key.path()).filter(|v| !v.is_empty()).unwrap_or(default)
}

/// One header dropdown: the anchor shows the choice, the popover lists the
/// options and marks the chosen one, as `layout.prop(…, text="")` does.
pub fn dropdown(s: &BState, host: BHostType, name: &'static str, icon: &'static str, key: impl BKey, options: &[&'static str], default: &'static str) -> BView {
    let chosen = chosen(s, &key, default).to_string();
    let pick = BPick::new(MenuKey::Top(host.slug(), name).path(), BControl::new().with_icon(icon).with_label(chosen.as_str()));
    options.iter().fold(pick, |pick, option| pick.item(option, *option == chosen, BAction::Set(key.path(), option.to_string()))).view(s)
}

/// A header toggle icon: lit while on.
pub fn toggle(s: &BState, key: ImageKey, icon: &'static str, tooltip: &'static str) -> BView {
    let on = s.get(&key.path()) != Some(crate::v2::state::val::V_FALSE);
    // A header toggle is a button, as the switcher beside it is.
    BView::Button(
        BControl::new()
            .with_icon(icon)
            .with_tooltip(tooltip)
            .with_active(on)
            .with_action(BAction::Set(key.path(), if on { crate::v2::state::val::V_FALSE } else { V_TRUE }.to_string())),
    )
}
