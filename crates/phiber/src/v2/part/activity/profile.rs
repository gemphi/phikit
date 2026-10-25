//! The Profile panel — laid out as Discord lays out a profile: the picture
//! first, large and round, with the name, the handle and the presence beside
//! it; the presence picker (Online, Idle, Do Not Disturb, Invisible) and the
//! one action that matters — Sign In, or Edit Profile once signed in — under
//! it; then the sections as accordions, the first open: **About Me**,
//! **Account** (the fields, each opening the profile dialog), **Channels**
//! (the ones they own) and **Followers**. Settings are not repeated here:
//! they have their own entry on the rail. Everything shown is read from
//! `profile.*` and `collab.*`; nothing is invented — with no account the
//! panel says "Not signed in" and the lists say they are empty.

use super::option::{channels, peers};
use crate::v2::host::BHostType;
use crate::v2::modal::profile::FIELDS;
use crate::v2::part::props::BPanelItem;
use crate::v2::state::{BKey as _, AppKey, BState, ModalKey, val::{V_FALSE, V_MODAL_SETTINGS}};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BCard, BControl};

/// The picture's diameter, Discord's popout avatar.
pub const AVATAR: f32 = 80.0;

/// The sections, in order; the first is open until folded.
pub const SECTIONS: [&str; 4] = ["About Me", "Account", "Channels", "Followers"];

/// Discord's four presences: (slug stored at `profile.status`, label, icon).
pub const PRESENCES: [(&str, &str, &str); 4] = [
    ("online", "Online", "record"),
    ("idle", "Idle", "time"),
    ("dnd", "Do Not Disturb", "circle_off"),
    ("invisible", "Invisible", "circle_dashed"),
];

fn read<'a>(s: &'a BState, key: &'static str) -> Option<&'a str> {
    s.get(AppKey::Profile(key)).filter(|v| !v.is_empty())
}

/// The fold key of a section: `ctx.activity.profile.<section>.expanded`.
pub fn fold_key(section: &str) -> crate::v2::state::BPath {
    BPanelItem::expand_key(BHostType::Activity, "profile", section)
}

fn section(title: &str, s: &BState, body: Vec<BView>) -> BView {
    let key = fold_key(title);
    let open = match s.get(&key) {
        Some(V_FALSE) => false,
        Some(_) => true,
        None => title == SECTIONS[0],
    };
    BCard::panel(title).fold(key, open).body(body).into()
}

fn row(icon: &str, label: impl Into<String>, action: Option<BAction>) -> BView {
    let mut control = BControl::new().with_icon(icon).with_label(label);
    if let Some(action) = action {
        control = control.with_action(action);
    }
    BView::MenuItem(control)
}

