# Profile Modal - Full Documentation

> **Type:** `BModalType::Profile`
> **Width:** 400.0
> **Source:** `store/modals/profile.rs`, `state/modals/profile.rs`

---

## Overview

The Profile modal shows account info and session actions. Equivalent to
Zed's account settings and Blender's profile/login state.

---

## View Tree

```
b_modal("Profile", 400.0)
  └─ Column
       ├─ Row (gap: 8)
       │    ├─ Label("Guest User")
       │    └─ Label("Not signed in")
       ├─ LabelIcon("Sign In", "login")
       ├─ LabelIcon("Preferences", "settings")
       ├─ LabelIcon("Theme Settings", "palette")
       ├─ LabelIcon("Keyboard Shortcuts", "keyboard")
       └─ LabelIcon("Sign Out", "logout")
```

---

## Store Spec

### `ProfileIdx` enum

```rust
pub enum ProfileType {
    SignIn,
    Preferences,
    ThemeSettings,
    KeyboardShortcuts,
    SignOut,
}
```

### Icons

| Action | Icon |
|--------|------|
| Sign In | `login` |
| Preferences | `settings` |
| Theme Settings | `palette` |
| Keyboard Shortcuts | `keyboard` |
| Sign Out | `logout` |

---

## State

```rust
#[derive(Clone, Debug, Default)]
pub struct ProfileState {
    pub user: Option<String>,        // None = guest
    pub selected_action: Option<ProfileType>,
}
```

---

## View Function

```rust
// state/modals/profile.rs
fn profile_modal_view(state: &ProfileState, _cx: &App) -> Vec<BView> {
    let actions = ProfileModalSpec::actions();
    let mut items: Vec<BView> = vec![
        BView::Row {
            gap: Some(8.0),
            children: vec![
                BView::label("Guest User"),
                BView::label("Not signed in"),
            ],
        },
    ];
    for action in actions {
        items.push(BView::label_icon(action.label, action.icon));
    }
    items
}
```

---

## Planned Improvements

- **User avatar** - show profile picture when signed in
- **Action dispatch** - each action should trigger a callback, not just display
- **Conditional actions** - hide "Sign In" when signed in, hide "Sign Out" when guest
- **Theme picker** - inline theme selection instead of opening Settings
- **Keyboard shortcut editor** - inline keymap editor instead of opening Settings

---

## Zed Comparison

Zed handles profile/account through:
- `agent_configuration/manage_profiles_modal.rs` - manage AI agent profiles
- `collab_ui` - collaboration panel with sign-in/sign-out
- Settings window has account/provider configuration pages

Phiber's Profile modal is simpler - a single modal with action buttons.
To match Zed, it would need:
- OAuth/login flow
- Profile picture and display name
- Per-provider configuration (OpenAI, Anthropic, Ollama, etc.)
- Team/workspace switching
