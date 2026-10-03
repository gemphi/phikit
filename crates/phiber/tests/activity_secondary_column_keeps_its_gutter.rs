//! The activity bar's secondary column does not run into its area's edges.
//!
//! The column is the Center pane of the Activity host's Center region, beside
//! the rail's Left pane. That region leads with a rail, and the rail branch of
//! the region walker padded only top and bottom — so the column ran flush into
//! the area's right border, while every other content region keeps a gutter
//! there. It is also the one rail region with anything beside the rail, which
//! is why nothing else showed the problem.
//!
//! The layout fingerprint cannot see any of this: it captures no state with a
//! group open, so the column is never painted in it. Hence this.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{ActivityKey, BKey as _, BState, CtxKey};

/// Open the secondary column on a grouping entry.
macro_rules! group_open {
    ($cx:ident) => {{
        let (app, cx) = $cx.add_window_view(|_, _| BPhiberApp::new());
        cx.simulate_resize(size(px(1400.0), px(900.0)));
        app.update(cx, |app, cx| {
            app.state = BState::new();
            app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
            app.state.set(&ActivityKey::Group.path(), "Spaces");
            cx.notify();
        });
        cx.run_until_parked();
        cx
    }};
}

#[gpui::test]
async fn the_secondary_column_is_painted_when_a_group_is_open(cx: &mut TestAppContext) {
    let cx = group_open!(cx);
    assert!(
        cx.debug_bounds("region-Center-Activity").is_some(),
        "the activity host's content region is painted"
    );
}

#[gpui::test]
async fn the_column_clears_the_rail_and_the_areas_right_edge(cx: &mut TestAppContext) {
    let cx = group_open!(cx);
    let region = cx.debug_bounds("region-Center-Activity").expect("the activity content region");
    let rail = cx.debug_bounds("pane-Left-Activity").expect("the rail pane");
    let column = cx.debug_bounds("pane-Center-Activity").expect("the secondary column pane");

    let right = |b: &gpui::Bounds<gpui::Pixels>| f32::from(b.origin.x + b.size.width);
    let left = |b: &gpui::Bounds<gpui::Pixels>| f32::from(b.origin.x);

    // The column starts after the rail ends, not at it: a pane boundary keeps
    // the same gutter every other host's panes keep.
    assert!(
        left(&column) > right(&rail),
        "the column starts at {} but the rail ends at {} — they are touching",
        left(&column),
        right(&rail)
    );

    // And it stops before the region's own right edge, so its rows do not
    // butt against the area's border.
    assert!(
        right(&column) < right(&region),
        "the column ends at {} and the region at {} — it is flush with the border",
        right(&column),
        right(&region)
    );
}
