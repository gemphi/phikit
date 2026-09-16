fn view(c: &BRenderCtx) -> Element {
    div()
        .w(px(c.style.dimensions.panel_width))
        .gap(px(c.style.spacing.gap_sm))
        .rounded(px(c.style.radius_for(BStyleRole::Panel)))
        .bg(to_hsla(c.style.colors.bg_panel))
}