/// The panel's contents: identity, then the four sections.
///
/// The CONTAINER is not here. Every other secondary activity column is a
/// `BCard::panel(entry.label).icon(entry.icon)` built in `group_strip`, and this
/// one returned a bare column instead — so the Profile column was the only one
/// in the strip without the card's border and heading. One place builds the
/// container now; this returns what goes in it.
pub fn body(s: &BState) -> Vec<BView> {
    let signed_in = read(s, "handle").is_some();
    let name = read(s, "display_name").or(read(s, "handle")).unwrap_or("Not signed in").to_string();
    let handle = read(s, "handle").map(|h| format!("@{h}")).unwrap_or_else(|| "Sign in to sync your work".to_string());
    let presence = read(s, "status").unwrap_or(if signed_in { "online" } else { "invisible" });
    let presence_label = PRESENCES.iter().find(|(slug, ..)| *slug == presence).map(|(_, label, _)| *label).unwrap_or("Online");
    let identity = BView::row(vec![
        BView::Avatar { path: read(s, "avatar").unwrap_or("").to_string(), size: AVATAR },
        BView::column(vec![BView::label(name), BView::label(handle), BView::label(format!("● {presence_label}"))]),
    ]);

    // The one action: sign in, or edit the profile once signed in.
    let open_profile = BAction::Set(ModalKey::Open.path(), crate::v2::state::val::V_MODAL_PROFILE.to_string());
    let action = BView::Button(
        BControl::new()
            .with_icon(if signed_in { "edit" } else { "user" })
            .with_label(if signed_in { "Edit Profile" } else { "Sign In" })
            .with_action(open_profile.clone()),
    );
    // The presence picker: four tiles, the one in force lit, each writing
    // `profile.status` — what the dot beside the name reads.
    let presences = BView::row(
        PRESENCES
            .iter()
            .map(|(slug, label, icon)| {
                let mut control = BControl::new()
                    .with_icon(*icon)
                    .with_active(*slug == presence)
                    .with_action(BAction::Set(AppKey::Profile("status").path(), slug.to_string()));
                control.tooltip = Some(label.to_string());
                BView::Button(control)
            })
            .collect(),
    );

    // About Me: the status line and the theme they work in, as Discord's
    // "About Me" and "Member Since" read — each opening the profile dialog.
    let about: Vec<BView> = [("info", "Status", "status"), ("preferences", "Theme", "theme"), ("chat", "Email", "email")]
        .into_iter()
        .map(|(icon, label, key)| row(icon, format!("{label}  ·  {}", read(s, key).unwrap_or("—")), Some(open_profile.clone())))
        .collect();
    // Account: every field, each a row that opens the profile dialog to edit.
    let account: Vec<BView> = FIELDS
        .iter()
        .map(|field| {
            let value = read(s, field.key).unwrap_or("—");
            row(field.icon, format!("{}  ·  {value}", field.label), Some(open_profile.clone()))
        })
        .collect();
    let _ = V_MODAL_SETTINGS;
    // Channels they own: the session's channels whose owner is this handle.
    let own = read(s, "handle").unwrap_or("");
    let owned: Vec<BView> = channels(s)
        .into_iter()
        .filter(|(_, owner)| !own.is_empty() && owner == own)
        .map(|(channel, _)| row("hash", channel, None))
        .collect();
    let channels = if owned.is_empty() { vec![row("info", "No channels yet", None)] } else { owned };
    // Followers: the peers who follow (`state` says `follows`).
    let following: Vec<BView> = peers(s)
        .into_iter()
        .filter(|(_, state)| state.contains("follow"))
        .map(|(peer, _)| row("user", peer, None))
        .collect();
    let followers = if following.is_empty() { vec![row("info", "No followers yet", None)] } else { following };

    vec![
        identity,
        presences,
        action,
        section(SECTIONS[0], s, about),
        section(SECTIONS[1], s, account),
        section(SECTIONS[2], s, channels),
        section(SECTIONS[3], s, followers),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cards(list: &[BView]) -> Vec<(String, bool)> {
        list.iter()
            .filter_map(|v| match v {
                BView::Card(card) => Some((card.title().unwrap_or("").to_string(), card.is_open())),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_panel_leads_with_the_picture_and_folds_its_four_sections_the_first_open() {
        let mut state = BState::new();
        state.set(AppKey::Profile("avatar"), "/pictures/me.png");
        state.set(AppKey::Profile("handle"), "ann");
        state.set(AppKey::CollabChannel("0"), "renders\tann");
        state.set(AppKey::CollabChannel("1"), "review\tbob");
        state.set(AppKey::CollabPeer("0"), "bob\tfollows you");
        let view = body(&state);
        let list = &view;
        let BView::Row { list: identity } = &list[0] else { panic!("identity first") };
        assert!(matches!(&identity[0], BView::Avatar { path, size } if path == "/pictures/me.png" && *size == AVATAR));
        assert_eq!(
            cards(&view),
            [("About Me".to_string(), true), ("Account".to_string(), false), ("Channels".to_string(), false), ("Followers".to_string(), false)]
        );
        // Fold the first, open the third: the keys decide.
        state.set(&fold_key("About Me"), V_FALSE);
        state.set(&fold_key("Channels"), "true");
        let opened = cards(&body(&state));
        assert!(!opened[0].1 && opened[2].1);
        state.set(&fold_key("Followers"), "true");
        let text = crate::v2::view::layout::BRenderer::render(&crate::v2::view::text::BAsciiRenderer, &BView::column(body(&state)), &state);
        assert!(text.contains("renders") && !text.contains("review"), "only the channels they own: {text}");
        assert!(text.contains("bob"), "the follower: {text}");
    }

    #[test]
    fn signed_out_the_panel_says_so_and_the_lists_are_honest() {
        let mut state = BState::new();
        for section in SECTIONS {
            state.set(&fold_key(section), "true");
        }
        let text = crate::v2::view::layout::BRenderer::render(&crate::v2::view::text::BAsciiRenderer, &BView::column(body(&state)), &state);
        assert!(text.contains("Not signed in") && text.contains("No channels yet") && text.contains("No followers yet"), "{text}");
    }
}

#[cfg(test)]
mod presence_tests {
    use super::*;

    #[test]
    fn the_presence_picker_writes_the_status_and_lights_the_one_in_force() {
        let mut state = BState::new();
        state.set(AppKey::Profile("handle"), "ann");
        state.set(AppKey::Profile("status"), "dnd");
        let view = body(&state);
        let BView::Row { list } = &view[1] else { panic!("the presences follow the identity") };
        assert_eq!(list.len(), PRESENCES.len());
        let lit: Vec<bool> = list.iter().map(|b| matches!(b, BView::Button(c) if c.active)).collect();
        assert_eq!(lit, [false, false, true, false], "Do Not Disturb is lit");
        assert!(matches!(&view[2], BView::Button(c) if c.label.as_deref() == Some("Edit Profile")), "signed in: Edit Profile");
        let out = body(&BState::new());
        assert!(matches!(&out[2], BView::Button(c) if c.label.as_deref() == Some("Sign In")), "signed out: Sign In");
    }
}
