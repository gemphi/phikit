//! Selecting cards with the pointer, and sweeping a box.
//!
//! The other half of the canvas's gesture. `mod.rs` owns the press, move and
//! release and hands the ones that are not about links to these.
//!
//! The three-way split of one drag is Blender's: `node_box_select_invoke` is
//! bound to CLICK_DRAG with `tweak=True` and returns `OPERATOR_PASS_THROUGH`
//! when the event is over a node or a socket, so the same gesture is a link off
//! a socket, a move off a card, and a box off the canvas — decided by what is
//! under the press, in that order.

use super::BWire;
use super::super::select::{self, BBoxMode, BClick};
use crate::v2::state::{BKey as _, NodeKey};
use crate::v2::view::prim::BAction;
use gpui::{App, Pixels, Point, Window};

impl BWire {
    /// Open the Add search wired to a fresh link's dragged end — the link-drag
    /// search a link dropped on empty canvas launches (`wire::up`). Scoped to the
    /// shader editor, whose Add menu is the live one (`add::menu`).
    pub(super) fn open_add_search(
        &self,
        drag: &crate::v2::view::draw::canvas::node::link::BLinking,
        window: &mut Window,
        app: &mut App,
    ) {
        if self.host != crate::v2::host::BHostType::Shader {
            return;
        }
        let acts = crate::v2::view::draw::canvas::node::add::menu::open_actions(
            self.host,
            &drag.anchor.node,
            &drag.anchor.socket,
            drag.from_output,
        );
        for a in acts {
            (self.handler)(&a, window, app);
        }
    }

    /// A press that hit no socket: select the card under it, or begin a box.
    pub(super) fn press_on_canvas(
        &self,
        cursor: (f32, f32),
        size: (f32, f32),
        event: &gpui::MouseDownEvent,
        window: &mut Window,
        app: &mut App,
    ) {
        let hit = self.card_under(cursor, size);
        let shift = event.modifiers.shift;
        if let Some(next) =
            select::clicked(&self.selection, hit.as_deref(), shift, BClick::Press)
        {
            self.write(NodeKey::Selected(self.host), select::encode(&next), window, app);
        }
        // Off the cards entirely: remember where, so the release can sweep a
        // box between the two points.
        if hit.is_none() {
            let at = (cursor.0 / size.0, cursor.1 / size.1);
            self.write(NodeKey::Boxing(self.host), format!("{},{}", at.0, at.1), window, app);
        }
    }

    /// The card under the cursor, topmost first — `node_under_mouse_get` walks
    /// the draw order reversed, so the raised card answers.
    fn card_under(&self, cursor: (f32, f32), size: (f32, f32)) -> Option<String> {
        let cards = self.cards(size);
        cards
            .iter()
            .rev()
            .find(|card| {
                cursor.0 >= card.x
                    && cursor.0 <= card.x + card.width
                    && cursor.1 >= card.y
                    && cursor.1 <= card.y + card.height
            })
            .map(|card| card.name.clone())
    }

    /// Every card the box between `from` and the cursor touches.
    fn boxed_cards(&self, from: (f32, f32), cursor: (f32, f32), size: (f32, f32)) -> Vec<String> {
        let sweep = (
            from.0.min(cursor.0),
            from.1.min(cursor.1),
            from.0.max(cursor.0),
            from.1.max(cursor.1),
        );
        self.cards(size)
            .iter()
            .filter(|card| {
                select::touches(sweep, (card.x, card.y, card.x + card.width, card.y + card.height))
            })
            .map(|card| card.name.clone())
            .collect()
    }
}

impl BWire {
    /// A release with no link in hand: settle a box, or narrow a selection.
    pub(super) fn release_on_canvas(&self, at: Point<Pixels>, window: &mut Window, app: &mut App) {
        let Some((cursor, size)) = self.locate(at) else { return };
        let shift = false;
        match self.box_from {
            // A box was begun off the cards. If the pointer actually travelled
            // it is a box; if it did not, this is the keymap's plain CLICK and
            // it clears the selection.
            Some(from) => {
                let at = (cursor.0 / size.0, cursor.1 / size.1);
                let swept = (at.0 - from.0).abs() > BOX_SLOP || (at.1 - from.1).abs() > BOX_SLOP;
                let next = if swept {
                    let inside = self.boxed_cards(
                        (from.0 * size.0, from.1 * size.1),
                        cursor,
                        size,
                    );
                    let names: Vec<&str> = inside.iter().map(String::as_str).collect();
                    Some(select::boxed(&self.selection, &names, BBoxMode::of(shift, false)))
                } else {
                    select::clicked(&self.selection, None, shift, BClick::Release)
                };
                if let Some(next) = next {
                    self.write(NodeKey::Selected(self.host), select::encode(&next), window, app);
                }
                (self.handler)(&BAction::Reset(NodeKey::Boxing(self.host).path()), window, app);
            }
            // A release over a card with no box and no link: the CLICK
            // binding's `deselect_all`, which narrows a group to one.
            None => {
                let hit = self.card_under(cursor, size);
                if let Some(next) =
                    select::clicked(&self.selection, hit.as_deref(), shift, BClick::Release)
                {
                    self.write(NodeKey::Selected(self.host), select::encode(&next), window, app);
                }
            }
        }
    }
}

/// How far the pointer has to travel for a press-and-release to count as a box
/// rather than a click, as a fraction of the canvas.
///
/// Blender distinguishes these in the event system — `PRESS` and `CLICK_DRAG`
/// are different event values and the keymap binds them to different operators
/// — so it needs no threshold of its own. GPUI delivers one press and one
/// release and leaves the distinction to the listener, so there has to be one
/// here. A hundredth of the canvas is below the smallest deliberate sweep and
/// above any hand tremor.
const BOX_SLOP: f32 = 0.01;
