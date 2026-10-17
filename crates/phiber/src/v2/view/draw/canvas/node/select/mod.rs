//! Which nodes are selected, which one is active, and what a click does to both.
//!
//! Ported from Blender 5.2.0, `editors/space_node/node_select.cc`
//! (`node_mouse_select`, `node_box_select_exec`, `node_select_single`) and the
//! `_template_node_select` keymap in `scripts/presets/keyconfig/keymap_data/
//! blender_default.py`.
//!
//! phiber's selection was ONE node name. Blender's is a set with one member
//! marked active, and the difference is not cosmetic — it is what makes a node
//! editor usable, because every operator from G to Delete acts on the set
//! (`get_transformed_nodes`: *"Keep only nodes that are selected or inside a
//! frame that is selected"*).
//!
//! ## Active is not just "the selected one"
//!
//! `NODE_SELECT` and `NODE_ACTIVE` are separate bits and both are consulted:
//!
//! * `compare_node_depth` ranks **active above selected above nothing** — three
//!   levels, which is why `draw_order` has three now and had two while there was
//!   no active node to speak of;
//! * `node_mouse_select`'s toggle tests `(flag & NODE_SELECT) && (flag &
//!   NODE_ACTIVE)` with the reason written in: *"Check active so clicking on an
//!   inactive node activates it."* So shift-clicking a node that is selected but
//!   not active makes it active rather than deselecting it, which is how you
//!   change which node of a group the next operator treats as the subject.
//!
//! ## What the keymap asks for
//!
//! `_template_node_select` binds `node.select` four ways, and the two that
//! matter here are the plain press (`select_passthrough=True`) and the shift
//! press (`toggle=True`). Passthrough is the subtle one: *"if the node is
//! already selected, `found = false`"*, so pressing an already-selected node
//! changes NOTHING. That is what lets a press-and-drag move a whole selection
//! instead of collapsing it to the node under the cursor first. The keymap then
//! adds a separate `CLICK` binding with `deselect_all=True` — a press and
//! release with no drag — so clicking a selected node still narrows the
//! selection to it, just on the release rather than the press.
//!
//! That pairing is why [`BClick::Press`] and [`BClick::Release`] are separate.


/// The selection, active member first.
///
/// One ordered list rather than a set plus a name, because the store holds one
/// value and position is the cheapest way to spell "this one is active" without
/// a second key that could disagree with the first.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BNodeSelection {
    pub names: Vec<String>,
}

impl BNodeSelection {
    pub fn of(names: &[&str]) -> Self {
        Self { names: names.iter().map(|n| n.to_string()).collect() }
    }

    /// `NODE_ACTIVE` — the one clicked last.
    pub fn active(&self) -> Option<&str> {
        self.names.first().map(String::as_str)
    }

    /// `NODE_SELECT`.
    pub fn holds(&self, name: &str) -> bool {
        self.names.iter().any(|n| n == name)
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// `node_select_single`: everything else off, this one on and active.
    fn only(name: &str) -> Self {
        Self { names: vec![name.to_string()] }
    }

    /// Add `name` and make it active, keeping the rest.
    fn with(&self, name: &str) -> Self {
        let mut names = vec![name.to_string()];
        names.extend(self.names.iter().filter(|n| *n != name).cloned());
        Self { names }
    }

    fn without(&self, name: &str) -> Self {
        Self { names: self.names.iter().filter(|n| *n != name).cloned().collect() }
    }
}

/// When the click happened. The keymap binds the two separately and they do
/// different things — see the module note on passthrough.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BClick {
    Press,
    Release,
}

/// What a click on `hit` does, or `None` if it changes nothing.
///
/// `hit` is the node under the cursor, `None` for empty canvas. `shift` is the
/// keymap's `toggle=True`.
pub fn clicked(
    selection: &BNodeSelection,
    hit: Option<&str>,
    shift: bool,
    when: BClick,
) -> Option<BNodeSelection> {
    match (hit, shift, when) {
        // `SEL_OP_XOR` with the active check: a selected-but-inactive node is
        // activated, not deselected.
        (Some(name), true, BClick::Press) => {
            let is_selected = selection.holds(name) && selection.active() == Some(name);
            Some(if is_selected { selection.without(name) } else { selection.with(name) })
        }
        // Passthrough: an already-selected node is left alone on the press, so
        // the drag that may follow carries the whole selection.
        (Some(name), false, BClick::Press) => {
            (!selection.holds(name)).then(|| BNodeSelection::only(name))
        }
        // `deselect_all=True` on the CLICK binding: a press and release with no
        // drag narrows a group to the one under the cursor.
        (Some(name), false, BClick::Release) => {
            (selection.names.len() > 1).then(|| BNodeSelection::only(name))
        }
        // Empty canvas: `node_deselect_all`, and only when something is there
        // to clear.
        (None, false, BClick::Release) => {
            (!selection.is_empty()).then(BNodeSelection::default)
        }
        // Shift on the release, or shift on empty canvas: nothing. Blender's
        // toggle bindings all fire on the press.
        _ => None,
    }
}

/// How a box select combines with what is already selected.
///
/// `eSelectOp` (`ED_select_utils.hh:39`), restricted to the three the node
/// keymap actually binds. `SEL_OP_AND` is bound nowhere for nodes and
/// `node_mouse_select` asserts it unreachable for picking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BBoxMode {
    /// `SEL_OP_SET` — the default drag. `SEL_OP_USE_PRE_DESELECT` is true for
    /// this one alone, so everything is deselected first.
    Set,
    /// `SEL_OP_ADD` — shift-drag.
    Add,
    /// `SEL_OP_SUB` — `select = (sel_op != SEL_OP_SUB)`, so the nodes inside
    /// come out.
    Sub,
}

impl BBoxMode {
    /// The mode a drag with these modifiers asks for.
    pub fn of(shift: bool, ctrl: bool) -> Self {
        match (shift, ctrl) {
            (_, true) => Self::Sub,
            (true, false) => Self::Add,
            (false, false) => Self::Set,
        }
    }
}

/// The selection after a box over `inside`.
///
/// `inside` is the nodes the box touched — **touched**, not contained:
/// `node_box_select_exec` uses `BLI_rctf_isect`, so a node half in the box is
/// in. That is worth stating because requiring containment is the commoner
/// choice elsewhere and would feel broken to anyone used to Blender.
///
/// The active node is whatever the box's first touched node is, in tree order,
/// since Blender's `tree_draw_order_update` runs afterwards off the flags and
/// the operator sets no active node of its own.
pub fn boxed(selection: &BNodeSelection, inside: &[&str], mode: BBoxMode) -> BNodeSelection {
    match mode {
        BBoxMode::Set => BNodeSelection::of(inside),
        BBoxMode::Add => {
            let mut next = selection.clone();
            for name in inside {
                next = next.with(name);
            }
            next
        }
        BBoxMode::Sub => {
            let mut next = selection.clone();
            for name in inside {
                next = next.without(name);
            }
            next
        }
    }
}

/// What a selection means for draw order, and whether a box touches a card.
pub mod rank;
/// The whole-graph selection operators (A / Alt A / Ctrl I).
pub mod op;
/// How the selection is spelled in one state value, and read back.
pub mod store;

pub use rank::{rank, rank_of_name, touches};
pub use store::{decode, encode, selection_of};

#[cfg(test)]
mod tests;
