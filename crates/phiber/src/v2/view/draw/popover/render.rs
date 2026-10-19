//! `BPopover::render` — the walker that places the panel and paints its
//! body. The data types and the switcher-column helper stay in `mod.rs`;
//! this file is just the painter, split so each sits under the 240-line
//! read cap (`tests/level_structure.rs`).

use super::*;

impl BPopover<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        if !self.open {
            return walk(self.anchor, ctx);
        }
        // The switcher is the one popover with its own two-column picker;
        // every other side adopts the panel width so a dropdown reads as a
        // continuation of the button that opened it.
        let two_column = self.side == BPopoverSide::Switcher;
        let mut content = BPopoverPart::Content { empty: self.body.is_empty() }
            .flex(ctx)
            .scroller(gpui::SharedString::from("popover-content"), ctx);
        match two_column.then(|| self.switcher_columns(ctx)).flatten() {
            Some(columns) => content = content.child(columns),
            None => {
                for item in self.body {
                    content = content.child(walk(item, ctx));
                }
            }
        }
        let panel = BPopoverPart::Panel { two_column, width: BPopoverWidth::of(self.side) }
            .flex(ctx)
            .div(ctx)
            .id(gpui::SharedString::from("popover-panel"))
            .debug_selector(|| "popover-panel".to_string())
            .child(content);
        // Blender closes a menu block once the pointer strays far enough from
        // it (`BLOCK_MOVEMOUSE_QUIT`). `BLeave` records where the panel and
        // anchor were painted; its backdrop asks on every click and move, and
        // sits BEHIND the panel, which occludes so its own clicks never reach
        // it. That is also what gives every enum picker "select item → the
        // popover closes" without a compound action.
        let leave = leave::BLeave::default();
        let panel = panel.child(leave.panel_probe());
        let backdrop = leave.backdrop(&ctx.handler, self.close);
        let placement = self.placement(ctx);
        let opened = placement.origin().child(placement.anchor(panel.into_any_element()));
        BPopoverPart::Anchor
            .flex(ctx)
            .div(ctx)
            .child(walk(self.anchor, ctx))
            .child(leave.anchor_probe())
            .child(deferred(backdrop))
            .child(deferred(opened))
            .into_any_element()
    }
}
