//! The UV editor over the objects it shows — `ED_space_image_show_uvedit`:
//! with the 3D viewport in Edit Mode on meshes that carry a UV map, the UV
//! editor edits them — its header grows the selection settings left of the
//! menus and the pivot, snapping and proportional editing after the spacer,
//! and the layout draws with its faces and vertices; in Object Mode it only
//! shows the selected objects' UVs, greyed (`show_uv`).

pub mod set;

use self::set::{BUvSetting, FALLOFF, ISLAND, OFF, ON, PIVOT, PROPORTIONAL, SELECT_DEFAULT, SELECT_MODES, SNAP, SNAP_ELEMENT, STICKY, SYNC};
use crate::phiphx::scene::SceneUvMap;
use crate::v2::host::BHostType;
use crate::v2::part::modebar::active_mode;
use crate::v2::scene::edit::BSceneEdit;
use crate::v2::scene::select::BSelection;
use crate::v2::state::{BKey as _, BState, CtxKey, MenuKey, UvKey};
use crate::v2::view::prim::{BAction, BControl, BPick};
use crate::v2::view::{BView, BViewItem};
use std::sync::Arc;

/// The 3D viewport's Edit Mode, as its mode menu names it.
pub const EDIT_MODE: &str = "Edit Mode";

/// What the UV editor shows: the UV maps of the active and selected mesh
/// objects, and whether they are being edited.
#[derive(Clone, Debug, Default)]
pub struct BUvShown {
    pub edit: bool,
    pub maps: Vec<Arc<SceneUvMap>>,
}

impl BUvShown {
    pub fn now(s: &BState) -> Self {
        let edit = active_mode(s, BHostType::View3D) == EDIT_MODE;
        let mut names = BSelection::now(s).list;
        if let Some(active) = s.get(CtxKey::Object).filter(|o| !o.is_empty() && !names.iter().any(|n| n == o)) {
            names.insert(0, active.to_string());
        }
        // The file's name for each: a renamed object's map is under its old one.
        let scene = BSceneEdit::now(s);
        let file_name = |now: &str| scene.renamed.iter().find(|(new, _)| new == now).map_or(now.to_string(), |(_, old)| old.clone());
        let maps = crate::v2::scene::source::with_loaded(|model| {
            let Some(model) = model else { return Vec::new() };
            names
                .iter()
                .filter_map(|name| {
                    let i = model.objects.iter().position(|o| o.name == file_name(name))?;
                    model.uv_maps.get(i).filter(|map| !map.is_empty()).cloned()
                })
                .collect()
        });
        Self { edit, maps }
    }

    /// `show_uvedit`: Edit Mode on something with UVs.
    pub fn edits(&self) -> bool {
        self.edit && !self.maps.is_empty()
    }
}

/// The header's UV settings, as `IMAGE_HT_header` draws them while editing.
#[derive(Clone, Copy, Debug)]
pub struct BUvHeader {
    pub host: BHostType,
}

impl BUvHeader {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }

    fn value<'a>(s: &'a BState, setting: &BUvSetting) -> &'a str {
        s.get(&setting.key.path()).filter(|v| !v.is_empty()).unwrap_or(setting.default)
    }

    /// An on / off setting as a lit icon.
    fn toggle(s: &BState, setting: &BUvSetting) -> BView {
        let on = Self::value(s, setting) == ON;
        let next = if on { OFF } else { ON };
        BView::Button(BControl::new().with_icon(setting.icon).with_tooltip(setting.tooltip).with_active(on).with_action(BAction::Set(setting.key.path(), next.to_string())))
    }

    /// An enum setting as an icon-only dropdown (`icon_only=True`).
    fn pick(&self, s: &BState, setting: &BUvSetting) -> BView {
        let chosen = Self::value(s, setting);
        let pick = BPick::new(MenuKey::Top(crate::v2::area::current::menu_scope(s, self.host), setting.tooltip).path(), BControl::new().with_icon(setting.icon).with_tooltip(setting.tooltip));
        setting.options.iter().fold(pick, |pick, option| pick.item(option, *option == chosen, BAction::Set(setting.key.path(), option.to_string()))).view(s)
    }

    /// Left of the menus: UV sync selection, the select mode — the mesh's
    /// while syncing (`template_edit_mode_selection`), the UV one else —
    /// island selection and sticky selection.
    pub fn select(&self, s: &BState) -> BView {
        if !BUvShown::now(s).edits() {
            return BView::row(Vec::new());
        }
        let key = if Self::value(s, &SYNC) == ON { UvKey::MeshSelectMode } else { UvKey::SelectMode };
        let mode = s.get(&key.path()).filter(|v| !v.is_empty()).unwrap_or(SELECT_DEFAULT);
        let mut list = vec![Self::toggle(s, &SYNC)];
        list.extend(SELECT_MODES.iter().map(|(label, icon)| {
            BView::Button(BControl::new().with_icon(*icon).with_tooltip(*label).with_active(mode == *label).with_action(BAction::Set(key.path(), label.to_string())))
        }));
        list.push(Self::toggle(s, &ISLAND));
        list.push(self.pick(s, &STICKY));
        BView::row(list)
    }

    /// After the spacer (`draw_xform_template`): the pivot, snapping and
    /// its element, proportional editing and its falloff.
    pub fn transform(&self, s: &BState) -> BView {
        if !BUvShown::now(s).edits() {
            return BView::row(Vec::new());
        }
        BView::row(vec![self.pick(s, &PIVOT), Self::toggle(s, &SNAP), self.pick(s, &SNAP_ELEMENT), Self::toggle(s, &PROPORTIONAL), self.pick(s, &FALLOFF)])
    }
}

/// The part left of the menus.
#[derive(Clone, Copy, Debug)]
pub struct BUvSelect(pub BHostType);

/// The part after the spacer.
#[derive(Clone, Copy, Debug)]
pub struct BUvTransform(pub BHostType);

impl BViewItem for BUvSelect {
    fn build(&self, s: &BState) -> BView {
        BUvHeader::new(self.0).select(s)
    }
}

impl BViewItem for BUvTransform {
    fn build(&self, s: &BState) -> BView {
        BUvHeader::new(self.0).transform(s)
    }
}
