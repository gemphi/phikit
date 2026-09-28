//! Playback — the transport group of the Timeline header. Structure only:
//! the part, and the list it makes. Which buttons, in which order, is in
//! `set.rs`. Blender ref: `bl_ui/space_time.py::playback_controls`, the
//! middle group between its two `separator_spacer()`s.

pub mod panel;
pub mod set;
pub mod settings;
pub mod snap;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;

#[derive(Clone, Copy, Debug)]
pub struct BPlayback {
    pub host: BHostType,
}

impl BPlayback {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BPlayback {
    /// A settings popover beside a button (`row.popover(panel=…, text="")`):
    /// a chevron that opens the panel, as the Playback entry does.
    fn popover(self, label: &'static str, body: fn(&BState) -> Vec<BView>, s: &BState) -> BView {
        use crate::v2::state::{BKey as _, MenuKey, val::{V_FALSE, V_TRUE}};
        use crate::v2::view::prim::{BAction, BControl, BPopoverSide};
        let key = MenuKey::Panel(self.host.slug(), label).path();
        let open = s.get(&key) == Some(V_TRUE);
        let toggle = BAction::Set(key.clone(), if open { V_FALSE } else { V_TRUE }.to_string());
        let anchor = BView::Button(BControl::new().with_icon("chevron_down").with_tooltip(label).with_active(open).with_action(toggle));
        BView::popover(anchor, body(s), open, BAction::Set(key, V_FALSE.to_string()), BPopoverSide::Below)
    }
}

impl BViewItem for BPlayback {
    /// The transport in `playback_controls`' order, with the Auto
    /// Keyframing popover after the record button, the Time Jump popover
    /// after the jump arrows and the Playhead popover after the magnet.
    fn build(&self, s: &BState) -> BView {
        let mut row = Vec::new();
        // `playback_controls` opens with the Playback popover. The Timeline
        // has it among its menus already; every other editor's transport
        // (the Dope Sheet's and Graph Editor's footers) leads with it.
        if self.host != BHostType::Time {
            use crate::v2::part::menubar::{BMenubarItem, editor::set::PLAYBACK};
            row.push(BMenubarItem::new(self.host, PLAYBACK).build(s));
        }
        for button in set::BTransportButton::shown(s) {
            row.push(button.view(s));
            match button {
                set::BTransportButton::AutoKey => row.push(self.popover("Auto Keyframing", panel::autokey_body, s)),
                set::BTransportButton::TimeForward => row.push(self.popover("Time Jump", panel::jump_body, s)),
                set::BTransportButton::Snap => row.push(self.popover("Playhead", panel::snap_body, s)),
                _ => {}
            }
        }
        BView::row(row)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BPlaybackList {
    pub host: BHostType,
}

impl BPlaybackList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BPlaybackList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BPlayback::new(self.host).build(s)]
    }
}
