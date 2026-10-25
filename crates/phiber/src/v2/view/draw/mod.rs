//! GPUI backend for `BRenderer`. Walks a `BView` tree into focused renderers.
//! One resolved UI style is shared by the whole render pass.

pub mod area;
mod backend;
pub mod atom;
pub mod color;
pub mod ctx;
pub mod flex;
pub mod host;
pub mod modal;
pub mod pair;
pub mod pane;
mod floor;
pub mod part;
pub mod popover;
pub mod prim;
pub mod region;
pub mod space;
pub mod surface;
pub mod zone;

// Every drawn view kind — atoms and prims were one family (sync133).
pub mod anchor;
pub mod bar;
pub mod body;
pub mod brand;
pub mod button;
pub mod canvas;
pub mod caption;
pub mod card;
mod slot;
pub mod chrome;
pub mod column;
pub mod ctrl;
pub mod field;
pub mod fold;
pub mod list;
pub mod log;
pub mod menu;
pub mod progress;
pub mod prose;
pub mod rail;
pub mod resize;
pub mod row;
pub mod scroller;
pub mod search;
pub mod crumb;
pub mod text_input;
pub mod sidebar;
pub mod sort;
pub mod split;
pub mod strip;
pub mod tab;
pub mod table;
pub mod avatar;
pub mod badge;
pub mod thumb;
pub mod toggle;
pub mod track;
pub mod tree;
pub mod vlabel;

use crate::v2::view::BView;
pub use backend::BGpuiRenderer;
pub use ctx::{BActionHandler, BRenderCtx, BResizeHandler};
pub use track::{BPickSpace, BTrack, BTrackWrite, BWheel};
use gpui::AnyElement;

pub(crate) fn walk(v: &BView, ctx: &BRenderCtx) -> AnyElement {
    match v {
        BView::Space { list, .. } => space::view(list, ctx),
        BView::Zone { kind, list } => zone::view(*kind, list, ctx),
        BView::Area { kind, child } => area::view(*kind, child, ctx),
        BView::Host { kind, list } => host::view(*kind, list, ctx),
        BView::Region { kind, list } => region::view(*kind, list, ctx),
        BView::Pane { kind, list } => pane::view(*kind, list, ctx),
        BView::Modal { kind, body } => modal::view(*kind, body, ctx),
        BView::Part { body, .. } => part::view(body, ctx),
        BView::Row { list } => prim::row(list, ctx),
        BView::ListRow { list, shade } => prim::list_row(list, *shade, ctx),
        BView::TableRow { cell_list, shade, action } => prim::table_row(cell_list, *shade, action.as_ref(), ctx),
        BView::TableHeader { cell_list } => prim::table_header(cell_list, ctx),
        BView::SortHeader { label, direction, align, action } => prim::sort_header(label, *direction, *align, action, ctx),
        BView::Card(card) => card::view(card, ctx),
        BView::TreeRow {
            depth,
            expanded,
            toggle,
            lead,
            icon,
            list,
            trail,
            shade,
            drag,
        } => prim::tree_row(
            *depth,
            *expanded,
            toggle.as_ref(),
            lead,
            icon,
            (list, trail),
            *shade,
            drag.as_ref(),
            ctx,
        ),
        BView::Indent { level } => prim::indent_resolved(*level, ctx),
        BView::Column { list } => prim::column(list, ctx),
        BView::Shared(view) => walk(view, ctx),
        BView::Split { list } => split::BSplit(list).render(ctx),
        BView::Label { text } => atom::label(text, ctx),
        BView::Pair { label, value } => pair::pair(label, value, ctx),
        BView::Prose { role, text } => prose::view(*role, text, ctx),
        BView::VLabel { text } => vlabel::view(text, ctx),
        BView::Field { text } => field::view(text, ctx),
        BView::SearchField { placeholder, value, editing, on_change, searches } => search::view(placeholder, value, (*editing, *searches), on_change, ctx),
        BView::Crumbs { trail, edit } => crumb::view(trail, edit, ctx),
        BView::Icon { name } => atom::icon(name, ctx),
        BView::Button(ctl) => atom::button(ctl, ctx),
        BView::Ctrl(ctrl) => ctrl::view(ctrl, ctx),
        BView::MenuItem(ctl) => menu::view(ctl, ctx),
        BView::MenuSep => menu::sep(ctx),
        BView::Spacer => atom::spacer(),
        BView::ZoneToggleGroup { controls } => toggle::group(controls, ctx),
        BView::Tab(ctl) => tab::view(ctl, ctx),
        BView::Rail(rail) => rail::view(rail, ctx),
        BView::MenuAnchor(ctl) => anchor::view(ctl, ctx),
        BView::Popover {
            anchor,
            body,
            open,
            close,
            side,
        } => popover::view(anchor, body, *open, close, *side, ctx),
        BView::WindowControls { leading } => caption::view(*leading, ctx),
        BView::Thumbnail(t) => thumb::view(t, ctx),
        BView::Grid { list } => thumb::grid(list, ctx),
        BView::Canvas { host, camera } => atom::canvas(*host, camera, ctx),
        BView::ProgressBar { fraction, label, wide } => progress::view(*fraction, label, *wide, ctx),
        BView::Log { open, toggle } => log::view(*open, toggle, ctx),
        BView::Avatar { path, size } => avatar::view(path, *size, ctx),
        BView::Badge { icon, count } => badge::view(icon, *count, ctx),
        BView::Track { track, share, text } => track::view(track, *share, text, ctx),
        BView::Wheel { wheel } => track::wheel_view(wheel, ctx),
        BView::Empty => atom::empty(),
    }
}
