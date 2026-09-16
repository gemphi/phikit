# Modals - View Structure

> **Source files:** `crates/buikit/src/modal.rs`, `crates/phiber/src/state/modals/`
> **Zed reference:** `crates/workspace/src/modal_layer.rs`, `crates/ui/src/components/modal.rs`

---

## View Architecture

```
App view layer
  └─ b_modal() container
       ├─ Header (title + dismiss button)
       ├─ Body (BView tree from view function)
       └─ Footer (optional action buttons)
```

The app's view layer reads `ModalState`, calls the appropriate view function
to get `Vec<BView>`, wraps it in a modal container, and renders it.

---

## Render Flow

```
User action (e.g., Edit → Preferences)
  ↓
App dispatches: modal_state.set_toggle(BModalType::Settings)
  ↓
App view layer checks modal_state
  ↓
If open: view function reads store specs + state → Vec<BView>
  ↓
Returns Vec<BView>
  ↓
App wraps in b_modal(title, width, body)
  ↓
GPUI renders the modal overlay
```

---

## Settings Modal View

```
b_modal("Settings", 640.0)
  └─ BView::Row
       ├─ BView::Column (sidebar, 200px)
       │    ├─ BView::Label("Interface")
       │    ├─ BView::Label("Editing")
       │    ├─ BView::Label("Navigation")
       │    ├─ BView::Label("Add-ons")
       │    ├─ BView::Label("Themes")
       │    └─ BView::Label("System")
       └─ BView::Column (content, flex-1)
            └─ BView::Label("Select a category from the sidebar")
```

### Current implementation

```rust
// state/modals/settings.rs
fn settings_modal_view(state: &SettingsState, _cx: &App) -> Vec<BView> {
    let categories = SettingsModalSpec::categories();
    let mut sidebar: Vec<BView> = Vec::new();
    for cat in categories {
        sidebar.push(BView::label(cat.label));
    }

    vec![
        BView::Row {
            gap: Some(8.0),
            children: vec![
                BView::Column { gap: Some(2.0), children: sidebar },
                BView::Column { gap: Some(8.0), children: vec![
                    BView::label("Select a category from the sidebar"),
                ]},
            ],
        },
    ]
}
```

### Planned: full settings view with rows

```
b_modal("Settings", 640.0)
  └─ Row
       ├─ Column (sidebar)
       │    └─ for each category: Button(label, active=selected)
       └─ Column (content)
            └─ for each row in active_category.rows():
                 Row
                   ├─ Label(row.0)   // setting name
                   └─ Label(row.1)   // current value
```

### Zed's settings UI

Zed opens settings as a **separate window** (`SettingsWindow`), not a modal:

```
SettingsWindow (separate GPUI window)
  ├─ PlatformTitleBar
  ├─ Navbar (left sidebar, 226px)
  │    ├─ Search bar (Editor entity)
  │    ├─ NavBarEntry list (uniform_list)
  │    │    └─ Page entries with icons, expand/collapse
  └─ Content area (min 400px)
       ├─ SettingsSectionHeader
       └─ SettingField controls
            ├─ DropdownMenu (enum variants)
            ├─ NumberField (numeric settings)
            ├─ SettingsInputField (text settings)
            ├<arg_value>Switch (boolean settings)
            ├─ font_picker, theme_picker, icon_theme_picker
            └─ Sub-pages (LLM providers, MCP servers, skills, etc.)
```

Key Zed features Phiber should adopt:
- **Search** - filter settings by name across all categories
- **Sub-pages** - drill into specific settings (e.g., LLM provider config)
- **Reset to default** - per-field reset button
- **File indicator** - show which settings file defines each value
- **Organization override** - show when a setting is org-managed

---

## Help Modal View

```
b_modal("Help", 480.0)
  └─ Column
       ├─ Label("Help")
       ├─ for each link: LabelIcon(label, icon)
       │    ├─ "Documentation" (book)
       │    ├─ "Tutorials" (play)
       │    ├─ "Keyboard Shortcuts" (keyboard)
       │    ├─ "API Reference" (code)
       │    ├─ "Community Forum" (users)
       │    └─ "Report a Bug" (bug)
       ├─ Separator
       └─ Label("Phiber v0.1.0 - Built with GPUI + Rust")
```

### Current implementation

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

## Profile Modal View

```
b_modal("Profile", 400.0)
  └─ Column
       ├─ Row
       │    ├─ Label("Guest User")
       │    └─ Label("Not signed in")
       ├─ for each action: LabelIcon(label, icon)
       │    ├─ "Sign In" (login)
       │    ├─ "Preferences" (settings)
       │    ├─ "Theme Settings" (palette)
       │    ├─ "Keyboard Shortcuts" (keyboard)
       │    └─ "Sign Out" (logout)
```

### Current implementation

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

## Zed's ModalLayer - Full Comparison

### Zed's modal rendering

```rust
// crates/workspace/src/modal_layer.rs
impl Render for ModalLayer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(active_modal) = &self.active_modal else {
            return div().into_any_element();
        };

        if active_modal.modal.render_bare(cx) {
            return active_modal.modal.view().into_any_element();
        }

        div()
            .absolute()
            .size_full()
            .inset_0()
            .occlude()                          // blocks clicks behind
            .when(fade_out, |this| this.bg(...)) // dim background
            .on_mouse_down(MouseButton::Left, |this, _, cx| {
                this.hide_modal(window, cx);    // click-outside to dismiss
            })
            .child(
                v_flex()
                    .h(px(0.0))
                    .top_20()
                    .items_center()
                    .track_focus(&active_modal.focus_handle)
                    .child(
                        h_flex()
                            .occlude()
                            .child(active_modal.modal.view())
                            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                cx.stop_propagation();  // don't dismiss when clicking modal
                            }),
                    ),
            )
    }
}
```

### Zed's Modal UI component

```rust
// crates/ui/src/components/modal.rs
pub struct Modal {
    id: ElementId,
    header: ModalHeader,      // icon, headline, description, dismiss/back buttons
    children: SmallVec<[AnyElement; 2]>,  // sections
    footer: Option<ModalFooter>,  // start_slot, end_slot
}

// Sections contain the actual settings content
pub struct Section {
    contained: bool,
    padded: bool,
    header: Option<SectionHeader>,  // label + end_slot
    children: SmallVec<[AnyElement; 2]>,
}
```

### What Phiber should adopt from Zed

1. **Click-outside-to-dismiss** - `on_mouse_down` on overlay, `stop_propagation` on modal body
2. **Focus management** - save/restore focus handle across modal open/close
3. **DismissDecision** - `Dismiss(true)`, `Dismiss(false)`, `Pending` for async confirmations
4. **ModalHeader/ModalFooter** - structured header with dismiss button, footer with action buttons
5. **Section component** - grouped content with header labels
6. **Picker stashing** - keep picker modals alive (hidden) for reopening with prior state
7. **Fade-out background** - dim the workspace behind the modal

### What Phiber should keep different

1. **BView data** - view functions return pure data, not GPUI elements. This keeps
   the view layer decoupled from the modal logic.
2. **ModalState** - app manages open/closed state directly. Zed's `ModalLayer` entity
   approach is more complex but handles focus/stash automatically.
3. **BModalType enum** - fixed set of modals (Settings, Help, Profile). Zed's
   `ModalView` trait allows any type. For Phiber's use case (3 known modals),
   the enum is sufficient.
