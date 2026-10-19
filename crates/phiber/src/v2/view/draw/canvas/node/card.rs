//! The painter for a single node card — big enough to leave `set.rs`
//! with just `render()` under the 240-line read cap
//! (`tests/level_structure.rs`).

use super::*;


/// One node's card, at the place it is actually drawn.
///
/// `from` is the displacement already stored for this node, carried into the
/// gesture so a second drag continues from where the first left off instead of
/// snapping back.
///
/// Both the position AND the size are fractions of the canvas, because that is
/// the only thing expressible before the canvas is measured (`to_fraction`) —
/// and it is what makes the zoom real rather than cosmetic: zooming out shrinks
/// the card instead of only sliding it toward its neighbours.
///
/// What does NOT scale is the text, because `text_size` wants pixels and there
/// are none here. Blender stops drawing a node's label below a zoom threshold
/// rather than shrinking it to nothing, so this does the same, with the card's
/// own share of the canvas standing in for the zoom it cannot see
/// ([`LABEL_FLOOR`]).
pub(super) fn card(
    node: &BlendNode,
    at: (f32, f32),
    size: BCardFraction,
    from: (f32, f32),
    selected: bool,
    ctx: &BRenderCtx,
) -> AnyElement {
    let id = node.name.clone();
    // The card's size is a fraction of the CANVAS (`BCardFraction`), so it
    // belongs on the absolute wrapper — whose percentage dimensions resolve
    // against the sized, positioned canvas (`surface::base`) — not on this
    // inner card, which would resolve its own percentage width against the
    // auto-sized wrapper and collapse to nothing (the node graph drew its
    // sockets and wires but no card bodies at all). The inner card fills the
    // wrapper and splits its own height into the header and the body.
    let total = size.header + size.body;
    let header_share = if total > 0.0 { (size.header / total).clamp(0.0, 1.0) } else { 1.0 };
    let mut card = div()
        .size_full()
        .flex()
        .flex_col()
        .rounded(px(ctx.style.radius_for(BStyleRole::Card)))
        .bg(to_hsla(ctx.style.colors.bg_panel))
        .border(px(ctx.style.strokes.border))
        .border_color(to_hsla(ctx.style.colors.border))
        .overflow_hidden()
        .child(
            div()
                .h(gpui::relative(header_share))
                .px(px(ctx.style.spacing.pad_sm))
                .flex()
                .items_center()
                .bg(to_hsla(header_color(node, ctx)))
                // A muted node's title fades with its header (`BUT_INACTIVE`).
                .text_color(to_hsla(title_ink(node, ctx)))
                .text_size(px(ctx.style.typography.header))
                .when(size.width >= LABEL_FLOOR, |header| {
                    header.child(SharedString::from(node.title().to_string()))
                }),
        )
        // The body is empty on purpose: everything in it — the socket dots and
        // their names — is painted over the canvas, because a socket sits ON
        // the card's border and a div cannot be drawn half outside its parent.
        // What the body contributes is its HEIGHT, which is Blender's; it
        // fills whatever the header leaves.
        .child(div().flex_1());
    if selected {
        card = card
            .border(px(ctx.style.strokes.border * 2.0))
            .border_color(to_hsla(ctx.style.colors.accent));
    }
    let handler = ctx.handler.clone();
    let action = BAction::Set(NodeKey::Selected(ctx.host).path(), id.clone());
    let placed = div()
        .id(SharedString::from(format!("node-{id}")))
        .absolute()
        .left(gpui::relative(at.0))
        .top(gpui::relative(at.1))
        .w(gpui::relative(size.width))
        .h(gpui::relative(total))
        .child(card)
        .on_click(move |_e: &ClickEvent, w, cx| {
            (Rc::clone(&handler))(&action, w, cx);
        });
    drag::grab(placed, drag::BNodeDrag::new(crate::v2::state::intern(&id), from)).into_any_element()
}
