//! GPUI backend for `BRenderer`. Walks a `BView` tree into focused renderers.
//! One resolved UI style is shared by the whole render pass.

pub mod area;
mod backend;
pub mod atom;
pub mod color;
pub mod ctx;
pub mod host;
pub mod modal;
pub mod pane;
pub mod part;
pub mod popover;
pub mod prim;
pub mod region;
pub mod space;
pub mod surface;
pub mod zone;

use crate::v2::view::BView;
pub use backend::BGpuiRenderer;
pub use ctx::{BActionHandler, BRenderCtx, BResizeHandler};
use gpui::AnyElement;

pub(crate) fn walk(v: &BView, c: &BRenderCtx) -> AnyElement {
    match v {
        BView::Space { children, .. } => space::view(children, c),
        BView::Zone { kind, children } => zone::view(*kind, children, c),
        BView::Area { kind, child } => area::view(*kind, child, c),
        BView::Host { kind, children } => host::view(*kind, children, c),
        BView::Region { kind, children } => region::view(*kind, children, c),
        BView::Pane { kind, children } => pane::view(*kind, children, c),
        BView::Modal { kind, body } => modal::view(*kind, body, c),
        BView::Part { body, .. } => part::view(body, c),
        BView::Row { children } => prim::row(children, c),
        BView::ListRow { children, shade } => prim::list_row(children, *shade, c),
        BView::TableRow { cells, shade, action } => prim::table_row(cells, *shade, action.as_ref(), c),
        BView::TableHeader { cells } => prim::table_header(cells, c),
        BView::SortHeader { label, direction, align, action } => prim::sort_header(label, *direction, *align, action, c),
        BView::AccordionHeader { title, expanded, action } => prim::accordion_header(title, *expanded, action, c),
        BView::TreeRow {
            depth,
            expanded,
            toggle,
            icon,
            children,
            trail,
            shade,
        } => prim::tree_row(
            *depth,
            *expanded,
            toggle.as_ref(),
            icon,
            children,
            trail,
            *shade,
            c,
        ),
        BView::Indent { level } => prim::indent_resolved(*level, c),
        BView::Column { children } => prim::column(children, c),
        BView::Panel { role, children } => prim::panel(*role, children, c),
        BView::Label { text } => atom::label(text, c),
        BView::VLabel { text } => atom::vlabel::view(text, c),
        BView::Field { text } => atom::field::view(text, c),
        BView::SearchField { placeholder, value, on_change } => atom::search::view(placeholder, value, on_change, c),
        BView::Icon { name } => atom::icon(name, c),
        BView::Button(ctl) => atom::button(ctl, c),
        BView::MenuItem(ctl) => atom::menu::view(ctl, c),
        BView::MenuSep => atom::menu::sep(c),
        BView::Spacer => atom::spacer(),
        BView::ZoneToggleGroup { controls } => atom::toggle::group(controls, c),
        BView::Tab(ctl) => atom::tab::view(ctl, c),
        BView::Rail(rail) => atom::rail::view(rail, c),
        BView::MenuAnchor(ctl) => atom::anchor::view(ctl, c),
        BView::Popover {
            anchor,
            body,
            open,
            close,
            side,
        } => popover::view(anchor, body, *open, close, *side, c),
        BView::Canvas { host } => atom::canvas(*host, c),
        BView::ProgressBar { fraction, label } => atom::progress::view(*fraction, label, c),
        BView::Empty => atom::empty(),
    }
}
