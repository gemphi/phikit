# Modals - State Structure

> **Source files:** `crates/phiber/src/state/modals/`
> **Hierarchy:** Modals are space-display types - they have their own state layer.

```
Space → Zone → BArea → Host → Region → Pane → Part → BView

Modal (overlay, not in the grid hierarchy)
  ├─ ModalState (which modal is open, active tab/selection)
  ├─ SettingsState (active category, search query)
  ├─ HelpState (selected link)
  └─ ProfileState (user info, selected action)

Host switching is NOT a modal - it's a BPartType::Switcher menu part.
```

---

## State File Layout

```
crates/phiber/src/state/modals/
  mod.rs              - module declarations, ModalState enum
  settings.rs         - SettingsState + view function
  help.rs             - HelpState + view function
  profile.rs          - ProfileState + view function
```

### Planned restructure (mod.rs/def.rs/ctx.rs/use.rs per modal)

```
crates/phiber/src/state/modals/
  mod.rs                          - ModalState enum, re-exports
  def.rs                          - default modal state per type
  ctx.rs                          - modal context sync
  use.rs                          - modal set methods (set_open, set_close, set_tab)

  settings/
    mod.rs                        - SettingsState { active_category, search }
    def.rs                        - default settings state
    ctx.rs                        - settings context sync
    use.rs                        - set_category, set_search

  help/
    mod.rs                        - HelpState { selected_link }
    def.rs                        - default help state
    ctx.rs                        - help context sync
    use.rs                        - set_selected_link

  profile/
    mod.rs                        - ProfileState { user, selected_action }
    def.rs                        - default profile state
    ctx.rs                        - profile context sync
    use.rs                        - set_action, set_user
```

---

## ModalState Enum (planned)

```rust
//! ModalState - tracks which modal is open and its internal state.
//! Ref: Blender's screen modal overlay

use buikit::BModalType;

#[derive(Clone, Debug)]
pub enum ModalState {
    Closed,
    Settings(SettingsState),
    Help(HelpState),
    Profile(ProfileState),
}

impl ModalState {
    pub fn is_open(&self) -> bool {
        !matches!(self, ModalState::Closed)
    }

    pub fn idx(&self) -> Option<BModalType> {
        match self {
            ModalState::Closed => None,
            ModalState::Settings(_) => Some(BModalType::Settings),
            ModalState::Help(_) => Some(BModalType::Help),
            ModalState::Profile(_) => Some(BModalType::Profile),
        }
    }
}

impl Default for ModalState {
    fn default() -> Self {
        ModalState::Closed
    }
}
```

---

## Settings State

```rust
#[derive(Clone, Debug)]
pub struct SettingsState {
    pub active_category: SettingType,
    pub search: String,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            active_category: SettingType::Interface,
            search: String::new(),
        }
    }
}

impl SettingsState {
    pub fn set_category(&mut self, category: SettingIdx) {
        self.active_category = category;
    }

    pub fn set_search(&mut self, query: String) {
        self.search = query;
    }
}
```

### Zed comparison

Zed's `SettingsWindow` has extensive state:
- `current_file: SettingsUiFile` - which settings file is being edited
- `pages: Vec<SettingsPage>` - all settings pages with items
- `sub_page_stack: Vec<SubPage>` - navigation stack for sub-pages
- `search_bar: Entity<Editor>` - search input
- `search_task: Option<Task<()>>` - async search
- `navbar_entries: Vec<NavBarEntry>` - navigation sidebar entries
- `filter_table: Vec<Vec<bool>>` - filtered visibility per item
- `project_setting_file_buffers` - cached file buffers

Phiber's `SettingsState` is minimal - just active category and search string.

---

## Help State

```rust
#[derive(Clone, Debug, Default)]
pub struct HelpState {
    pub selected_link: Option<HelpType>,
}
```

---

## Profile State

```rust
#[derive(Clone, Debug, Default)]
pub struct ProfileState {
    pub user: Option<String>,        // None = guest
    pub selected_action: Option<ProfileType>,
}
```

---

## Modal Context Sync (planned)

```rust
//! state/modals/ctx.rs - modal context sync.
//! Modals don't nest in BContext (they're overlays, not in the UI tree).
//! Instead, modal state is accessed directly by the app's view layer.

// Modals bypass BContext - they read from ModalState directly.
// The app's modal view reads ModalState and calls the appropriate
// view function to produce Vec<BView>.
```

### Zed comparison

Zed's `ModalLayer` manages focus context:
- `previous_focus_handle: Option<FocusHandle>` - saved before modal opens
- `focus_handle: FocusHandle` - modal's own focus handle
- On dismiss: restores `previous_focus_handle`
- `dismiss_on_focus_lost: bool` - auto-dismiss when focus leaves

Phiber doesn't have focus management yet. To match Zed:
- Save the active space/zone/area focus before opening a modal
- Restore it on close
- Handle `on_before_dismiss` for unsaved changes confirmation

---

## Set Methods (planned)

```rust
//! state/modals/use.rs - modal set methods.
//! Pure state mutations - the caller invokes these, then calls cx.notify().

impl ModalState {
    pub fn set_open(&mut self, modal_type: BModalType) {
        *self = match modal_type {
            BModalType::Settings => ModalState::Settings(SettingsState::default()),
            BModalType::Help => ModalState::Help(HelpState::default()),
            BModalType::Profile => ModalState::Profile(ProfileState::default()),
        };
    }

    pub fn set_close(&mut self) {
        *self = ModalState::Closed;
    }

    pub fn set_toggle(&mut self, modal_type: BModalType) {
        if self.idx() == Some(modal_type) {
            self.set_close();
        } else {
            self.set_open(modal_type);
        }
    }
}
```

### Zed comparison

Zed's `ModalLayer::toggle_modal()`:
```rust
pub fn toggle_modal<V, B>(&mut self, window, cx, build_view: B)
where V: ModalView, B: FnOnce(&mut Window, &mut Context<V>) -> V
{
    if let Some(active_modal) = &self.active_modal {
        let should_close = active_modal.modal.view().downcast::<V>().is_ok();
        let did_close = self.hide_modal(window, cx);
        if should_close || !did_close { return; }
    }
    let new_modal = cx.new(|cx| build_view(window, cx));
    self.show_modal(Box::new(new_modal), window, cx);
}
```

Key difference: Zed's toggle checks type identity via `downcast`, handles
`DismissDecision::Pending` (async dismiss), and manages focus restoration.
Phiber's `set_toggle` is a simple enum match.
