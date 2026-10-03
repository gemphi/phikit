//! The Properties header's Options popover — `PROPERTIES_PT_options`.
//!
//! Every host presents itself the same way: what it is on the left, what you
//! are looking for in the middle, how it is filtered or displayed on the
//! right (`part::search`, `part::filter`). Properties had the first two and
//! an empty right end, which is the one editor in Blender where that end is
//! not empty: it carries this panel.
//!
//! Structure only — which rows, and what they default to, are in `set.rs`.

pub mod set;

use crate::v2::state::val::{V_FALSE, V_TRUE};
use crate::v2::state::{BKey as _, BPath, BState, ButsKey};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BControl, BPopoverSide};

/// The key one option's value is stored under.
pub fn option_key(spec: &set::BPropsOptionSpec) -> BPath {
    ButsKey::Option(spec.key).path()
}

/// What an option answers: what the user set, or what Blender ships with.
pub fn value_of<'a>(s: &'a BState, spec: &set::BPropsOptionSpec) -> &'a str {
    s.get(&option_key(spec)).unwrap_or(spec.default)
}

/// The popover's rows — one `select_row` per option, under its own heading,
/// as `col.label` then `col.row().prop(…, expand=True)` draws it.
pub fn body(s: &BState) -> Vec<BView> {
    set::OPTIONS
        .iter()
        .map(|spec| BView::select_row(s, spec.heading, option_key(spec), spec.key, spec.items, spec.default))
        .collect()
}

/// The header's right-hand control: the icon that opens the popover.
pub fn view(s: &BState) -> BView {
    let key = ButsKey::OptionsOpen.path();
    let open = s.get(&key) == Some(V_TRUE);
    let toggle = BAction::Set(key.clone(), if open { V_FALSE } else { V_TRUE }.to_string());
    let close = BAction::Set(key, V_FALSE.to_string());
    let anchor = BView::Button(BControl::new().with_icon("filter").with_active(open).with_action(toggle));
    BView::popover(anchor, body(s), open, close, BPopoverSide::BelowEnd)
}
