//! The name a card's selectors are built from (`card-head-<name>`,
//! `card-title-<name>`, …), split out of `card.rs` for its line cap.

use crate::v2::view::BView;
use crate::v2::view::prim::{BCard, BCardKind};

/// The name a card's selectors are built from: its fold key, its title, or
/// — for a headless card — the first words of its first row.
pub(super) fn slot(card: &BCard) -> String {
    let words = match (&card.fold, card.title()) {
        (Some(fold), _) => return fold.key.encode().replace('.', "-"),
        (None, Some(title)) if !title.is_empty() => title.to_string(),
        _ => card.body.first().map(first_words).unwrap_or_default(),
    };
    if words.is_empty() { "card".to_string() } else { words.to_lowercase().replace(' ', "-") }
}

fn first_words(v: &BView) -> String {
    match v {
        BView::Label { text } => text.clone(),
        BView::Button(ctl) => ctl.label.clone().unwrap_or_default(),
        BView::Icon { name } => name.clone(),
        BView::Row { list } | BView::Column { list } | BView::Split { list } => {
            list.first().map(first_words).unwrap_or_default()
        }
        // A region column is named by what it holds, as its panel role was.
        BView::Card(card) if matches!(card.kind, BCardKind::Column(_)) => {
            card.body.first().map(first_words).unwrap_or_default()
        }
        BView::ListRow { list, .. } => list.first().map(first_words).unwrap_or_default(),
        _ => String::new(),
    }
}
