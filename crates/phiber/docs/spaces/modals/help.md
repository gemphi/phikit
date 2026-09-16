# Help Modal - Full Documentation

> **Type:** `BModalType::Help`
> **Width:** 480.0
> **Source:** `store/modals/help.rs`, `state/modals/help.rs`

---

## Overview

The Help modal shows documentation links, tutorials, keyboard shortcuts,
and version info. Equivalent to Blender's `Help` menu and Zed's help commands.

---

## View Tree

```
b_modal("Help", 480.0)
  └─ Column
       ├─ Label("Help")
       ├─ LabelIcon("Documentation", "book")
       ├─ LabelIcon("Tutorials", "play")
       ├─ LabelIcon("Keyboard Shortcuts", "keyboard")
       ├─ LabelIcon("API Reference", "code")
       ├─ LabelIcon("Community Forum", "users")
       ├─ LabelIcon("Report a Bug", "bug")
       ├─ Separator (horizontal)
       └─ Label("Phiber v0.1.0 - Built with GPUI + Rust")
```

---

## Store Spec

### `HelpIdx` enum

```rust
pub enum HelpType {
    Documentation,
    Tutorials,
    KeyboardShortcuts,
    ApiReference,
    CommunityForum,
    ReportBug,
}
```

### Icons

| Link | Icon |
|------|------|
| Documentation | `book` |
| Tutorials | `play` |
| Keyboard Shortcuts | `keyboard` |
| API Reference | `code` |
| Community Forum | `users` |
| Report a Bug | `bug` |

---

## State

```rust
#[derive(Clone, Debug, Default)]
pub struct HelpState {
    pub selected_link: Option<HelpType>,
}
```

---

## View Function

```rust
// state/modals/help.rs
fn help_view(state: &HelpState, _cx: &App) -> Vec<BView> {
    let links = BHelpModalSpec::links();
    let mut items: Vec<BView> = vec![BView::label("Help")];
    for link in links {
        items.push(BView::label_icon(link.label, link.icon));
    }
    items.push(BView::Separator { vertical: false });
    items.push(BView::label("Phiber v0.1.0 - Built with GPUI + Rust"));
    items
}
```

---

## Planned Improvements

- **Clickable links** - each `LabelIcon` should be a `Button` that opens the URL
- **Keyboard shortcut overlay** - inline shortcut display instead of external link
- **Search** - filter help links by name
- **About section** - show build info, GPU info, Rust version
- **Tooltip on hover** - description of each help link

---

## Zed Comparison

Zed doesn't have a dedicated help modal. Instead:
- `zed::open_url_modal.rs` - opens URLs from the command palette
- Help commands are dispatched via the command palette (`Cmd-Shift-P`)
- Documentation links open in the system browser
- Version info is in the About dialog (platform-native)

Phiber's Help modal is more structured - a dedicated modal with categorized links.
