//! About window — Blender's `WM_MT_splash_about` (`bl_operators/wm.py:3496`).
//!
//! Left column (65 %): Version, Date, Hash, Branch, Windowing Environment.
//! Right column: the app icon. Below: Release Notes · Credits · License ·
//! Website · Donate (`WM_MT_splash_about`, L3527-3540). One rounded Card per
//! block, like the welcome window.

pub mod set;

use crate::v2::modal::splash::APP_NAME;
use crate::v2::state::BState;
use crate::v2::view::prim::BCard;
use crate::v2::view::prim::BControl;
use crate::v2::view::prim::BStrip;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
pub use set::ABOUT_LINKS;
pub use set::BAboutLink;

#[derive(Clone, Copy, Debug, Default)]
pub struct BAboutModal;

fn fact(label: &str, value: String) -> BView {
    BView::row(vec![BView::label(format!("{label}: ")), BView::label(value)])
}

fn facts() -> BView {
    BCard::panel(APP_NAME)
        .body(vec![
            fact("Version", env!("CARGO_PKG_VERSION").to_string()),
            fact("Date", option_env!("PHIBER_BUILD_DATE").unwrap_or("dev build").to_string()),
            fact("Hash", option_env!("PHIBER_BUILD_HASH").unwrap_or("unknown").to_string()),
            fact("Branch", option_env!("PHIBER_BUILD_BRANCH").unwrap_or("main").to_string()),
            fact("Windowing Environment", "gpui".to_string()),
        ])
        .into()
}

fn brand() -> BView {
    BCard::headless().body(vec![BView::icon("phiber_splash")]).into()
}

fn links() -> BView {
    BView::from(BCard::strip(BStrip::Footer,
        ABOUT_LINKS
            .iter()
            .map(|l| {
                BView::Button(
                    BControl::new()
                        .with_icon(l.icon)
                        .with_label(l.label)
                        .with_action(crate::v2::modal::splash::set::link_action(l.url)),
                )
            })
            .collect(),
    ))
}

impl BViewItem for BAboutModal {
    fn build(&self, _s: &BState) -> BView {
        BView::column(vec![BView::row(vec![facts(), brand()]), links()])
    }
}
