//! Profile modal — Phiber user identity + session actions.

pub mod set;

use crate::v2::state::AppKey;
use crate::v2::state::BKey as _;
use crate::v2::state::BState;
use crate::v2::view::prim::BAction;
use crate::v2::view::prim::BCard;
use crate::v2::view::prim::BControl;
use crate::v2::view::prim::BStrip;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
pub use set::ACTIONS;
pub use set::BProfileField;
pub use set::FIELDS;

#[derive(Clone, Copy, Debug, Default)]
pub struct BProfileModal;

impl BViewItem for BProfileModal {
    /// Identity fields read from `profile.<field>`; session actions below.
    /// Sign-in is not connected yet, so empty fields show "Not signed in".
    fn build(&self, s: &BState) -> BView {
        // No heading inside: the modal's own title bar already names it, and
        // a second header in the card repeated it (`modal::title`).
        //
        // The identity block reads like a profile card — avatar, display name,
        // handle — above the fields that edit them.
        let read = |key: &'static str| {
            s.get(AppKey::Profile(key))
                .filter(|v| !v.is_empty())
                .map(str::to_string)
        };
        let name = read("display_name").unwrap_or_else(|| "Not signed in".to_string());
        let handle = read("handle").map(|h| format!("@{h}")).unwrap_or_default();
        let identity = BView::row(vec![
            BView::icon("user"),
            BView::column(vec![BView::label(name), BView::label(handle)]),
        ]);
        let mut rows: Vec<BView> = vec![identity];
        rows.extend(FIELDS.iter().map(|f| {
            let key = AppKey::Profile(f.key).path();
            let value = s.get(&key).unwrap_or_default().to_string();
            let editing = crate::v2::state::edit::is_editing(s, &key);
            BView::split(vec![
                    BView::row(vec![BView::icon(f.icon), BView::label(f.label)]),
                    BView::text_field(
                        "Not signed in",
                        value,
                        editing,
                        BAction::Set(key, String::new()),
                    ),
                ],)
        }));
        let actions = ACTIONS
            .iter()
            .map(|a| {
                BView::Button(
                    BControl::new()
                        .with_icon(a.icon)
                        .with_label(a.label)
                        .with_action(BAction::Set(
                            AppKey::Profile("action").path(),
                            a.key.to_string(),
                        )),
                )
            })
            .collect::<Vec<_>>();
        let mut footer = vec![BView::spacer()];
        footer.extend(actions);
        BView::column(vec![
            BCard::headless().body(rows).into(),
            BView::from(BCard::strip(BStrip::Footer, footer)),
        ])
    }
}
